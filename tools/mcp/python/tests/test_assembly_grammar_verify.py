"""GRAMMAR-GEN-VERIFY-001 (P0) tests."""

from __future__ import annotations

import json

import pytest

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators.assembly_grammar_verify import (
    validate_assembly_grammar_verify,
    validate_assembly_grammar_verify_path,
    validate_assembly_p0_gate,
)

ROOT = repo_root()
WAREHOUSE_7X5 = ROOT / "assets/staging/assemblies/industrial_west_7x5_s39_9fa1.json"
WAREHOUSE_4X2 = ROOT / "assets/staging/assemblies/industrial_west_4x2_s43_a879.json"


@pytest.mark.skipif(not WAREHOUSE_7X5.is_file(), reason="warehouse 7x5 snapshot missing")
def test_industrial_7x5_passes_style_pack_after_p0_fix() -> None:
    snap = json.loads(WAREHOUSE_7X5.read_text(encoding="utf-8"))
    rep = validate_assembly_grammar_verify(snap, ship=True)
    assert rep.status == "passed"


@pytest.mark.skipif(not WAREHOUSE_4X2.is_file(), reason="warehouse 4x2 snapshot missing")
def test_industrial_4x2_fails_thin_footprint() -> None:
    snap = json.loads(WAREHOUSE_4X2.read_text(encoding="utf-8"))
    rep = validate_assembly_grammar_verify(snap, ship=True)
    kinds = {e.kind for e in rep.errors}
    assert "WarehouseFootprintThin" in kinds or "StylePackDrift" in kinds


def test_minimal_valid_shell_passes_grammar_checks() -> None:
    snap = {
        "schema_version": 1,
        "assembly_id": "test_industrial_6x4_s1",
        "style_pack_id": "style_industrial_west",
        "source_tier": "production",
        "procedural_rules_version": "building_grammar_v1",
        "reference_tags": ["archetype:IndustrialWarehouse"],
        "footprint": {"width": 6, "depth": 4, "floors": 1},
        "grammar_rule_chain": [
            {"layer": "archetype", "rule_id": "IndustrialWarehouse"},
            {"layer": "massing", "rule_id": "long_hall"},
            {"layer": "facade", "rule_id": "facade_v1"},
            {"layer": "roof", "rule_id": "roof_default"},
        ],
        "module_placements": [],
    }
    for x, y, token, mid in [
        (0, 0, "C", "corner_L"),
        (1, 0, "W", "wall_steel_1u"),
        (2, 0, "W", "wall_steel_1u"),
        (3, 0, "W", "wall_steel_1u"),
        (4, 0, "W", "wall_steel_1u"),
        (5, 0, "C", "corner_L"),
        (0, 1, "W", "wall_steel_1u"),
        (5, 1, "W", "wall_steel_1u"),
        (0, 2, "W", "wall_steel_1u"),
        (5, 2, "W", "wall_steel_1u"),
        (0, 3, "C", "corner_L"),
        (1, 3, "D", "door_shop"),
        (2, 3, "W", "wall_steel_1u"),
        (3, 3, "W", "wall_steel_1u"),
        (4, 3, "W", "wall_steel_1u"),
        (5, 3, "C", "corner_L"),
    ]:
        snap["module_placements"].append(
            {
                "module_id": mid,
                "job_id": f"{mid}_production_run001",
                "token": token,
                "grid_x": x,
                "grid_y": y,
                "floor": 0,
                "glb_path": f"assets/models/modules/{mid}_production_run001/model.glb",
                "material_profile": "steel_panel_01",
            }
        )
    for x in range(6):
        for y in range(4):
            if x in (0, 5) or y in (0, 3):
                snap["module_placements"].append(
                    {
                        "module_id": "roof_sawtooth",
                        "job_id": "roof_sawtooth_production_run001",
                        "token": "R",
                        "grid_x": x,
                        "grid_y": y,
                        "floor": 1,
                        "glb_path": "assets/models/modules/roof_sawtooth_production_run001/model.glb",
                        "material_profile": "roof_metal_01",
                    }
                )
    rep = validate_assembly_grammar_verify(snap, ship=False)
    assert rep.status == "passed"


def test_cli_path_wrapper_full_p0() -> None:
    if not WAREHOUSE_7X5.is_file():
        pytest.skip("warehouse 7x5 snapshot missing")
    rep = validate_assembly_grammar_verify_path(WAREHOUSE_7X5, full_p0=True)
    assert rep.status == "passed"
    assert rep.validator == "assembly_p0"
