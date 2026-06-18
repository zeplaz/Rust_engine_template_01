"""APS-GRAM-P3-001 — inspector row → footprint highlight."""

from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest

APS_PARENT = Path(__file__).resolve().parents[2]
if str(APS_PARENT) not in sys.path:
    sys.path.insert(0, str(APS_PARENT))

pytest.importorskip("PIL")
pytestmark = pytest.mark.aps_gui


def test_long_hall_rule_highlights_footprint_cells(gui_panel_host) -> None:
    from art_pipeline_suite.assembly_panel import AssemblyPanel
    from art_pipeline_suite.state import SuiteState
    from rust_engine_mcp import assembly

    panel = AssemblyPanel(gui_panel_host, SuiteState(), on_log=lambda _m: None, start_job=None)
    panel.pack(fill="both", expand=True)
    gui_panel_host.winfo_toplevel().update_idletasks()

    snap = assembly.generate_assembly_snapshot(
        archetype_id="IndustrialWarehouse",
        district_style="industrial_west",
        seed=42,
        source_tier="production",
    )
    panel._load_snapshot_into_ui(snap)
    gui_panel_host.winfo_toplevel().update_idletasks()

    count = panel.footprint_canvas.highlight_for_rule("long_hall")
    assert count >= 1

    panel._on_grammar_inspector_rule_select("massing", "long_hall")
    assert len(panel.footprint_canvas._rule_highlight_cells) >= 1


def test_write_aps_grammar_p3_witness(gui_panel_host) -> None:
    from art_pipeline_suite.assembly_panel import AssemblyPanel
    from art_pipeline_suite.state import SuiteState
    from rust_engine_mcp import assembly
    from rust_engine_mcp.paths import repo_root

    panel = AssemblyPanel(gui_panel_host, SuiteState(), on_log=lambda _m: None, start_job=None)
    panel.pack(fill="both", expand=True)
    gui_panel_host.winfo_toplevel().update_idletasks()

    snap = assembly.generate_assembly_snapshot(
        archetype_id="IndustrialWarehouse",
        district_style="industrial_west",
        seed=42,
        source_tier="production",
    )
    panel._load_snapshot_into_ui(snap)
    count = panel.footprint_canvas.highlight_for_rule("long_hall")

    body = {
        "inspector_click_highlights_grid": count >= 1,
        "rule_id_tested": "long_hall",
        "cells_highlighted_count": count,
        "seed": 42,
        "archetype_id": "IndustrialWarehouse",
        "district_style": "industrial_west",
    }
    out = repo_root() / "debug_runs/aps_grammar_p3_live.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    assert body["inspector_click_highlights_grid"] is True
