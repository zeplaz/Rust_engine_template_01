"""P3 min-window layout guards — PLAN-OVR-P3-GUARD-SPEC-001."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

APS_ROOT = Path(__file__).resolve().parents[2]
if str(APS_ROOT) not in sys.path:
    sys.path.insert(0, str(APS_ROOT))

tk = pytest.importorskip("tkinter")

from art_pipeline_suite.aps_theme import MIN_WINDOW_SIZE  # noqa: E402


@pytest.fixture
def tk_root():
    try:
        root = tk.Tk()
    except tk.TclError as exc:
        pytest.skip(f"no Tk display: {exc}")
    root.withdraw()
    yield root
    try:
        root.destroy()
    except tk.TclError:
        pass


@pytest.fixture
def aps_app(tk_root):
    from art_pipeline_suite.app import ArtPipelineSuiteApp

    try:
        app = ArtPipelineSuiteApp()
    except tk.TclError as exc:
        pytest.skip(f"no Tk display: {exc}")
    app.withdraw()
    w, h = MIN_WINDOW_SIZE
    app.geometry(f"{w}x{h}")
    app.update_idletasks()
    yield app
    try:
        app.destroy()
    except tk.TclError:
        pass


def test_aps_min_window_geometry_contract(aps_app) -> None:
    assert aps_app.minsize() == MIN_WINDOW_SIZE


def test_aps_form_tabs_disable_horizontal_scroll(aps_app) -> None:
    from art_pipeline_suite.scrollable import ScrollableFrame

    aps_app._apply_lane("buildings", log=False)
    aps_app.notebook.select(1)
    aps_app.update_idletasks()
    tab = aps_app.notebook.nametowidget(aps_app.notebook.tabs()[1])
    scrolls = [w for w in tab.winfo_children() if isinstance(w, ScrollableFrame)]
    assert scrolls, "expected ScrollableFrame on tab"
    assert scrolls[0]._enable_horizontal is False


def test_aps_assembly_footprint_visible_at_min(aps_app) -> None:
    aps_app._apply_lane("buildings", log=False)
    for i in range(aps_app.notebook.index("end")):
        if aps_app.notebook.tab(i, "text") == "Assembly":
            aps_app.notebook.select(i)
            break
    aps_app.update_idletasks()
    canvas = aps_app.assembly.footprint_canvas.canvas
    assert canvas.winfo_viewable()
    assert canvas.winfo_height() > 40


def test_aps_chrome_row_count_at_min(aps_app) -> None:
    assert hasattr(aps_app, "_chrome_row2")
    assert aps_app._chrome_row2.winfo_viewable()
