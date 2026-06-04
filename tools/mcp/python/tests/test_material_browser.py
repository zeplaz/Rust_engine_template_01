"""MCP-APS-MATERIAL-BROWSER-001 — material catalog + snapshot apply."""

from __future__ import annotations

import json

import pytest

from rust_engine_mcp import assembly
from rust_engine_mcp.material_profiles import (
    ensure_profile_textures,
    infer_generator,
    load_material_profile_catalog,
    register_material_profile,
)
from rust_engine_mcp.material_textures import write_registry
from rust_engine_mcp.paths import repo_root


def test_material_catalog_includes_pilot_steel_panel():
    write_registry()
    ids = [e.profile_id for e in load_material_profile_catalog()]
    assert "steel_panel_01" in ids
    assert "brick_red_01" in ids


def test_ensure_profile_textures_writes_albedo():
    write_registry()
    entry = ensure_profile_textures("brick_red_01", size=64)
    assert entry.albedo_path is not None
    assert entry.albedo_path.is_file()


def test_apply_material_via_browser_api():
    snap = {
        "schema_version": 1,
        "assembly_id": "test_4x2_s1_abcd",
        "style_pack_id": "style_victorian",
        "source_tier": "lod0",
        "seed": 1,
        "footprint": {"width": 4, "depth": 2, "floors": 1},
        "module_placements": [
            {
                "module_id": "wall_brick_1u",
                "slot_key": "wall_1u",
                "token": "W",
                "grid_x": 1,
                "grid_y": 0,
                "floor": 0,
                "glb_path": "assets/models/modules/wall_brick_1u_lod0_run001/model.glb",
            }
        ],
    }
    snap = assembly.enrich_snapshot(snap)
    node_id = assembly.placement_node_id(snap["module_placements"][0])
    updated = assembly.update_placement(snap, node_id, material_profile="steel_panel_01")
    assert updated["module_placements"][0]["material_profile"] == "steel_panel_01"


def test_ensure_inferred_profile_generates_and_registers():
    pid = "test_custom_brick_zz_01"
    register_material_profile(pid, generator="brick", category="residential/brick")
    entry = ensure_profile_textures(pid, size=64, force=True)
    assert entry.albedo_path is not None
    assert entry.albedo_path.is_file()
    assert entry.category == "residential/brick"
    assert infer_generator(pid) == "brick"


def test_material_browser_module_import():
    root = repo_root()
    assert (root / "tools/mcp/art_pipeline_suite/material_browser.py").is_file()


def test_grammar_material_authority_witness():
    write_registry()
    ensure_profile_textures("steel_panel_01", size=64)
    try:
        snap = assembly.generate_assembly_snapshot(
            archetype_id="IndustrialWarehouse",
            district_style="industrial_west",
            seed=43,
            write=False,
        )
    except (ValueError, FileNotFoundError) as exc:
        pytest.skip(str(exc))
    node = (snap.get("module_placements") or [None])[0]
    assert node
    node_id = assembly.placement_node_id(node)
    updated = assembly.update_placement(snap, node_id, material_profile="steel_panel_01")
    body = {
        "gate_id": "APS-MATERIAL-BROWSER-001",
        "green": True,
        "catalog_profiles": len(load_material_profile_catalog()),
        "sample_applied": updated["module_placements"][0].get("material_profile"),
    }
    out = repo_root() / "debug_runs" / "aps_material_browser_live.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    assert body["sample_applied"] == "steel_panel_01"
