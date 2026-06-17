"""P1-1 regression — MaterialLibraryWidget flat-category filter must not NameError.

Guards the bug where `_category_matches_filter` referenced an undefined `entry`
instead of its `entry_category` parameter on the non-studio_tree (flat) branch.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

APS_ROOT = Path(__file__).resolve().parents[2]  # tools/mcp
if str(APS_ROOT) not in sys.path:
    sys.path.insert(0, str(APS_ROOT))
if str(APS_ROOT / "python") not in sys.path:
    sys.path.insert(0, str(APS_ROOT / "python"))

tk = pytest.importorskip("tkinter")
pytest.importorskip("PIL")


@pytest.fixture
def root():
    try:
        r = tk.Tk()
    except tk.TclError:  # pragma: no cover - no display
        pytest.skip("no Tk display available")
    r.withdraw()
    yield r
    try:
        r.destroy()
    except tk.TclError:
        pass


def _make_widget(root, layout="vertical"):
    from art_pipeline_suite.material_library_widget import MaterialLibraryWidget

    return MaterialLibraryWidget(root, layout=layout)


def test_flat_category_filter_does_not_nameerror(root):
    """The flat-category (non-studio_tree) branch must read entry_category, not `entry`."""
    w = _make_widget(root, layout="vertical")

    # "all" — everything passes regardless of entry_category.
    w._category_var.set("all")
    assert w._category_matches_filter("industrial/wall") is True
    assert w._category_matches_filter("anything") is True

    # A concrete flat category — must compare against entry_category, no NameError.
    w._category_var.set("industrial")
    assert w._category_matches_filter("industrial") is True
    assert w._category_matches_filter("INDUSTRIAL") is True  # case-insensitive
    assert w._category_matches_filter("residential") is False


def test_studio_tree_category_filter_branch(root):
    """The studio_tree branch (prefix / parent match) still works after the fix."""
    w = _make_widget(root, layout="studio_tree")
    w._tree_category = "industrial"
    assert w._category_matches_filter("industrial") is True
    assert w._category_matches_filter("industrial/wall") is True  # child prefix
    assert w._category_matches_filter("residential") is False

    w._tree_category = None  # falls through to flat branch
    w._category_var.set("all")
    assert w._category_matches_filter("anything") is True
