"""APSR-D3 — inline-feedback adoption sweep (status atom vs bare writes)."""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

SUITE_REL = "tools/mcp/art_pipeline_suite"

STATUS_ATOM_PANELS = (
    "variants_panel.py",
    "catalog.py",
    "material_library_widget.py",
    "grammar_iterate_panel.py",
    "atlas_panel.py",
    "assembly_qc_strip.py",
    "catalog_kit_coverage_strip.py",
    "golden_seed_review_panel.py",
)

ATOM_HELPERS = ("apply_status_atom", "set_inline_status", "apply_material_card_status")

RAW_STATUS_FG = re.compile(
    r"""(?:foreground|fg)\s*=\s*[^,)\n]*\bCOLOR_(?:PASS|FAIL|WARN)\b"""
)

BARE_STATUS_SET = re.compile(
    r"""\.(?:set|configure)\([^)]*(?:PASS:|FAIL:|✓|✗)"""
)


@dataclass(frozen=True)
class FeedbackViolation:
    file: str
    line: int
    detail: str


def scan_inline_feedback_violations(*, repo: Path | None = None) -> list[FeedbackViolation]:
    root = (repo or repo_root()) / SUITE_REL
    out: list[FeedbackViolation] = []
    for name in STATUS_ATOM_PANELS:
        path = root / name
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8")
        if not any(h in text for h in ATOM_HELPERS):
            out.append(FeedbackViolation(name, 0, "status panel missing atom helper import"))
        for i, line in enumerate(text.splitlines(), start=1):
            if RAW_STATUS_FG.search(line):
                out.append(FeedbackViolation(name, i, "raw status color on widget"))
    for path in sorted(root.glob("*_panel.py")):
        text = path.read_text(encoding="utf-8")
        if any(h in text for h in ATOM_HELPERS):
            continue
        for i, line in enumerate(text.splitlines(), start=1):
            if BARE_STATUS_SET.search(line) and "aps_inline_feedback" not in line:
                out.append(FeedbackViolation(path.name, i, "bare status string write"))
    return out


def inline_feedback_audit(*, repo: Path | None = None) -> dict[str, Any]:
    violations = scan_inline_feedback_violations(repo=repo)
    return {
        "violation_count": len(violations),
        "violations": [{"file": v.file, "line": v.line, "detail": v.detail} for v in violations[:30]],
        "green": not violations,
    }


def write_apsr_d3_witness(*, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    audit = inline_feedback_audit(repo=repo)
    body: dict[str, Any] = {
        "task_id": "APSR-A3-D3-001",
        "gate": "APSR-A3-D3-001",
        "green": audit["green"],
        "violation_count": audit["violation_count"],
        "plan_ref": "src/dev/plan_aps_refactor_v1.md#APSR-D3",
    }
    return write_aps_live_witness(
        body,
        "debug_runs/apsr_a3_d3_001_live.json",
        schema="apsr_a3_d3_live_v1",
        profile="APSR_A3_D3",
        source_system="apsr_a3_d3",
        ritual="BLANG:WIT-HON APSR-A3-D3-001" if audit["green"] else None,
        repo=repo,
    )
