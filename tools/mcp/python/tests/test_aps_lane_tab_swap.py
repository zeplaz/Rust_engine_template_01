"""APS-E1-TAB-SWAP-001 — dual notebook tab sets (no zip-label remap)."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

APS_ROOT = Path(__file__).resolve().parents[2]
if str(APS_ROOT) not in sys.path:
    sys.path.insert(0, str(APS_ROOT))


@pytest.fixture
def aps_app():
    import tkinter as tk

    from art_pipeline_suite.app import ArtPipelineSuiteApp

    try:
        app = ArtPipelineSuiteApp()
    except tk.TclError as exc:
        pytest.skip(f"no Tk display: {exc}")
    app.withdraw()
    app.update_idletasks()
    yield app
    try:
        app.destroy()
    except tk.TclError:
        pass


def _tab_texts(notebook) -> list[str]:
    return [notebook.tab(i, "text") for i in range(notebook.index("end"))]


def test_landscape_notebook_four_tabs_no_materials(aps_app) -> None:
    aps_app._apply_lane("landscape", log=False)
    tabs = _tab_texts(aps_app._notebook_landscape)
    assert tabs == ["Presets", "Grammar", "States", "Atlas"]
    assert "Materials" not in tabs
    assert "Assembly" not in tabs
    assert "Variants" not in tabs


def test_buildings_notebook_five_tabs_unchanged(aps_app) -> None:
    aps_app._apply_lane("buildings", log=False)
    tabs = _tab_texts(aps_app._notebook_buildings)
    assert len(tabs) == 5
    assert tabs[0] == "Catalog"
    assert "Materials" in tabs
    assert "Assembly" in tabs


def test_no_zip_label_remap_on_single_notebook(aps_app) -> None:
    """Option D forbids renaming 5 building panels to 4 landscape labels."""
    aps_app._apply_lane("buildings", log=False)
    b_tabs = _tab_texts(aps_app._notebook_buildings)
    aps_app._apply_lane("landscape", log=False)
    l_tabs = _tab_texts(aps_app._notebook_landscape)
    assert b_tabs != l_tabs
    assert aps_app._notebook_buildings is not aps_app._notebook_landscape


def test_landscape_panels_not_building_classes(aps_app) -> None:
    from art_pipeline_suite.assembly_panel import AssemblyPanel
    from art_pipeline_suite.landscape_grammar_panel import LandscapeGrammarPanel
    from art_pipeline_suite.landscape_presets_panel import LandscapePresetsPanel
    from art_pipeline_suite.variants_panel import VariantsPanel

    assert isinstance(aps_app.landscape_presets, LandscapePresetsPanel)
    assert isinstance(aps_app.landscape_grammar, LandscapeGrammarPanel)
    assert not isinstance(aps_app.landscape_grammar, AssemblyPanel)
    assert not isinstance(aps_app.variants, VariantsPanel) or aps_app.variants is not aps_app.landscape_states


def test_verify_e1_tab_swap_headless() -> None:
    from rust_engine_mcp.aps_option_d_e1 import verify_e1_tab_swap

    body = verify_e1_tab_swap()
    assert body["green"] is True
    assert body["landscape_tab_count"] == 4
    assert body["materials_in_landscape_tabs"] is False
