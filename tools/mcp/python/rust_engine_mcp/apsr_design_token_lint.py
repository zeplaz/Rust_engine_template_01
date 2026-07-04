"""APSR-D1 — design token lint (fonts, hex, banned chrome)."""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

SUITE_REL = "tools/mcp/art_pipeline_suite"

HEX_ALLOWLIST = frozenset(
    {
        "aps_theme.py",
        "footprint_canvas.py",
        "atlas_preview_panel.py",
        "landscape_grammar_panel.py",
        "assembly_panel.py",
        "assembly_preview_section.py",
        "status_log_panel.py",
    }
)

BANNED_UI_HEX = (
    'foreground="#555"',
    'foreground="#444"',
    'foreground="#0a4a7a"',
    'foreground="#8b0000"',
    'font=("Segoe UI", 7)',
)

SUB_NINE_FONT = re.compile(r'\(\s*"(?:Segoe UI|Consolas)"\s*,\s*([0-8])\b')
RAW_HEX_IN_KW = re.compile(
    r"""(?:foreground|background|bg|fg|fill|outline)\s*=\s*["']#([0-9a-fA-F]{3,8})["']"""
)


@dataclass(frozen=True)
class TokenViolation:
    file: str
    line: int
    detail: str


def _suite_root(*, repo: Path | None = None) -> Path:
    return (repo or repo_root()) / SUITE_REL


def scan_token_violations(*, repo: Path | None = None) -> list[TokenViolation]:
    root = _suite_root(repo=repo)
    out: list[TokenViolation] = []
    for path in sorted(root.glob("*.py")):
        if path.name in HEX_ALLOWLIST:
            continue
        lines = path.read_text(encoding="utf-8").splitlines()
        for i, line in enumerate(lines, start=1):
            for banned in BANNED_UI_HEX:
                if banned in line:
                    out.append(TokenViolation(path.name, i, f"banned chrome: {banned}"))
            m = SUB_NINE_FONT.search(line)
            if m and not (path.name == "footprint_canvas.py" and "glyph_size" in line):
                out.append(TokenViolation(path.name, i, "sub-9px font literal"))
            if RAW_HEX_IN_KW.search(line):
                out.append(TokenViolation(path.name, i, "raw hex in UI kwargs"))
    return out


def token_lint_audit(*, repo: Path | None = None) -> dict[str, Any]:
    violations = scan_token_violations(repo=repo)
    return {
        "violation_count": len(violations),
        "violations": [{"file": v.file, "line": v.line, "detail": v.detail} for v in violations[:40]],
        "green": not violations,
    }


def write_apsr_d1_witness(*, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    audit = token_lint_audit(repo=repo)
    body: dict[str, Any] = {
        "task_id": "APSR-A3-D1-001",
        "gate": "APSR-A3-D1-001",
        "green": audit["green"],
        "violation_count": audit["violation_count"],
        "plan_ref": "src/dev/plan_aps_refactor_v1.md#APSR-D1",
    }
    return write_aps_live_witness(
        body,
        "debug_runs/apsr_a3_d1_001_live.json",
        schema="apsr_a3_d1_live_v1",
        profile="APSR_A3_D1",
        source_system="apsr_a3_d1",
        ritual="BLANG:WIT-HON APSR-A3-D1-001" if audit["green"] else None,
        repo=repo,
    )
