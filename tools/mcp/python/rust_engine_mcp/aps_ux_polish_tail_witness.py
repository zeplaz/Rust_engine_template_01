"""APS-UX-POLISH tail — audit v2 F5/F6/F7 coder-mcp fixes."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

WITNESS_REL = "debug_runs/aps_ux_polish_tail_live.json"


def audit_ux_polish_tail(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    suite = root / "tools/mcp/art_pipeline_suite"
    pipeline = (suite / "pipeline_status_bar.py").read_text(encoding="utf-8")
    pills = (suite / "pipeline_pills.py").read_text(encoding="utf-8")
    app = (suite / "app.py").read_text(encoding="utf-8")
    footprint = (suite / "footprint_canvas.py").read_text(encoding="utf-8")
    theme = (suite / "aps_theme.py").read_text(encoding="utf-8")

    f5_valid_saved = (
        "assembly_p0_passed" in pipeline
        and "saved_qc_not_run" in pipeline
        and "saved (not checked)" in pills
    )
    f6_footprint_text = "TOKEN_LABELS" in footprint and "—" in footprint
    f7_flow_feedback = "already on" in app and "_on_pipeline_step" in app
    f4_font_small = 'FONT_SMALL = ("Segoe UI", 9)' in theme

    tag_tier2 = (root / "debug_runs/aps_tag_tier2_live.json")
    tier2_green = False
    if tag_tier2.is_file():
        tier2_green = bool(json.loads(tag_tier2.read_text(encoding="utf-8")).get("green"))

    green = f5_valid_saved and f6_footprint_text and f7_flow_feedback and f4_font_small and tier2_green
    return {
        "task_id": "APS-UX-POLISH-TAIL-001",
        "program_id": "APS-UX-AUDIT-001",
        "green": green,
        "fixes": {
            "f5_pipeline_valid_vs_saved": f5_valid_saved,
            "f6_footprint_swatch_text": f6_footprint_text,
            "f7_flow_bar_feedback": f7_flow_feedback,
            "f4_font_small_9pt": f4_font_small,
            "tag_tier2_impl": tier2_green,
        },
        "audit_ref": "src/dev/design_aps_ux_audit_v2.md",
    }


def write_aps_ux_polish_tail_witness(*, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    body = audit_ux_polish_tail(repo=repo)
    return write_aps_live_witness(
        body,
        WITNESS_REL,
        schema="aps_ux_polish_tail_live_v1",
        profile="APS_UX_POLISH_TAIL",
        source_system="aps_ux_polish_tail_witness",
        ritual="BLANG:WIT-HON APS-UX-POLISH-TAIL-001" if body.get("green") else None,
    )
