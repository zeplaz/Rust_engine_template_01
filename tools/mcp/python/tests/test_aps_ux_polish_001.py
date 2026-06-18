"""APS-UX-POLISH-001 — P0 a11y + phase prerequisites."""

from __future__ import annotations

import json

from rust_engine_mcp.aps_ux_polish_001_witness import (
    APS_UX_POLISH_001_WITNESS,
    refresh_aps_ux_polish_001_witness,
)
from rust_engine_mcp.paths import repo_root


def test_p0_validation_uses_theme_colors():
    text = (repo_root() / "tools/mcp/art_pipeline_suite/aps_inline_feedback.py").read_text(
        encoding="utf-8"
    )
    assert "COLOR_FAIL" in text
    assert "validation_foreground" in text


def test_p0_material_status_label():
    mat = (repo_root() / "tools/mcp/art_pipeline_suite/material_library_widget.py").read_text(
        encoding="utf-8"
    )
    assert "def _status_label" in mat
    assert "ready" in mat


def test_p0_metadata_collapsed_hint():
    meta = (repo_root() / "tools/mcp/art_pipeline_suite/metadata_flow_panel.py").read_text(
        encoding="utf-8"
    )
    assert "_initial_expanded" in meta
    assert "_collapsed_hint" in meta


def test_refresh_polish_001_witness():
    assert refresh_aps_ux_polish_001_witness()
    data = json.loads((repo_root() / APS_UX_POLISH_001_WITNESS).read_text(encoding="utf-8"))
    assert data["green"] is True
    assert all(data["p0_a11y"].values())
