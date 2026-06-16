"""ARCH-MAT-001 — material_profile on every bake job path."""

from __future__ import annotations

import json
from pathlib import Path

from rust_engine_mcp.material_authority import annotate_tile_bake_job
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.tile_compile_loop import keyframe_job_for_cell
from rust_engine_mcp.building_definition import BakeCell, load_building_definition


WAREHOUSE_BDEF = (
    repo_root()
    / "tools/mcp/schemas/examples/building_definition_warehouse_industrial_west_production_v1.json"
)


def test_annotate_tile_bake_job_injects_snapshot():
    snap = repo_root() / "tools/mcp/schemas/examples/assembly_snapshot_warehouse_industrial_west_production_v1.json"
    job = annotate_tile_bake_job(
        {"operation": "tile_variant_bake", "mode": "assembly"},
        snapshot_path=snap,
    )
    assert job.get("ok", True) is not False
    assert "assembly_snapshot" in job
    assert job["assembly_snapshot"].endswith(".json")
    assert job.get("material_authority") == "snapshot material_profile"


def test_keyframe_job_carries_assembly_snapshot():
    defn = load_building_definition(WAREHOUSE_BDEF)
    cells = __import__(
        "rust_engine_mcp.building_definition", fromlist=["expand_bake_matrix_minimum"]
    ).expand_bake_matrix_minimum(defn)
    job = keyframe_job_for_cell(defn, cells[0], light_blend="utils/Tile_iso_rig_v1.blend")
    assert job.get("assembly_snapshot")
    assert not job.get("_material_prep_failed")


def test_bpy_material_authority_module_present():
    path = repo_root() / "tools/mcp/blender/scripts/ops/material_authority.py"
    text = path.read_text(encoding="utf-8")
    assert "apply_snapshot_material_profiles" in text
    assert "apply_from_job" in text


def test_tile_keyframe_bake_calls_material_authority():
    path = repo_root() / "tools/mcp/blender/scripts/ops/tile_keyframe_bake.py"
    assert "apply_from_job" in path.read_text(encoding="utf-8")
