"""Tests for CITY-G2-C5-001 MCP lane — palette index + atlas keys."""

from __future__ import annotations

from pathlib import Path

from rust_engine_mcp import city_palette_g2
from rust_engine_mcp.paths import repo_root


def test_palette_fields_for_pilot_industrial_module() -> None:
    fields = city_palette_g2.palette_fields_for_entry(
        {
            "module_id": "wall_concrete_2u",
            "style_pack": "style_industrial_west",
            "development_tier": "production",
        }
    )
    assert fields["palette_family"] == "palette_industrial_west"
    assert fields["palette_variation_count"] == 3


def test_palette_fields_skip_smoke_tier() -> None:
    assert city_palette_g2.palette_fields_for_entry(
        {
            "module_id": "wall_concrete_2u",
            "style_pack": "style_industrial_west",
            "development_tier": "smoke",
        }
    ) == {}


def test_wire_rowhouse_atlas_palette_keys_idempotent() -> None:
    first = city_palette_g2.wire_rowhouse_atlas_palette_keys()
    assert first.get("ok") is True
    second = city_palette_g2.wire_rowhouse_atlas_palette_keys()
    assert second.get("ok") is True
    assert second.get("added") == []


def test_city_g2_c5_mcp_audit_shape() -> None:
    body = city_palette_g2.audit_city_g2_c5_mcp()
    assert body["task_id"] == "CITY-G2-C5-001"
    assert "checks" in body
    assert (repo_root() / "tools/mcp/schemas/palette_catalog_v1.schema.json").is_file()
