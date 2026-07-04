"""APSR-A0-T2-001 — panel sync characterization + stale-assembly xfail repro."""

from __future__ import annotations

import pytest

pytestmark = pytest.mark.aps_gui


def _generate_victorian_snapshot(aps_app) -> None:
    aps_app._apply_lane("buildings", log=False)
    aps_app.assembly.style_var.set("style_victorian")
    aps_app.assembly.footprint_var.set("4x2")
    aps_app.assembly.floors_var.set(2)
    aps_app.assembly.seed_var.set(42)
    aps_app.assembly.use_grammar_var.set(False)
    aps_app.assembly.on_generate()
    aps_app.update_idletasks()
    assert aps_app.assembly._snapshot is not None


def test_style_pack_change_leaves_shadow_snapshot_stale(aps_app) -> None:
    """Catalog can change ``state.style_pack_id`` and sync patches live snapshot."""
    _generate_victorian_snapshot(aps_app)
    snap_style = str(aps_app.assembly._snapshot.get("style_pack_id"))
    assert snap_style == "style_victorian"

    aps_app.state.style_pack_id = "style_industrial_west"
    aps_app.assembly.sync_from_state()
    aps_app.update_idletasks()

    assert aps_app.assembly.style_var.get() == "style_industrial_west"
    assert str(aps_app.assembly._snapshot.get("style_pack_id")) == "style_industrial_west"


def test_lane_round_trip_leaves_assembly_shadow_snapshot_stale(aps_app) -> None:
    _generate_victorian_snapshot(aps_app)
    before_id = aps_app.state.assembly_id
    assert before_id

    aps_app.state.style_pack_id = "style_industrial_west"
    aps_app._apply_lane("landscape", log=False)
    aps_app.update_idletasks()
    aps_app._apply_lane("buildings", log=False)
    aps_app.update_idletasks()

    assert aps_app.assembly._snapshot is not None
    assert str(aps_app.assembly._snapshot.get("style_pack_id")) == "style_industrial_west"
    assert aps_app.state.assembly_id == before_id


def test_send_to_assembly_syncs_panel_chrome_from_state(aps_app) -> None:
    """``on_send_to_assembly`` must at least mirror catalog fields into panel chrome."""
    aps_app.state.style_pack_id = "style_industrial_west"
    aps_app.state.footprint = "6x4"
    aps_app.state.floors = 3
    aps_app.state.seed = 7

    aps_app.on_send_to_assembly()
    aps_app.update_idletasks()

    assert aps_app.assembly.style_var.get() == "style_industrial_west"
    assert aps_app.assembly.footprint_var.get() == "6x4"
    assert int(aps_app.assembly.floors_var.get()) == 3
    assert int(aps_app.assembly.seed_var.get()) == 7


def test_bake_variants_prerequisite_reads_state_snapshot_not_shadow_only(aps_app) -> None:
    """Variants bake path reads ``state.assembly_snapshot_data`` when present."""
    _generate_victorian_snapshot(aps_app)
    assert aps_app.state.assembly_snapshot_data is not None
    assert aps_app.state.assembly_snapshot_data.get("assembly_id") == aps_app.state.assembly_id
