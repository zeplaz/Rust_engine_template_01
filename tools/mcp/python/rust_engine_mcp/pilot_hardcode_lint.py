"""MCP-GUARD-001 — scan for warehouse-shaped hardcode outside allowlists."""

from __future__ import annotations

import fnmatch
import json
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators.report import ValidationIssue, ValidationReport

ALLOWLIST_REL = "tools/mcp/guard/pilot_hardcode_allowlist_v1.json"
WITNESS_PATH = "debug_runs/pilot_hardcode_lint_live.json"
MAX_VIOLATIONS = 50


def _load_allowlist(path: Path | None = None) -> dict[str, Any]:
    cfg_path = path or (repo_root() / ALLOWLIST_REL)
    return json.loads(cfg_path.read_text(encoding="utf-8"))


def _norm_rel(path: Path, root: Path) -> str:
    try:
        rel = path.relative_to(root)
    except ValueError:
        return path.as_posix()
    return rel.as_posix()


def _path_allowed(rel: str, *, globs: list[str], exact: list[str]) -> bool:
    rel = rel.replace("\\", "/")
    if rel in exact:
        return True
    for pattern in globs:
        if fnmatch.fnmatch(rel, pattern):
            return True
    return False


def _scan_file(
    path: Path,
    root: Path,
    needles: list[str],
    extensions: set[str],
    permanent_globs: list[str],
    transitional: list[str],
) -> tuple[list[dict[str, Any]], bool, bool]:
    rel = _norm_rel(path, root)
    if path.suffix.lower() not in extensions:
        return [], False, False
    if _path_allowed(rel, globs=permanent_globs, exact=[]):
        return [], True, False
    transitional_hit = _path_allowed(rel, globs=[], exact=transitional)
    try:
        text = path.read_text(encoding="utf-8")
    except OSError:
        return [], False, transitional_hit
    violations: list[dict[str, Any]] = []
    for needle in needles:
        if needle not in text:
            continue
        if transitional_hit:
            continue
        line_no = 0
        for idx, line in enumerate(text.splitlines(), start=1):
            if needle in line:
                line_no = idx
                break
        violations.append(
            {
                "file": rel,
                "line": line_no,
                "needle": needle,
                "hint": f"hardcoded pilot needle `{needle}` — use catalog/manifest loaders",
            }
        )
    return violations, False, transitional_hit


def pilot_hardcode_lint(*, allowlist_path: Path | None = None) -> dict[str, Any]:
    root = repo_root()
    cfg = _load_allowlist(allowlist_path)
    needles: list[str] = list(cfg.get("needles") or [])
    scan_roots: list[str] = list(cfg.get("scan_roots") or ["src"])
    extensions = {e.lower() for e in (cfg.get("scan_extensions") or [".rs"])}
    permanent_globs: list[str] = list(cfg.get("permanent_allowlist_globs") or [])
    transitional: list[str] = list(cfg.get("transitional_allowlist") or [])

    violations: list[dict[str, Any]] = []
    transitional_files: set[str] = set()
    scanned = 0
    hit_count = 0
    permanent_hits = 0
    transitional_hits = 0

    for scan_root in scan_roots:
        base = root / scan_root
        if not base.is_dir():
            continue
        for path in base.rglob("*"):
            if not path.is_file():
                continue
            scanned += 1
            file_violations, permanent, transitional_hit = _scan_file(
                path,
                root,
                needles,
                extensions,
                permanent_globs,
                transitional,
            )
            if permanent:
                try:
                    text = path.read_text(encoding="utf-8")
                    if any(n in text for n in needles):
                        permanent_hits += 1
                except OSError:
                    pass
            if transitional_hit:
                rel = _norm_rel(path, root)
                try:
                    text = path.read_text(encoding="utf-8")
                    if any(n in text for n in needles):
                        transitional_hits += 1
                        transitional_files.add(rel)
                except OSError:
                    pass
            if file_violations:
                hit_count += len(file_violations)
                violations.extend(file_violations)

    violation_count = len(violations)
    green = violation_count == 0
    truncated = False
    if len(violations) > MAX_VIOLATIONS:
        violations = violations[:MAX_VIOLATIONS]
        truncated = True

    return {
        "task_id": cfg.get("task_id", "MCP-GUARD-001"),
        "gate_id": "pilot_hardcode_lint",
        "ok": green,
        "green": green,
        "scanned_files": scanned,
        "needle_count": len(needles),
        "hit_count": hit_count,
        "violation_count": violation_count,
        "transitional_hit_count": transitional_hits,
        "permanent_allowlist_hit_count": permanent_hits,
        "transitional_until": cfg.get("transitional_until", ""),
        "allowlist": ALLOWLIST_REL,
        "violations": violations,
        "transitional_files": sorted(transitional_files),
        "violations_truncated": truncated,
    }


def validate_pilot_hardcode_lint(*, allowlist_path: Path | None = None) -> ValidationReport:
    body = pilot_hardcode_lint(allowlist_path=allowlist_path)
    status = "passed" if body["green"] else "failed"
    errors = [
        ValidationIssue(
            kind="PilotHardcode",
            severity="error",
            file=v.get("file", ""),
            line=int(v.get("line") or 0),
            hint=v.get("hint", ""),
            signature=f"pilot_hardcode:{v.get('needle', '')}",
        )
        for v in body.get("violations", [])
    ]
    return ValidationReport(
        validator="test",
        status=status,
        compression_level=3,
        summary=(
            f"pilot_hardcode_lint: {body['violation_count']} violation(s), "
            f"{body['transitional_hit_count']} transitional hit(s)"
        ),
        error_count=len(errors),
        errors=errors[:8],
        confidence=0.95 if body["green"] else 0.9,
    )


def write_pilot_hardcode_lint_witness(*, allowlist_path: Path | None = None) -> dict[str, Any]:
    body = pilot_hardcode_lint(allowlist_path=allowlist_path)
    out = repo_root() / WITNESS_PATH
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = str(out.relative_to(repo_root())).replace("\\", "/")
    return body
