"""APSR-D2 — tooltip coverage assertion for APS interactive widgets."""

from __future__ import annotations

import json
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

SUITE_REL = "tools/mcp/art_pipeline_suite"
ALLOWLIST_REL = "tools/mcp/schemas/aps_tooltip_coverage_allowlist_v1.json"

INTERACTIVE_RE = re.compile(
    r"\bttk\.(?:Button|Checkbutton|Radiobutton|Combobox)\s*\("
    r"|\btk\.(?:Button|Checkbutton|Radiobutton)\s*\("
)

SKIP_FILES = frozenset(
    {
        "aps_tooltips.py",
        "scrollable.py",
        "aps_tk.py",
        "aps_collapsible.py",
    }
)


@dataclass(frozen=True)
class TooltipGap:
    file: str
    line: int
    detail: str


def _load_allowlist(*, repo: Path | None = None) -> set[str]:
    root = repo or repo_root()
    path = root / ALLOWLIST_REL
    if not path.is_file():
        return set()
    data = json.loads(path.read_text(encoding="utf-8"))
    return {str(x) for x in data.get("exempt_sites") or []}


def scan_tooltip_gaps(*, repo: Path | None = None) -> list[TooltipGap]:
    root = repo or repo_root()
    suite = root / SUITE_REL
    allow = _load_allowlist(repo=root)
    gaps: list[TooltipGap] = []
    for path in sorted(suite.glob("*.py")):
        if path.name in SKIP_FILES:
            continue
        text = path.read_text(encoding="utf-8")
        has_bind = "bind_aps_tooltip" in text or "bind_many" in text
        for i, line in enumerate(text.splitlines(), start=1):
            if not INTERACTIVE_RE.search(line):
                continue
            site = f"{path.name}:{i}"
            if site in allow:
                continue
            if not has_bind:
                gaps.append(TooltipGap(path.name, i, "file creates interactive widgets but never binds tooltips"))
    return gaps


def tooltip_coverage_audit(*, repo: Path | None = None) -> dict[str, Any]:
    gaps = scan_tooltip_gaps(repo=repo)
    suite = (repo or repo_root()) / SUITE_REL
    bind_count = sum(text.count("bind_aps_tooltip(") for text in (p.read_text(encoding="utf-8") for p in suite.glob("*.py")))
    return {
        "bind_aps_tooltip_calls": bind_count,
        "gap_count": len(gaps),
        "gaps": [{"file": g.file, "line": g.line, "detail": g.detail} for g in gaps[:30]],
        "green": not gaps,
    }


def write_apsr_d2_witness(*, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    audit = tooltip_coverage_audit(repo=repo)
    body: dict[str, Any] = {
        "task_id": "APSR-A3-D2-001",
        "gate": "APSR-A3-D2-001",
        "green": audit["green"],
        "bind_aps_tooltip_calls": audit["bind_aps_tooltip_calls"],
        "gap_count": audit["gap_count"],
        "plan_ref": "src/dev/plan_aps_refactor_v1.md#APSR-D2",
    }
    return write_aps_live_witness(
        body,
        "debug_runs/apsr_a3_d2_001_live.json",
        schema="apsr_a3_d2_live_v1",
        profile="APSR_A3_D2",
        source_system="apsr_a3_d2",
        ritual="BLANG:WIT-HON APSR-A3-D2-001" if audit["green"] else None,
        repo=repo,
    )
