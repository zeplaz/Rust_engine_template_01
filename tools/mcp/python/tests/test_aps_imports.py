"""APS launch guard — fail CI if Art Pipeline Suite cannot import."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

APS_ROOT = Path(__file__).resolve().parents[2]
if str(APS_ROOT) not in sys.path:
    sys.path.insert(0, str(APS_ROOT))


def test_aps_suite_imports() -> None:
    from art_pipeline_suite import app  # noqa: F401
    from art_pipeline_suite.aps_tooltips import TOOLTIPS, bind_aps_tooltip
    from art_pipeline_suite.assembly_preview_panel import AssemblyPreviewPanel
    from art_pipeline_suite.grammar_inspector import GrammarInspectorPanel
    from art_pipeline_suite.job_controller import JobController
    from art_pipeline_suite.scrollable import ScrollableFrame
    from art_pipeline_suite.variants_panel import VariantsPanel

    assert callable(bind_aps_tooltip)
    assert len(TOOLTIPS) >= 40
    assert JobController is not None
    assert ScrollableFrame is not None
    assert VariantsPanel is not None
    assert GrammarInspectorPanel is not None
    assert AssemblyPreviewPanel is not None


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
