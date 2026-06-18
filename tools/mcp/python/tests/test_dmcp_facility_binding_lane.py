"""DMCP facility binding lane witness tests."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from rust_engine_mcp.dmcp_facility_binding_lane import (
    power_tier_for_units,
    refresh_dmcp_facility_binding_lane_witness,
    run_facility_binding_lane_audit,
)
from rust_engine_mcp.paths import repo_root, schemas_dir
from rust_engine_mcp.schemas import load_json_file, validate_building_grammar


def test_dmcp_facility_binding_lane_witness() -> None:
    body = refresh_dmcp_facility_binding_lane_witness()
    assert body.get("green") is True
    assert body.get("done_count") == 3


def test_all_grammar_bindings_green() -> None:
    audit = run_facility_binding_lane_audit()
    bindings = audit["grammar_bindings"]
    assert bindings["binding_count"] == 3
    assert bindings["green_binding_count"] == 3
    assert bindings["green"] is True


def test_rail_edge_grammar_with_binding_schema() -> None:
    path = schemas_dir() / "examples" / "building_grammar_rail_edge_v1.json"
    data = load_json_file(path)
    validate_building_grammar(data)
    assert data["facility_binding"]["catalog_id"] == "logistics_rail_warehouse"


def test_factory_cluster_grammar_with_binding_schema() -> None:
    path = schemas_dir() / "examples" / "building_grammar_factory_cluster_v1.json"
    data = load_json_file(path)
    validate_building_grammar(data)
    binding = data["facility_binding"]
    assert binding["catalog_id"] == "concrete_mixer_plant"
    assert binding["power_tier"] == power_tier_for_units(28)


def test_concrete_site_grids_cell_count() -> None:
    root = repo_root()
    for rel in (
        "assets/configs/buildings/pilots/concrete_aggregate_mine_site_v0.json",
        "assets/configs/buildings/pilots/concrete_cement_kiln_site_v0.json",
        "assets/configs/buildings/pilots/concrete_mixer_plant_site_v0.json",
    ):
        body = json.loads((root / rel).read_text(encoding="utf-8"))
        assert len(body["cells"]) == body["width"] * body["depth"]


def test_concrete_pilot_three_steps_match_chain() -> None:
    audit = run_facility_binding_lane_audit()
    pilot_row = next(r for r in audit["rows"] if r["id"] == "DMCP-PILOT-CONCRETE-SITE-001")
    assert pilot_row["pilot_audit"]["green"] is True
    assert len(pilot_row["pilot_audit"]["steps"]) == 3
