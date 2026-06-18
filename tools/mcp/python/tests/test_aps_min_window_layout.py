"""P3 min-window layout guards — PLAN-OVR-P3-GUARD-SPEC-001."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

APS_ROOT = Path(__file__).resolve().parents[2]
if str(APS_ROOT) not in sys.path:
    sys.path.insert(0, str(APS_ROOT))

pytestmark = pytest.mark.aps_gui

from art_pipeline_suite.aps_headless import layout_widget_visible  # noqa: E402
from art_pipeline_suite.aps_theme import MIN_WINDOW_SIZE  # noqa: E402


@pytest.fixture
def aps_app_min(aps_app):
    """Min geometry on shared session app — no second Tk root."""
    w, h = MIN_WINDOW_SIZE
    aps_app.geometry(f"{w}x{h}")
    aps_app.update_idletasks()
    yield aps_app
    from art_pipeline_suite.aps_theme import DEFAULT_WINDOW_SIZE

    dw, dh = DEFAULT_WINDOW_SIZE
    aps_app.geometry(f"{dw}x{dh}")
    aps_app.update_idletasks()


def test_aps_min_window_geometry_contract(aps_app_min) -> None:
    assert aps_app_min.minsize() == MIN_WINDOW_SIZE


def test_aps_form_tabs_disable_horizontal_scroll(aps_app_min) -> None:
    from art_pipeline_suite.scrollable import ScrollableFrame

    aps_app_min._apply_lane("buildings", log=False)
    aps_app_min.notebook.select(1)
    aps_app_min.update_idletasks()
    tab = aps_app_min.notebook.nametowidget(aps_app_min.notebook.tabs()[1])
    scrolls = [w for w in tab.winfo_children() if isinstance(w, ScrollableFrame)]
    assert scrolls, "expected ScrollableFrame on tab"
    assert scrolls[0]._enable_horizontal is False


def test_aps_assembly_footprint_visible_at_min(aps_app_min) -> None:
    aps_app_min._apply_lane("buildings", log=False)
    for i in range(aps_app_min.notebook.index("end")):
        if aps_app_min.notebook.tab(i, "text") == "Assembly":
            aps_app_min.notebook.select(i)
            break
    aps_app_min.update_idletasks()
    canvas = aps_app_min.assembly.footprint_canvas.canvas
    assert layout_widget_visible(canvas, min_height=40)


def test_aps_chrome_row_count_at_min(aps_app_min) -> None:
    assert hasattr(aps_app_min, "_chrome_row2")
    assert layout_widget_visible(aps_app_min._chrome_row2)
