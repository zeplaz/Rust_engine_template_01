"""Regression guards for two operator-confirmed UI-jank fixes.

BUG 1 — lane switch lag: ``_apply_lane`` must early-return (no disk reads, no
landscape-panel refreshes, no lane persistence) when the clicked lane is already
the applied one.

BUG 2 — tab-switch chrome shift: ``_sync_next_step`` must not pack/forget the
advance row, and must leave the advance widgets idempotent + geometry-stable when
the advance state is unchanged across tab changes.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

APS_PARENT = Path(__file__).resolve().parents[2]
if str(APS_PARENT) not in sys.path:
    sys.path.insert(0, str(APS_PARENT))

pytestmark = pytest.mark.aps_gui


# --- BUG 1 -----------------------------------------------------------------


def test_apply_lane_noops_when_lane_unchanged(aps_app) -> None:
    """Re-selecting the already-applied lane does no redundant heavy work.

    The fixture leaves the app on 'buildings'. A second _apply_lane('buildings')
    must early-return: no landscape-panel refresh, no save_active_lane disk write.
    """
    from art_pipeline_suite import app as app_mod

    aps_app._apply_lane("buildings", log=False)
    aps_app.update_idletasks()
    assert aps_app._applied_lane == "buildings"

    saves: list[str] = []
    refreshes: list[str] = []
    orig_save = app_mod.save_active_lane
    app_mod.save_active_lane = lambda lane: saves.append(lane)
    orig_refresh = aps_app._refresh_landscape_panels
    aps_app._refresh_landscape_panels = lambda: refreshes.append("refresh")
    try:
        aps_app._apply_lane("buildings", log=False)
        aps_app.update_idletasks()
    finally:
        app_mod.save_active_lane = orig_save
        aps_app._refresh_landscape_panels = orig_refresh

    assert saves == [], "redundant save_active_lane on an unchanged lane"
    assert refreshes == [], "redundant landscape-panel refresh on an unchanged lane"


def test_apply_lane_swap_defers_heavy_work_to_idle(aps_app) -> None:
    """A real swap to landscape defers panel refresh + lane persistence to idle.

    The instant visual swap (notebook + chrome + applied-lane flag) happens
    synchronously; the disk-touching work runs on after_idle so the click feels
    instant.
    """
    from art_pipeline_suite import app as app_mod

    aps_app._apply_lane("buildings", log=False)
    aps_app.update_idletasks()

    saves: list[str] = []
    refreshes: list[str] = []
    orig_save = app_mod.save_active_lane
    app_mod.save_active_lane = lambda lane: saves.append(lane)
    orig_refresh = aps_app._refresh_landscape_panels
    aps_app._refresh_landscape_panels = lambda: refreshes.append("refresh")
    try:
        aps_app._apply_lane("landscape", log=False)
        # Instant parts are already done; deferred parts have NOT run yet.
        assert aps_app._applied_lane == "landscape"
        assert aps_app.notebook is aps_app._notebook_landscape
        assert saves == [], "lane persistence ran synchronously on the click"
        assert refreshes == [], "panel refresh ran synchronously on the click"
        # Flush the idle queue — deferred work lands now.
        aps_app.update_idletasks()
        assert saves == ["landscape"], "deferred save_active_lane did not run at idle"
        assert refreshes == ["refresh"], "deferred panel refresh did not run at idle"
    finally:
        app_mod.save_active_lane = orig_save
        aps_app._refresh_landscape_panels = orig_refresh
        aps_app._apply_lane("buildings", log=False)
        aps_app.update_idletasks()


# --- BUG 2 -----------------------------------------------------------------


def test_sync_next_step_does_not_repack_advance_row(aps_app) -> None:
    """_sync_next_step toggles text/state, never pack_forget/pack.

    Spying on the advance widgets' pack/pack_forget proves the 'Next step:' row
    keeps its reserved space across repeated syncs (the source of the chrome shift).
    """
    bar = aps_app.pipeline_status
    calls: list[str] = []
    for w in (bar._advance_btn, bar._advance_blocked_lbl):
        w.pack = lambda *a, **k: calls.append("pack")  # type: ignore[assignment]
        w.pack_forget = lambda *a, **k: calls.append("pack_forget")  # type: ignore[assignment]

    bar._sync_next_step()
    bar._sync_next_step()
    bar._sync_next_step()

    assert calls == [], f"advance row was repacked during sync: {calls}"


def test_sync_next_step_idempotent_when_state_unchanged(aps_app) -> None:
    """Two syncs with the same verb state write no widget properties the 2nd time."""
    bar = aps_app.pipeline_status
    bar._sync_next_step()  # prime cached state

    configures: list[tuple[str, dict]] = []
    orig_btn_cfg = bar._advance_btn.configure
    orig_var_set = bar._advance_blocked_var.set

    def _spy_cfg(*a, **k):
        configures.append(("btn", dict(k)))
        return orig_btn_cfg(*a, **k)

    def _spy_set(v):
        configures.append(("blocked", {"v": v}))
        return orig_var_set(v)

    bar._advance_btn.configure = _spy_cfg  # type: ignore[assignment]
    bar._advance_blocked_var.set = _spy_set  # type: ignore[assignment]
    try:
        bar._sync_next_step()  # state unchanged → no writes
    finally:
        bar._advance_btn.configure = orig_btn_cfg  # type: ignore[assignment]
        bar._advance_blocked_var.set = orig_var_set  # type: ignore[assignment]

    assert configures == [], f"redundant advance-widget writes on unchanged state: {configures}"


def test_advance_row_geometry_stable_across_tab_changes(aps_app) -> None:
    """The 'Next step:' row position/height must not move when switching tabs."""
    aps_app._apply_lane("buildings", log=False)
    aps_app.update_idletasks()
    bar = aps_app.pipeline_status

    geometries = []
    n_tabs = aps_app.notebook.index("end")
    for i in range(n_tabs):
        aps_app.notebook.select(i)
        aps_app.update_idletasks()
        geometries.append(
            (
                bar._advance_btn.winfo_y(),
                bar._advance_btn.winfo_height(),
            )
        )

    assert len(set(geometries)) == 1, (
        f"advance row geometry shifted across tabs: {geometries}"
    )
