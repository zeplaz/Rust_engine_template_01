"""MCP-WIT-003 — witness honesty validator (BLANG:WIT-HON)."""

from __future__ import annotations

import json
import re
import time
from fnmatch import fnmatch
from pathlib import Path
from typing import Any, Iterator

from rust_engine_mcp.paths import repo_root, schemas_dir

from .report import KnownFix, ValidationIssue, ValidationReport

CATALOG_REL = "tools/mcp/schemas/witness_integrity_rules_v1.json"
WITNESS_REL = "debug_runs/mcp_witness_honesty_validator_live.json"
FIXTURES_REL = "tools/mcp/schemas/examples/witness_honesty_fixtures"

_STATUS_CLOSED = frozenset({"done", "lib_done", "signed", "closed"})
_STATUS_OPEN = frozenset({"blocked", "paused", "deferred", "open", "reopened"})


def _resolve(path: str | Path) -> Path:
    p = Path(path)
    if not p.is_absolute():
        p = repo_root() / p
    return p.resolve()


def _rel(path: Path, root: Path) -> str:
    try:
        return str(path.relative_to(root)).replace("\\", "/")
    except ValueError:
        return str(path).replace("\\", "/")


def load_witness_integrity_catalog(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    path = root / CATALOG_REL
    return json.loads(path.read_text(encoding="utf-8"))


def validate_witness_integrity_catalog(*, repo: Path | None = None) -> None:
    import jsonschema

    root = repo or repo_root()
    catalog = load_witness_integrity_catalog(repo=root)
    schema = json.loads((schemas_dir() / "witness_integrity_rules_v1.schema.json").read_text(encoding="utf-8"))
    jsonschema.validate(instance=catalog, schema=schema)


def _get_dot(data: Any, dot_path: str) -> Any:
    cur = data
    for part in dot_path.split("."):
        if not isinstance(cur, dict):
            return None
        cur = cur.get(part)
    return cur


def _path_matches(rel: str, pattern: str | None) -> bool:
    if not pattern:
        return True
    return fnmatch(rel, pattern) or fnmatch(rel, pattern.removeprefix("**/"))


def _is_green(data: dict[str, Any], green_fields: list[str] | None = None) -> bool:
    for field in green_fields or ("green", "all_green"):
        val = data.get(field)
        if val is True:
            return True
    return False


def _agent_commands_all_lib(meta: dict[str, Any]) -> bool:
    cmds = meta.get("agent_commands") or []
    if not cmds:
        return False
    return all("--lib" in str(cmd) for cmd in cmds)


def _exit_predicate_fails(data: dict[str, Any]) -> str | None:
    block = data.get("exit_predicate")
    if not isinstance(block, dict):
        return None
    must = block.get("must") or []
    for clause in must:
        if not isinstance(clause, dict):
            continue
        path = str(clause.get("path") or "")
        expected = clause.get("eq")
        actual = _get_dot(data, path) if path else None
        if actual != expected:
            return f"exit_predicate.must {path}=={expected!r} but got {actual!r}"
    return None


def _normalize_queue_status(raw: str) -> str:
    s = str(raw or "").strip().lower()
    if s in _STATUS_CLOSED:
        return "closed"
    if s in _STATUS_OPEN:
        return "open"
    if s in {"ready", "active", "in_progress"}:
        return "ready"
    return s or "unknown"


def _iter_queue_rows(queue_doc: dict[str, Any], entry: dict[str, Any]) -> Iterator[dict[str, Any]]:
    rows_path = str(entry.get("rows_path") or "rows")
    if rows_path == "p2_tasks":
        yield from queue_doc.get("p2_tasks") or []
        drain = queue_doc.get("coder_mcp_drain") or {}
        for bucket in ("done_coder_mcp", "done_designer_mcp", "done_planner_mcp"):
            for slice_id in drain.get(bucket) or []:
                yield {"id": slice_id, "status": "done", "_synthetic_done_bucket": bucket}
        return
    block = queue_doc.get(rows_path)
    if isinstance(block, list):
        yield from block


def _rollup_entry_for(rel: str, catalog: dict[str, Any]) -> dict[str, Any] | None:
    for entry in catalog.get("rollup_registry") or []:
        if str(entry.get("path") or "").replace("\\", "/") == rel:
            return entry
    return None


def _child_witness_paths(
    data: dict[str, Any],
    rollup_entry: dict[str, Any] | None,
) -> list[str]:
    paths: list[str] = []
    if rollup_entry:
        paths.extend(str(p) for p in rollup_entry.get("mandatory_children") or [])
        for dot in rollup_entry.get("child_extract_paths") or []:
            val = _get_dot(data, dot)
            if isinstance(val, str) and val.strip():
                paths.append(val.strip())
    return paths


def _proceed_ship_pass(data: dict[str, Any]) -> bool:
    gates = data.get("gates") or {}
    proceed = gates.get("g4_8_proceed_ship") or data.get("g4_8_proceed_ship") or data.get("proceed_ship")
    return proceed in ("pass", True, "PASS")


def _keyframe_for_batch(root: Path, batch_id: str) -> Path | None:
    if not batch_id:
        return None
    pilot = batch_id.removeprefix("tile_").removesuffix("_production_v1")
    candidates = [
        root / f"debug_runs/art_pipeline/{pilot}_production_keyframe_g4_live.json",
        root / f"debug_runs/art_pipeline/{pilot}_keyframe_g4_live.json",
    ]
    for path in candidates:
        if path.is_file():
            return path
    art = root / "debug_runs/art_pipeline"
    if art.is_dir():
        for path in sorted(art.glob(f"*{pilot}*keyframe*g4*_live.json")):
            return path
    return None


def evaluate_witness_honesty_rules(
    data: dict[str, Any],
    *,
    witness_rel: str,
    catalog: dict[str, Any],
    root: Path,
    include_rollup_children: bool = True,
    _child_depth: int = 0,
) -> list[ValidationIssue]:
    issues: list[ValidationIssue] = []
    meta = data.get("_agent_meta") if isinstance(data.get("_agent_meta"), dict) else {}
    rule_by_id = {str(r.get("rule_id")): r for r in catalog.get("rules") or []}

    def add(rule_id: str, hint: str, *, severity: str | None = None) -> None:
        rule = rule_by_id.get(rule_id) or {}
        issues.append(
            ValidationIssue(
                kind=rule_id,
                severity=severity or str(rule.get("severity") or "error"),  # type: ignore[arg-type]
                file=witness_rel,
                symbol=rule_id,
                field=rule_id,
                hint=hint,
                signature=rule_id,
            )
        )

    # WIT-GREEN-TINT-ZERO
    if _path_matches(witness_rel, "**/landscape_grammar_lg4*_live.json"):
        if data.get("green") is True and int(data.get("topology_tint_visible_chunks") or 0) == 0:
            add(
                "WIT-GREEN-TINT-ZERO",
                "green=true but topology_tint_visible_chunks==0",
            )

    # WIT-OPERATOR-LIB-FIXTURE
    if data.get("operator_visible") is True:
        proof_grade = data.get("proof_grade")
        if proof_grade == "lib_fixture" or _agent_commands_all_lib(meta):
            add(
                "WIT-OPERATOR-LIB-FIXTURE",
                f"operator_visible=true with proof_grade={proof_grade!r} or --lib-only harness",
            )

    # WIT-ART-DISHONEST
    art_quality = str(data.get("art_quality") or "")
    if data.get("green") is True and art_quality.startswith("rejected"):
        add("WIT-ART-DISHONEST", f"green=true but art_quality={art_quality!r}")

    # WIT-TINY-PNG-PILOT
    png_count = int(data.get("png_count") or 0)
    ship = data.get("ship")
    if png_count >= 1 and ship is not False:
        for row in data.get("png_dimensions") or []:
            if not isinstance(row, dict):
                continue
            if int(row.get("bytes") or 0) < 512:
                add(
                    "WIT-TINY-PNG-PILOT",
                    f"png {row.get('file')} bytes={row.get('bytes')} < 512 with ship!=false",
                )
                break

    # WIT-GATE-DRIFT-G4
    gates = data.get("gates") if isinstance(data.get("gates"), dict) else {}
    if gates.get("G4") == "planned":
        batch_id = str(data.get("batch_id") or "")
        keyframe = _keyframe_for_batch(root, batch_id)
        if keyframe and keyframe.is_file():
            kdata = json.loads(keyframe.read_text(encoding="utf-8"))
            if _proceed_ship_pass(kdata):
                add(
                    "WIT-GATE-DRIFT-G4",
                    f"gates.G4=planned but {_rel(keyframe, root)} has proceed_ship pass",
                )

    # WIT-ENV-BOOTSTRAP-ONLY
    live_paths = {str(p).replace("\\", "/") for p in catalog.get("live_sim_required_paths") or []}
    if (
        witness_rel in live_paths
        and data.get("green") is True
        and data.get("live_sim_required") is True
        and _agent_commands_all_lib(meta)
    ):
        add(
            "WIT-ENV-BOOTSTRAP-ONLY",
            "live_sim_required witness green from --lib-only agent_commands",
            severity="warning",
        )

    # WIT-MISSING-ENVELOPE
    if witness_rel.endswith("_live.json") and not str(meta.get("schema") or "").strip():
        add(
            "WIT-MISSING-ENVELOPE",
            "missing _agent_meta.schema on *_live.json witness",
            severity="warning",
        )

    # WIT-EXIT-PREDICATE
    exit_fail = _exit_predicate_fails(data)
    if exit_fail:
        add("WIT-EXIT-PREDICATE", exit_fail)

    rollup_entry = _rollup_entry_for(witness_rel, catalog)
    green_fields = list(rollup_entry.get("green_fields") or ["green", "all_green"]) if rollup_entry else ["green", "all_green"]
    parent_green = _is_green(data, green_fields)

    # WIT-PHASE-CLOSE-WITHOUT-SUB
    phase_keys = [k for k in data if re.fullmatch(r"phase_[a-z]_green", str(k))]
    if phase_keys and all(data.get(k) is True for k in phase_keys) and parent_green:
        child_paths = _child_witness_paths(data, rollup_entry)
        for child_rel in child_paths:
            child_path = root / child_rel
            if not child_path.is_file():
                add("WIT-PHASE-CLOSE-WITHOUT-SUB", f"mandatory child missing: {child_rel}")
                continue
            child_data = json.loads(child_path.read_text(encoding="utf-8"))
            child_issues = evaluate_witness_honesty_rules(
                child_data,
                witness_rel=child_rel.replace("\\", "/"),
                catalog=catalog,
                root=root,
                include_rollup_children=False,
                _child_depth=_child_depth + 1,
            )
            child_errors = [i for i in child_issues if i.severity == "error"]
            if child_errors:
                add(
                    "WIT-PHASE-CLOSE-WITHOUT-SUB",
                    f"phase close green but child {child_rel} fails: {child_errors[0].hint}",
                )

    # WIT-ROLLUP-CHILD-ONLY
    if include_rollup_children and rollup_entry and parent_green and _child_depth == 0:
        for child_rel in _child_witness_paths(data, rollup_entry):
            child_rel = child_rel.replace("\\", "/")
            child_path = root / child_rel
            if not child_path.is_file():
                add("WIT-ROLLUP-CHILD-ONLY", f"rollup child missing: {child_rel}")
                continue
            child_data = json.loads(child_path.read_text(encoding="utf-8"))
            child_issues = evaluate_witness_honesty_rules(
                child_data,
                witness_rel=child_rel,
                catalog=catalog,
                root=root,
                include_rollup_children=False,
                _child_depth=_child_depth + 1,
            )
            child_errors = [i for i in child_issues if i.severity == "error"]
            if child_errors:
                add(
                    "WIT-ROLLUP-CHILD-ONLY",
                    f"parent green but child {child_rel} fails: {child_errors[0].hint}",
                )

    return issues


def evaluate_queue_honesty_rules(
    catalog: dict[str, Any],
    *,
    root: Path,
) -> list[ValidationIssue]:
    from .queue_integrity import collect_queue_integrity

    body = collect_queue_integrity(repo=root)
    return list(body.get("issues") or [])


def validate_witness_honesty(
    data: dict[str, Any],
    *,
    witness_rel: str,
    catalog: dict[str, Any] | None = None,
    root: Path | None = None,
    compression_level: int = 3,
) -> ValidationReport:
    root = root or repo_root()
    catalog = catalog or load_witness_integrity_catalog(repo=root)
    issues = evaluate_witness_honesty_rules(
        data,
        witness_rel=witness_rel.replace("\\", "/"),
        catalog=catalog,
        root=root,
    )
    errors = [i for i in issues if i.severity == "error"]
    status: str = "failed" if errors else ("warning" if issues else "passed")
    fixes = [
        KnownFix(signature=rid, fix=str(rule.get("fix_hint") or ""), confidence=0.9)
        for rid, rule in {str(r.get("rule_id")): r for r in catalog.get("rules") or []}.items()
        if any(i.signature == rid for i in issues) and rule.get("fix_hint")
    ]
    return ValidationReport(
        validator="test",
        status=status,  # type: ignore[arg-type]
        compression_level=compression_level,
        summary=f"witness_honesty {witness_rel}: {len(errors)} error(s), {len(issues)} issue(s)",
        error_count=len(errors),
        warning_count=len(issues) - len(errors),
        errors=issues,
        known_fixes=fixes[:6],
        confidence=0.95 if not errors else 0.8,
    ).compress(compression_level)


def validate_witness_honesty_path(
    path: str | Path,
    *,
    compression_level: int = 3,
    include_queue_rules: bool = False,
) -> ValidationReport:
    root = repo_root()
    resolved = _resolve(path)
    if not resolved.is_file():
        return ValidationReport(
            validator="test",
            status="failed",
            compression_level=compression_level,
            summary="witness file not found",
            error_count=1,
            errors=[
                ValidationIssue(
                    kind="MissingFile",
                    severity="error",
                    file=_rel(resolved, root),
                    hint="Provide a *_live.json witness path",
                    signature="witness_missing",
                )
            ],
        )
    catalog = load_witness_integrity_catalog(repo=root)
    data = json.loads(resolved.read_text(encoding="utf-8"))
    rel = _rel(resolved, root)
    report = validate_witness_honesty(data, witness_rel=rel, catalog=catalog, root=root, compression_level=compression_level)
    if include_queue_rules:
        queue_issues = evaluate_queue_honesty_rules(catalog, root=root)
        if queue_issues:
            merged = list(report.errors) + queue_issues
            errors = [i for i in merged if i.severity == "error"]
            report = ValidationReport(
                validator="test",
                status="failed" if errors else report.status,
                compression_level=compression_level,
                summary=f"{report.summary}; queue_rules={len(queue_issues)}",
                error_count=len(errors),
                warning_count=len(merged) - len(errors),
                errors=merged,
                known_fixes=report.known_fixes,
                confidence=report.confidence,
            ).compress(compression_level)
    return report


def validate_witness_honesty_scan(
    scan_dir: str | Path = "debug_runs",
    *,
    compression_level: int = 3,
    include_queue_rules: bool = True,
) -> ValidationReport:
    root = repo_root()
    catalog = load_witness_integrity_catalog(repo=root)
    base = _resolve(scan_dir)
    issues: list[ValidationIssue] = []
    scanned = 0
    failed_files = 0

    if base.is_dir():
        for path in sorted(base.rglob("*_live.json")):
            scanned += 1
            rel = _rel(path, root)
            try:
                data = json.loads(path.read_text(encoding="utf-8"))
            except (OSError, json.JSONDecodeError) as exc:
                issues.append(
                    ValidationIssue(
                        kind="UnreadableWitness",
                        severity="error",
                        file=rel,
                        hint=str(exc),
                        signature="witness_unreadable",
                    )
                )
                failed_files += 1
                continue
            file_report = validate_witness_honesty(
                data,
                witness_rel=rel,
                catalog=catalog,
                root=root,
                compression_level=compression_level,
            )
            if file_report.status != "passed":
                failed_files += 1
                for issue in file_report.errors:
                    issues.append(issue)

    if include_queue_rules:
        issues.extend(evaluate_queue_honesty_rules(catalog, root=root))

    errors = [i for i in issues if i.severity == "error"]
    status: str = "failed" if errors else ("warning" if issues else "passed")
    return ValidationReport(
        validator="test",
        status=status,  # type: ignore[arg-type]
        compression_level=compression_level,
        summary=f"witness_honesty scan { _rel(base, root) }: scanned={scanned} failed_files={failed_files} issues={len(issues)}",
        error_count=len(errors),
        warning_count=len(issues) - len(errors),
        errors=issues,
        confidence=0.9 if not errors else 0.75,
    ).compress(compression_level)


def _fixture_pairs(root: Path) -> list[tuple[str, str, bool]]:
    """Return (rel_path, rule_id, expect_pass) tuples for self-test."""
    fixtures = root / FIXTURES_REL
    pairs: list[tuple[str, str, bool]] = []
    if not fixtures.is_dir():
        return pairs
    for path in sorted(fixtures.glob("*.json")):
        if path.name.startswith("_queue_"):
            continue
        try:
            doc = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            continue
        meta = doc.get("_fixture") if isinstance(doc.get("_fixture"), dict) else {}
        expect = str(meta.get("expect") or "pass")
        rule_id = str(meta.get("rule_id") or "")
        pairs.append((_rel(path, root), rule_id, expect == "pass"))
    return pairs


def refresh_mcp_witness_honesty_validator_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    validate_witness_integrity_catalog(repo=root)
    fixture_results: list[dict[str, Any]] = []
    fixtures_ok = True
    for rel, rule_id, expect_pass in _fixture_pairs(root):
        report = validate_witness_honesty_path(root / rel, compression_level=3)
        ok = (report.status == "passed") if expect_pass else (report.status != "passed")
        fixture_results.append(
            {
                "fixture": rel,
                "rule_id": rule_id,
                "expect_pass": expect_pass,
                "ok": ok,
                "status": report.status,
                "summary": report.summary,
            }
        )
        fixtures_ok = fixtures_ok and ok

    scan_report = validate_witness_honesty_scan("debug_runs", compression_level=3)
    green = fixtures_ok and len(fixture_results) >= 6
    body: dict[str, Any] = {
        "gate": "MCP-WIT-006",
        "green": green,
        "verdict": "PASS" if green else "FAIL",
        "catalog": CATALOG_REL,
        "fixture_results": fixture_results,
        "scan": {
            "status": scan_report.status,
            "summary": scan_report.summary,
            "error_count": scan_report.error_count,
            "warning_count": scan_report.warning_count,
        },
        "commands": [
            "python -m rust_engine_mcp.cli validate-report witness_honesty <path> --compress 3",
            "python -m rust_engine_mcp.cli validate-report witness_honesty --scan debug_runs --compress 3",
        ],
        "_agent_meta": {
            "schema": "mcp_witness_honesty_validator_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "MCP_WITNESS_HONESTY_VALIDATOR",
            "source_system": "witness_honesty",
            "relative_path": WITNESS_REL,
            "ritual": "BLANG:WIT-HON MCP-WIT-006" if green else None,
        },
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = WITNESS_REL
    return body
