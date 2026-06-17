"""APS-E1-PIPELINE-LANE-001 — landscape pipeline pills + Stamp step."""

from __future__ import annotations

import sys
from pathlib import Path

APS_ROOT = Path(__file__).resolve().parents[2]
if str(APS_ROOT) not in sys.path:
    sys.path.insert(0, str(APS_ROOT))


def test_landscape_pipeline_includes_stamp() -> None:
    from art_pipeline_suite.domain_router import pipeline_steps_for
    from art_pipeline_suite.state import ArtDomain

    keys = [k for k, _ in pipeline_steps_for(ArtDomain.LANDSCAPE.value)]
    assert keys == ["presets", "grammar", "states", "atlas", "stamp"]


def test_pipeline_pills_landscape_stamp_pending() -> None:
    import tkinter as tk

    from art_pipeline_suite.pipeline_status_bar import PipelineStatusBar
    from art_pipeline_suite.state import ArtDomain, SuiteState

    root = tk.Tk()
    root.withdraw()
    state = SuiteState(art_domain=ArtDomain.LANDSCAPE.value)
    bar = PipelineStatusBar(root, state)
    bar.refresh()
    stamp_var = bar._pills["stamp"][1].cget("text")
    assert "Stamp" in stamp_var
    assert "pending" in stamp_var.lower() or "○" in stamp_var
    bar.destroy()
    root.destroy()


def test_pipeline_pills_presets_valid_after_validate() -> None:
    import tkinter as tk

    from art_pipeline_suite.pipeline_status_bar import PipelineStatusBar
    from art_pipeline_suite.state import ArtDomain, SuiteState

    root = tk.Tk()
    root.withdraw()
    state = SuiteState(
        art_domain=ArtDomain.LANDSCAPE.value,
        selected_landscape_preset_id="fire_recovery_v0",
        landscape_preset_validate_ok=True,
    )
    bar = PipelineStatusBar(root, state)
    bar.refresh()
    text = bar._pills["presets"][1].cget("text")
    assert "valid" in text.lower() or "✓" in text
    bar.destroy()
    root.destroy()


def test_verify_e1_pipeline_lane_headless() -> None:
    from rust_engine_mcp.aps_option_d_e1 import verify_e1_pipeline_lane

    body = verify_e1_pipeline_lane()
    assert body["green"] is True
    assert body["has_stamp_step"] is True
