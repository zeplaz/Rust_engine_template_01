"""MCP-GUARD-002…004 — teachable audit, archetype ratio, warehouse track guards."""

from __future__ import annotations

import fnmatch
import json
import re
from collections import Counter
from pathlib import Path
from typing import Any

from rust_engine_mcp.grammar_build_set import building_set_coverage_report, load_building_sets, load_pilot_catalog
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators.report import ValidationIssue, ValidationReport

CONFIG_REL = "tools/mcp/guard/build_set_guards_v1.json"
TEACHABLE_WITNESS = "debug_runs/example_teachable_audit_live.json"
RATIO_WITNESS = "debug_runs/single_archetype_ratio_guard_live.json"
WAREHOUSE_WITNESS = "debug_runs/warehouse_track_guard_live.json"
MAX_VIOLATIONS = 40


def _load_config(path: Path | None = None) -> dict[str, Any]:
    cfg_path = path or (repo_root() / CONFIG_REL)
    return json.loads(cfg_path.read_text(encoding="utf-8"))


def _norm_rel(path: Path, root: Path) -> str:
    try:
        return path.relative_to(root).as_posix()
    except ValueError:
        return path.as_posix()


def _glob_match(rel: str, patterns: list[str]) -> bool:
    rel = rel.replace("\\", "/")
    for pattern in patterns:
        if fnmatch.fnmatch(rel, pattern):
            return True
        if fnmatch.fnmatch(Path(rel).name, pattern):
            return True
    return False


def _subject_files(root: Path, cfg: dict[str, Any]) -> list[Path]:
    teachable = cfg.get("teachable") or {}
    examples_root = root / str(teachable.get("examples_root") or "tools/mcp/schemas/examples")
    subject_globs: list[str] = list(teachable.get("subject_globs") or [])
    exempt_globs: list[str] = list(teachable.get("exempt_globs") or [])
    canonical_globs: list[str] = list(teachable.get("canonical_fixture_globs") or [])
    out: list[Path] = []
    if not examples_root.is_dir():
        return out
    for path in sorted(examples_root.rglob("*.json")):
        rel = _norm_rel(path, root)
        if _glob_match(rel, exempt_globs):
            continue
        if _glob_match(rel, canonical_globs):
            continue
        if not _glob_match(rel, subject_globs):
            continue
        out.append(path)
    return out


def _read_teaches(path: Path) -> list[str]:
    try:
        body = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return []
    meta = body.get("_meta") if isinstance(body, dict) else None
    if not isinstance(meta, dict):
        return []
    teaches = meta.get("teaches")
    if not isinstance(teaches, list):
        return []
    return [str(x) for x in teaches if x]


def example_teachable_audit(*, config_path: Path | None = None) -> dict[str, Any]:
    """MCP-GUARD-002 — schema examples must declare _meta.teaches with ≥2 axes."""
    root = repo_root()
    cfg = _load_config(config_path)
    teachable = cfg.get("teachable") or {}
    min_axes = int(cfg.get("min_teaches_axes") or 2)
    valid_axes = {str(x) for x in (cfg.get("valid_teaches_axes") or [])}
    violations: list[dict[str, Any]] = []
    checked: list[str] = []

    for path in _subject_files(root, cfg):
        rel = _norm_rel(path, root)
        checked.append(rel)
        teaches = _read_teaches(path)
        if len(teaches) < min_axes:
            violations.append(
                {
                    "file": rel,
                    "teaches_count": len(teaches),
                    "hint": f"missing _meta.teaches with >={min_axes} axes",
                }
            )
            continue
        unknown = [t for t in teaches if t not in valid_axes]
        if unknown:
            violations.append(
                {
                    "file": rel,
                    "unknown_axes": unknown,
                    "hint": "teaches axes must be from build_set_guards_v1 valid_teaches_axes",
                }
            )

    green = len(violations) == 0
    truncated = len(violations) > MAX_VIOLATIONS
    if truncated:
        violations = violations[:MAX_VIOLATIONS]

    return {
        "task_id": "MCP-GUARD-002",
        "gate_id": "example_teachable_audit",
        "ok": green,
        "green": green,
        "checked_files": len(checked),
        "checked": checked,
        "min_teaches_axes": min_axes,
        "violation_count": len(violations) if not truncated else "truncated",
        "violations": violations,
        "violations_truncated": truncated,
        "config": CONFIG_REL,
        "examples_root": teachable.get("examples_root"),
    }


def _count_archetypes(text: str, patterns: list[str]) -> Counter[str]:
    counts: Counter[str] = Counter()
    for pattern in patterns:
        for match in re.finditer(pattern, text):
            archetype = match.group(1)
            if not archetype or archetype.startswith("module_"):
                continue
            if not archetype[0].isupper():
                continue
            counts[archetype] += 1
    return counts


def single_archetype_ratio_guard(*, config_path: Path | None = None) -> dict[str, Any]:
    """MCP-GUARD-003 — fail when one archetype dominates refs without set insurance."""
    root = repo_root()
    cfg = _load_config(config_path)
    ratio_cfg = cfg.get("archetype_ratio") or {}
    scan_roots: list[str] = list(ratio_cfg.get("scan_roots") or [])
    extensions = {e.lower() for e in (ratio_cfg.get("scan_extensions") or [".json", ".py"])}
    patterns: list[str] = list(ratio_cfg.get("archetype_patterns") or [])
    max_ratio = float(ratio_cfg.get("max_ratio") or 0.4)
    min_pilots = int(ratio_cfg.get("building_set_exception_min_pilots") or 2)

    counts: Counter[str] = Counter()
    scanned = 0
    for scan_root in scan_roots:
        base = root / scan_root
        if not base.is_dir():
            continue
        for path in base.rglob("*"):
            if not path.is_file() or path.suffix.lower() not in extensions:
                continue
            scanned += 1
            try:
                text = path.read_text(encoding="utf-8")
            except OSError:
                continue
            counts.update(_count_archetypes(text, patterns))

    total = sum(counts.values())
    ratios = {k: (v / total if total else 0.0) for k, v in counts.items()}
    coverage = building_set_coverage_report()
    grammar_pilot_count = sum(int(r.get("grammar_pilots") or 0) for r in (coverage.get("rows") or []))
    set_insured = bool(coverage.get("green")) and grammar_pilot_count >= min_pilots

    violations: list[dict[str, Any]] = []
    if total > 0 and not set_insured:
        for archetype, ratio in sorted(ratios.items(), key=lambda kv: kv[1], reverse=True):
            if ratio > max_ratio:
                violations.append(
                    {
                        "archetype": archetype,
                        "count": counts[archetype],
                        "ratio": round(ratio, 4),
                        "max_ratio": max_ratio,
                        "hint": "diversify grammar pilots/examples or insure building_set manifest",
                    }
                )

    green = len(violations) == 0
    return {
        "task_id": "MCP-GUARD-003",
        "gate_id": "single_archetype_ratio_guard",
        "ok": green,
        "green": green,
        "scanned_files": scanned,
        "total_refs": total,
        "archetype_counts": dict(counts),
        "archetype_ratios": {k: round(v, 4) for k, v in ratios.items()},
        "max_ratio": max_ratio,
        "building_set_insured": set_insured,
        "grammar_pilot_count": grammar_pilot_count,
        "violation_count": len(violations),
        "violations": violations,
        "config": CONFIG_REL,
    }


def _manifest_track_paths(root: Path) -> set[str]:
    paths: set[str] = set()
    for pilot in load_pilot_catalog():
        for key in ("site_json_path",):
            rel = pilot.get(key)
            if rel:
                paths.add(f"assets/configs/buildings/{rel}".replace("\\", "/"))
        preset = pilot.get("arch_dna_preset")
        if preset:
            paths.add(f"tools/mcp/schemas/examples/arch_dna_{preset}.json")
    for row in load_building_sets():
        for batch in row.get("tile_batches_optional") or []:
            paths.add(str(batch).replace("\\", "/"))
    return paths


def warehouse_track_guard(*, config_path: Path | None = None) -> dict[str, Any]:
    """MCP-GUARD-004 — new warehouse paths need manifest row or teaches grammar_eval/pilot_catalog."""
    root = repo_root()
    cfg = _load_config(config_path)
    track_cfg = cfg.get("warehouse_track") or {}
    scan_roots: list[str] = list(track_cfg.get("scan_roots") or [])
    needle = str(track_cfg.get("path_needle") or "warehouse").lower()
    required_any = {str(x) for x in (track_cfg.get("required_teaches_any") or [])}
    exempt_globs: list[str] = list(track_cfg.get("exempt_globs") or [])
    canonical_globs: list[str] = list(track_cfg.get("canonical_track_globs") or [])
    allowlist_globs: list[str] = list(track_cfg.get("allowlist_globs") or [])
    manifest_paths = _manifest_track_paths(root)

    violations: list[dict[str, Any]] = []
    scanned = 0
    for scan_root in scan_roots:
        base = root / scan_root
        if not base.is_dir():
            continue
        for path in base.rglob("*"):
            if not path.is_file():
                continue
            if path.suffix.lower() != ".json":
                continue
            rel = _norm_rel(path, root)
            if needle not in rel.lower():
                continue
            scanned += 1
            if _glob_match(rel, exempt_globs):
                continue
            if _glob_match(rel, canonical_globs) or _glob_match(rel, allowlist_globs):
                continue
            if rel in manifest_paths:
                continue
            teaches = _read_teaches(path)
            if required_any.intersection(teaches):
                continue
            violations.append(
                {
                    "file": rel,
                    "hint": "warehouse track needs building_set manifest row or _meta.teaches including grammar_eval/pilot_catalog",
                }
            )

    green = len(violations) == 0
    truncated = len(violations) > MAX_VIOLATIONS
    if truncated:
        violations = violations[:MAX_VIOLATIONS]

    return {
        "task_id": "MCP-GUARD-004",
        "gate_id": "warehouse_track_guard",
        "ok": green,
        "green": green,
        "scanned_files": scanned,
        "manifest_path_count": len(manifest_paths),
        "violation_count": len(violations) if not truncated else "truncated",
        "violations": violations,
        "violations_truncated": truncated,
        "config": CONFIG_REL,
    }


def _guard_validation_report(
    *,
    validator: str,
    body: dict[str, Any],
    kind: str,
    signature: str,
) -> ValidationReport:
    status = "passed" if body.get("green") else "failed"
    errors = [
        ValidationIssue(
            kind=kind,
            severity="error",
            file=str(v.get("file") or v.get("archetype") or ""),
            hint=str(v.get("hint") or ""),
            signature=signature,
        )
        for v in body.get("violations") or []
    ]
    return ValidationReport(
        validator=validator,
        status=status,
        compression_level=3,
        summary=f"{body.get('gate_id')}: {len(errors)} violation(s)",
        error_count=len(errors),
        errors=errors[:8],
        confidence=0.95 if body.get("green") else 0.9,
    )


def validate_example_teachable_audit(*, config_path: Path | None = None) -> ValidationReport:
    body = example_teachable_audit(config_path=config_path)
    return _guard_validation_report(
        validator="example_teachable_audit",
        body=body,
        kind="TeachableAudit",
        signature="example_teachable_audit_missing_meta",
    )


def validate_single_archetype_ratio_guard(*, config_path: Path | None = None) -> ValidationReport:
    body = single_archetype_ratio_guard(config_path=config_path)
    return _guard_validation_report(
        validator="single_archetype_ratio_guard",
        body=body,
        kind="ArchetypeRatio",
        signature="single_archetype_ratio_exceeded",
    )


def validate_warehouse_track_guard(*, config_path: Path | None = None) -> ValidationReport:
    body = warehouse_track_guard(config_path=config_path)
    return _guard_validation_report(
        validator="warehouse_track_guard",
        body=body,
        kind="WarehouseTrack",
        signature="warehouse_track_unmanifested",
    )


def write_example_teachable_audit_witness(*, config_path: Path | None = None) -> dict[str, Any]:
    body = example_teachable_audit(config_path=config_path)
    out = repo_root() / TEACHABLE_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = str(out.relative_to(repo_root())).replace("\\", "/")
    return body


def write_single_archetype_ratio_guard_witness(*, config_path: Path | None = None) -> dict[str, Any]:
    body = single_archetype_ratio_guard(config_path=config_path)
    out = repo_root() / RATIO_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = str(out.relative_to(repo_root())).replace("\\", "/")
    return body


def write_warehouse_track_guard_witness(*, config_path: Path | None = None) -> dict[str, Any]:
    body = warehouse_track_guard(config_path=config_path)
    out = repo_root() / WAREHOUSE_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = str(out.relative_to(repo_root())).replace("\\", "/")
    return body


def write_build_set_guards_witnesses(*, config_path: Path | None = None) -> dict[str, Any]:
    return {
        "teachable": write_example_teachable_audit_witness(config_path=config_path),
        "ratio": write_single_archetype_ratio_guard_witness(config_path=config_path),
        "warehouse": write_warehouse_track_guard_witness(config_path=config_path),
        "green": all(
            x.get("green")
            for x in (
                example_teachable_audit(config_path=config_path),
                single_archetype_ratio_guard(config_path=config_path),
                warehouse_track_guard(config_path=config_path),
            )
        ),
    }
