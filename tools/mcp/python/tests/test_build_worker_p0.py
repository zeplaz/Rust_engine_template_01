"""P0 module resolution + BUILD-WORKER-001 tests."""

from __future__ import annotations

import json

import pytest

from rust_engine_mcp import assembly
from rust_engine_mcp.assembly_build_worker import (
    ensure_snapshot_material_textures,
    material_profiles_in_snapshot,
)
from rust_engine_mcp.building_definition import MINIMUM_G4_CELLS, expand_bake_matrix_minimum, load_building_definition
from rust_engine_mcp.material_textures import write_registry
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators.assembly_grammar_verify import validate_assembly_p0_gate

EXAMPLE_SNAP = (
    repo_root()
    / "tools/mcp/schemas/examples/assembly_snapshot_warehouse_industrial_west_production_v1.json"
)
BDEF = (
    repo_root()
    / "tools/mcp/schemas/examples/building_definition_warehouse_industrial_west_production_v1.json"
)


def test_style_pack_resolves_industrial_corner_production_not_victorian():
    # Live generate uses corners_as_walls (PG-MODULE-AUDIT-001); still assert the
    # style-pack slot resolves to an industrial_west production corner, not Victorian.
    from rust_engine_mcp.library import load_index_json

    pack = assembly.load_style_pack("style_industrial_west")
    mid = str((pack.get("slots") or {}).get("corner_outer") or "")
    assert mid == "corner_L"
    row = assembly._resolve_module_row(
        mid,
        load_index_json(),
        style_pack_id="style_industrial_west",
        source_tier="production",
    )
    assert row is not None, "corner_outer must resolve in style_industrial_west"
    job = str(row.get("job_id") or "")
    assert "victorian" not in job.lower()
    assert job == "corner_L_industrial_west_production_run001"
    assert "industrial_west" in job or str(row.get("development_tier") or "") == "production"

    snap = assembly.generate_assembly_snapshot(
        archetype_id="IndustrialWarehouse",
        district_style="industrial_west",
        seed=39,
        source_tier="production",
        write=False,
    )
    jobs = [str(p.get("job_id") or "") for p in snap.get("module_placements") or []]
    assert jobs
    assert all("victorian" not in j.lower() for j in jobs)


def test_warehouse_grammar_uses_door_warehouse_not_door_shop():
    snap = assembly.generate_assembly_snapshot(
        archetype_id="IndustrialWarehouse",
        district_style="industrial_west",
        seed=39,
        source_tier="production",
        write=False,
    )
    doors = [p for p in snap["module_placements"] if "door" in str(p.get("module_id") or "")]
    assert doors
    assert all("door_warehouse" in str(p.get("module_id")) for p in doors)


def test_warehouse_example_passes_p0_gate_no_style_drift():
    if not EXAMPLE_SNAP.is_file():
        pytest.skip("warehouse example missing")
    snap = json.loads(EXAMPLE_SNAP.read_text(encoding="utf-8"))
    rep = validate_assembly_p0_gate(snap, snapshot_path=str(EXAMPLE_SNAP), ship=True)
    kinds = {e.kind for e in rep.errors}
    assert "StylePackDrift" not in kinds
    assert "WarehouseFootprintThin" not in kinds


def test_ensure_snapshot_material_textures_generates_steel_profiles():
    write_registry()
    snap = {
        "module_placements": [
            {"material_profile": "steel_panel_01"},
            {"material_profile": "steel_door_warehouse_01"},
            {"material_profile": "roof_metal_01"},
        ]
    }
    result = ensure_snapshot_material_textures(snap, size=64)
    assert result["ok"]
    assert material_profiles_in_snapshot(snap) == [
        "roof_metal_01",
        "steel_door_warehouse_01",
        "steel_panel_01",
    ]


def test_keyframe_minimum_matrix_is_24_cells():
    defn = load_building_definition(BDEF)
    cells = expand_bake_matrix_minimum(defn)
    assert len(cells) == MINIMUM_G4_CELLS == 24
    facings = {c.facing for c in cells}
    assert len(facings) == 8
    variants = {c.variant_key for c in cells}
    assert variants == {"clean_day", "clean_night_on", "damaged_night_on"}
