"""MCP-MAT-BRIEF-001 — material_profile_brief + catalog roll-up."""

from __future__ import annotations

from rust_engine_mcp import material_brief


def test_material_profile_brief_steel():
    body = material_brief.material_profile_brief("steel_panel_01")
    assert body["ok"] is True
    assert body["schema"] == "material_profile_brief_v1"
    assert body["known"] is True
    assert body["category_path"] == "industrial/steel"
    assert body["texture_status"] in ("ready", "partial", "missing")
    assert "hint" in body


def test_material_profile_brief_unknown_infers_category():
    body = material_brief.material_profile_brief("brick_custom_99")
    assert body["ok"] is True
    assert body["category_path"] == "residential/brick"


def test_material_catalog_brief():
    body = material_brief.material_catalog_brief(max_rows=5)
    assert body["ok"] is True
    assert body["total"] > 0
    assert "ready" in body["counts"]


def test_refresh_witness():
    assert material_brief.refresh_mcp_mat_brief_witness()
