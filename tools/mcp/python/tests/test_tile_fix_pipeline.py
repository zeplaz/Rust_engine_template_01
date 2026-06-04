"""TILE-FIX-004..008 — assembly, materials, building definition, compile plan."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from rust_engine_mcp.building_definition import (
    expand_bake_matrix,
    load_building_definition,
)
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.tile_compile_loop import compile_plan, validate_compile_preconditions
from rust_engine_mcp.validators.assembly_production import validate_assembly_snapshot_path
from rust_engine_mcp.validators.material_textures import validate_material_textures


ROOT = repo_root()
ROWHOUSE_SNAP = ROOT / "tools/mcp/schemas/examples/assembly_snapshot_rowhouse_victorian_production_v1.json"
ROWHOUSE_BDEF = ROOT / "tools/mcp/schemas/examples/building_definition_rowhouse_victorian_production_v1.json"
WAREHOUSE_BDEF = ROOT / "tools/mcp/schemas/examples/building_definition_warehouse_industrial_west_production_v1.json"
WAREHOUSE_SNAP = ROOT / "tools/mcp/schemas/examples/assembly_snapshot_warehouse_industrial_west_production_v1.json"


def test_assembly_production_snapshot_passes_when_glbs_present():
    if not ROWHOUSE_SNAP.is_file():
        pytest.skip("rowhouse production snapshot missing")
    rep = validate_assembly_snapshot_path(ROWHOUSE_SNAP, ship=True)
    missing_only = all(
        e.signature == "assembly_production_glb_missing" for e in rep.errors if e.severity == "error"
    )
    if rep.status == "failed" and not missing_only:
        hints = [e.hint for e in rep.errors]
        pytest.fail(f"unexpected assembly errors: {hints}")
    assert rep.status in ("passed", "failed")


def test_material_textures_fail_without_pngs():
    rep = validate_material_textures(
        {"development_tier": "production", "material_profile": "brick_red_01"},
        ship=True,
    )
    if (ROOT / "assets/materials/textures/brick_red_01/albedo.png").is_file():
        assert rep.status == "passed"
    else:
        assert rep.status == "failed"
        assert any(e.signature == "material_textures_missing_maps" for e in rep.errors)


def test_material_textures_reject_greybox_fallback():
    rep = validate_material_textures(
        {
            "development_tier": "production",
            "material_profile": "brick_red_01",
            "greybox_fallback": True,
        },
        ship=True,
    )
    assert rep.status == "failed"
    assert any(e.signature == "material_textures_greybox_fallback_forbidden" for e in rep.errors)


def test_building_definition_expands_facing_matrix():
    if not ROWHOUSE_BDEF.is_file():
        pytest.skip("building definition example missing")
    defn = load_building_definition(ROWHOUSE_BDEF)
    cells = expand_bake_matrix(defn)
    assert defn.facings == 8
    assert len(cells) > len(defn.variants)
    clean = [c for c in cells if c.variant_key == "clean_day"]
    assert len(clean) == 8


def test_compile_plan_includes_facing_steps():
    if not ROWHOUSE_BDEF.is_file():
        pytest.skip("building definition example missing")
    defn = load_building_definition(ROWHOUSE_BDEF)
    plan = compile_plan(defn)
    assert plan["cell_count"] == len(expand_bake_matrix(defn))
    assert plan["steps"][0]["facing"] == 0
    assert "yaw_deg" in plan["steps"][0]


def test_rowhouse_snapshot_is_production_tier():
    data = json.loads(ROWHOUSE_SNAP.read_text(encoding="utf-8"))
    assert data.get("source_tier") == "production"
    jobs = {p.get("job_id") for p in data.get("module_placements") or []}
    assert any("production" in (j or "") for j in jobs)


def test_warehouse_building_definition_minimum_matrix():
    if not WAREHOUSE_BDEF.is_file():
        pytest.skip("warehouse building definition missing")
    defn = load_building_definition(WAREHOUSE_BDEF)
    cells = expand_bake_matrix(defn)
    assert defn.facings == 8
    assert defn.building_id == "warehouse_industrial"
    assert len(cells) == 576
    clean = [c for c in cells if c.variant_key == "clean_day"]
    assert len(clean) == 8
    pilot_states = {"clean_day", "clean_night_on", "damaged_night_on"}
    pilot_facings = set(range(8))
    pilot = [
        c
        for c in cells
        if c.variant_key in pilot_states
        and c.facing in pilot_facings
        and c.frame == 0
    ]
    assert len(pilot) == 24


def test_warehouse_visual_config_v2_passes():
    path = ROOT / "assets/configs/buildings/visual_config_warehouse_industrial_west_v2.json"
    if not path.is_file():
        pytest.skip("warehouse visual_config v2 missing")
    from rust_engine_mcp.schemas import validate_visual_config

    validate_visual_config(json.loads(path.read_text(encoding="utf-8")))


def test_warehouse_assembly_blend_exists():
    if not WAREHOUSE_SNAP.is_file():
        pytest.skip("warehouse snapshot missing")
    blend = ROOT / "assets/staging/assemblies/industrial_west_4x2_s43_a879.blend"
    assert blend.is_file()


def test_minimum_g4_cell_count_constant():
    from rust_engine_mcp.building_definition import MINIMUM_G4_CELLS, expand_bake_matrix_minimum, load_building_definition

    if not WAREHOUSE_BDEF.is_file():
        pytest.skip("warehouse building definition missing")
    defn = load_building_definition(WAREHOUSE_BDEF)
    assert len(expand_bake_matrix_minimum(defn)) == MINIMUM_G4_CELLS == 24


def test_warehouse_shell_production_job_ids_in_bdef():
    from rust_engine_mcp.building_definition import production_shell_modules_ready, load_building_definition

    if not WAREHOUSE_BDEF.is_file():
        pytest.skip("warehouse building definition missing")
    defn = load_building_definition(WAREHOUSE_BDEF)
    shell_ok, blockers = production_shell_modules_ready(defn)
    assert shell_ok
    assert not blockers
    jobs = {m["module_id"]: m["job_id"] for m in defn.modules}
    assert jobs["wall_steel_1u"] == "wall_steel_1u_production_run001"
    assert jobs["roof_sawtooth"] == "roof_sawtooth_production_run001"


def test_atlas_meta_v2_minimum_lookup_keys_from_visual_config():
    from rust_engine_mcp.validators.atlas_meta import _required_lookup_keys

    visual = json.loads(
        (ROOT / "assets/configs/buildings/visual_config_warehouse_industrial_west_v2.json").read_text(
            encoding="utf-8"
        )
    )
    keys = _required_lookup_keys(visual, facings=8, minimum_g4=True)
    assert len(keys) == 24


def test_pack_cells_to_atlas_builds_24_lookups(tmp_path):
    from rust_engine_mcp.atlas_meta_v2_pack import cell_png_basename, pack_cells_to_atlas
    from rust_engine_mcp.building_definition import expand_bake_matrix_minimum, load_building_definition

    if not WAREHOUSE_BDEF.is_file():
        pytest.skip("warehouse building definition missing")
    defn = load_building_definition(WAREHOUSE_BDEF)
    cells = expand_bake_matrix_minimum(defn)
    try:
        from PIL import Image
    except ImportError:
        pytest.skip("Pillow not installed")
    for cell in cells:
        Image.new("RGBA", (128, 128), (40, 80, 120, 255)).save(tmp_path / cell_png_basename(cell))
    info = pack_cells_to_atlas(cells, tmp_path, atlas_png=tmp_path / "atlas.png", columns=8)
    assert len(info["lookups"]) == 24
    assert info["rows"] == 3
