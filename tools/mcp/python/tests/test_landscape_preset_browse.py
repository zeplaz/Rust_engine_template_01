"""MCP-LANDSCAPE-BROWSE-STUB-001 tests."""

from __future__ import annotations

from rust_engine_mcp.landscape_preset_browse import (
    list_landscape_presets,
    preset_summary,
    validate_landscape_preset,
)


def test_list_landscape_presets_index() -> None:
    body = list_landscape_presets()
    assert body["ok"] is True
    assert body["ship_count"] >= 10
    assert body["topology_count"] == 30


def test_validate_ship_preset_passes() -> None:
    listed = list_landscape_presets()
    ship = (listed.get("ship_presets") or ["fire_recovery_v0"])[0]
    report = validate_landscape_preset(ship)
    assert report.status == "passed"


def test_preset_summary_topology_kinds() -> None:
    listed = list_landscape_presets()
    ship = (listed.get("ship_presets") or ["fire_recovery_v0"])[0]
    summary = preset_summary(ship)
    assert summary["ok"] is True
    assert summary["validate_status"] == "passed"
