"""MCP-APS-IMPORT-GUARD-001 — APS panels + backend import smoke."""

from __future__ import annotations

import importlib
import sys
from pathlib import Path

import pytest

APS_ROOT = Path(__file__).resolve().parents[2]
if str(APS_ROOT) not in sys.path:
    sys.path.insert(0, str(APS_ROOT))

APS_PANEL_IMPORTS: list[tuple[str, str]] = [
    ("art_pipeline_suite.app", "ArtPipelineSuiteApp"),
    ("art_pipeline_suite.catalog", "CatalogPanel"),
    ("art_pipeline_suite.assembly_panel", "AssemblyPanel"),
    ("art_pipeline_suite.assembly_preview_panel", "AssemblyPreviewPanel"),
    ("art_pipeline_suite.materials_panel", "MaterialsPanel"),
    ("art_pipeline_suite.variants_panel", "VariantsPanel"),
    ("art_pipeline_suite.variants_preview_panel", "VariantsPreviewPanel"),
    ("art_pipeline_suite.atlas_panel", "AtlasPanel"),
    ("art_pipeline_suite.landscape_presets_panel", "LandscapePresetsPanel"),
    ("art_pipeline_suite.landscape_grammar_panel", "LandscapeGrammarPanel"),
    ("art_pipeline_suite.landscape_states_panel", "LandscapeStatesPanel"),
    ("art_pipeline_suite.landscape_extract_parity_panel", "LandscapeExtractParityPanel"),
    ("art_pipeline_suite.landscape_state_labels", "resolver_plain_label"),
    ("art_pipeline_suite.grammar_inspector", "GrammarInspectorPanel"),
    ("art_pipeline_suite.grammar_iterate_panel", "GrammarIteratePanel"),
    ("art_pipeline_suite.job_controller", "JobController"),
    ("art_pipeline_suite.scrollable", "ScrollableFrame"),
    ("art_pipeline_suite.pipeline_status_bar", "PipelineStatusBar"),
    ("art_pipeline_suite.pipeline_pills", "format_pill"),
    ("rust_engine_mcp.veg_catalog_loader", "catalog_validator_report"),
    ("rust_engine_mcp.veg_resolver_parity", "check_resolver_catalog_parity"),
    ("rust_engine_mcp.aps_veg_state_axis", "verify_veg_state_axis"),
    ("rust_engine_mcp.aps_veg_extract_parity", "check_veg_extract_parity"),
    ("rust_engine_mcp.aps_atlas_land_register", "check_atlas_land_register"),
    ("rust_engine_mcp.landscape_lg5_expanded_batch", "write_landscape_expanded_keyframes"),
    ("art_pipeline_suite.aps_inline_feedback", "status_atom"),
    ("art_pipeline_suite.aps_inline_feedback", "flow_prerequisite_message"),
]

APS_BACKEND_SYMBOLS: list[tuple[str, str]] = [
    ("rust_engine_mcp.aps_catalog_preview", "render_module_list_thumb"),
    ("rust_engine_mcp.aps_slot_preview", "render_material_preview"),
    ("rust_engine_mcp.aps_mat_002", "write_aps_mat_002_witness"),
    ("rust_engine_mcp.aps_atlas_qc", "format_atlas_qc_display"),
    ("rust_engine_mcp.aps_artist_tool_e2e", "run_artist_tool_e2e"),
    ("rust_engine_mcp.aps_artist_tool_e2e", "refresh_aps_e0_relaunch"),
    ("rust_engine_mcp.aps_witness_honesty", "write_aps_live_witness"),
    ("rust_engine_mcp.landscape_preset_browse", "list_landscape_presets"),
    ("rust_engine_mcp.material_brief", "material_profile_brief"),
    ("rust_engine_mcp.variant_set", "load_variant_set"),
    ("rust_engine_mcp.variants_sessions", "build_variant_set_from_assembly"),
]


def test_aps_suite_py_modules_nonzero() -> None:
    suite = APS_ROOT / "art_pipeline_suite"
    skipped = {"__init__.py"}
    for path in sorted(suite.glob("*.py")):
        if path.name in skipped:
            continue
        assert path.stat().st_size > 0, f"zero-byte panel module: {path.name}"


@pytest.mark.parametrize(("module", "symbol"), APS_PANEL_IMPORTS)
def test_aps_panel_symbol_imports(module: str, symbol: str) -> None:
    mod = importlib.import_module(module)
    assert getattr(mod, symbol) is not None


@pytest.mark.parametrize(("module", "symbol"), APS_BACKEND_SYMBOLS)
def test_aps_backend_symbol_imports(module: str, symbol: str) -> None:
    mod = importlib.import_module(module)
    assert callable(getattr(mod, symbol))


def test_aps_suite_imports() -> None:
    from art_pipeline_suite import app  # noqa: F401
    from art_pipeline_suite.aps_tooltips import TOOLTIPS, bind_aps_tooltip
    from art_pipeline_suite.state import ArtDomain, SuiteState

    assert callable(bind_aps_tooltip)
    assert len(TOOLTIPS) >= 40
    assert ArtDomain.BUILDINGS.value == "buildings"
    assert SuiteState().art_domain == "buildings"


def test_aps_backend_preview_modules() -> None:
    from rust_engine_mcp.aps_catalog_preview import render_module_list_thumb
    from rust_engine_mcp.aps_mat_002 import write_aps_mat_002_witness
    from rust_engine_mcp.aps_slot_preview import render_material_preview
    from rust_engine_mcp import material_brief, variant_set

    assert callable(render_module_list_thumb)
    assert callable(render_material_preview)
    assert callable(write_aps_mat_002_witness)
    assert callable(material_brief.material_profile_brief)
    assert callable(variant_set.load_variant_set)
