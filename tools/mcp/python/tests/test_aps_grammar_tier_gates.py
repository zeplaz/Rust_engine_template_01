"""APS-GRAM-TIER-002 — AssemblyPanel tier gates at G0."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

APS_PARENT = Path(__file__).resolve().parents[2]
if str(APS_PARENT) not in sys.path:
    sys.path.insert(0, str(APS_PARENT))

pytest.importorskip("PIL")
pytestmark = pytest.mark.aps_gui


def _make_panel(gui_panel_host):
    from art_pipeline_suite.assembly_panel import AssemblyPanel
    from art_pipeline_suite.state import SuiteState

    panel = AssemblyPanel(gui_panel_host, SuiteState(), on_log=lambda _m: None, start_job=None)
    panel.pack(fill="both", expand=True)
    gui_panel_host.winfo_toplevel().update_idletasks()
    return panel


def test_apply_grammar_tier_g0_hides_advanced_panels(gui_panel_host) -> None:
    panel = _make_panel(gui_panel_host)
    panel.apply_grammar_tier("G0")
    gui_panel_host.winfo_toplevel().update_idletasks()

    snap = panel.grammar_tier_gate_snapshot()
    assert snap["tier"] == "G0"
    assert snap["dna_panel_visible"] is False
    assert snap["iterate_panel_visible"] is False
    assert snap["kit_hint_visible"] is True
    assert snap["build_set_expanded_default"] is False


def test_refresh_grammar_tier_from_registry_matches_api(gui_panel_host) -> None:
    from rust_engine_mcp import grammar_build_set

    panel = _make_panel(gui_panel_host)
    panel.refresh_grammar_tier_from_registry()
    gui_panel_host.winfo_toplevel().update_idletasks()

    expected = grammar_build_set.grammar_set_tier()["tier"]
    assert panel._grammar_set_tier == expected


def test_write_aps_grammar_tier_gates_witness(gui_panel_host) -> None:
    from rust_engine_mcp import grammar_build_set

    panel = _make_panel(gui_panel_host)
    panel.apply_grammar_tier("G0")
    gui_panel_host.winfo_toplevel().update_idletasks()
    snap = panel.grammar_tier_gate_snapshot()

    body = grammar_build_set.write_aps_grammar_tier_gates_witness(
        tier=str(snap["tier"]),
        dna_panel_visible=bool(snap["dna_panel_visible"]),
        iterate_panel_visible=bool(snap["iterate_panel_visible"]),
        build_set_expanded_default=bool(snap["build_set_expanded_default"]),
        kit_hint_visible=bool(snap["kit_hint_visible"]),
        archetype_combo_count=int(snap["archetype_combo_count"]),
    )
    assert body["tier"] == "G0"
    assert body["dna_panel_visible"] is False
    assert body["kit_hint_visible"] is True


def test_apply_grammar_tier_g1_refresh(gui_panel_host) -> None:
    from rust_engine_mcp import grammar_build_set

    panel = _make_panel(gui_panel_host)
    panel.apply_grammar_tier("G1")
    gui_panel_host.winfo_toplevel().update_idletasks()
    snap = panel.grammar_tier_gate_snapshot()

    assert snap["tier"] == "G1"
    assert snap["kit_hint_visible"] is False
    assert snap["archetype_combo_count"] >= 3
    assert snap["dna_panel_visible"] is False
    assert snap["iterate_panel_visible"] is False

    grammar_build_set.write_aps_grammar_tier_g1_gates_witness(
        archetype_combo_count=int(snap["archetype_combo_count"]),
        kit_hint_visible=bool(snap["kit_hint_visible"]),
        dna_panel_visible=bool(snap["dna_panel_visible"]),
        iterate_panel_visible=bool(snap["iterate_panel_visible"]),
    )
