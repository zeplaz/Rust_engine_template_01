"""ARCH-001 / ARCH-003 — assembly graph nodes on snapshots."""

from __future__ import annotations

import pytest

from rust_engine_mcp import assembly
from rust_engine_mcp.schemas import validate_assembly_snapshot


def test_enrich_placement_adds_material_profile_from_index():
    placement = {
        "module_id": "wall_brick_1u",
        "job_id": "wall_brick_1u_lod0_run001",
        "slot_key": "wall_1u",
        "token": "W",
        "grid_x": 1,
        "grid_y": 0,
        "floor": 0,
        "glb_path": "assets/models/modules/wall_brick_1u_lod0_run001/model.glb",
    }
    enriched = assembly.enrich_placement(placement, source_tier="lod0")
    assert enriched.get("node_id")
    assert enriched.get("material_profile")
    assert enriched.get("placement_tags")
    assert enriched.get("lod_policy")


def test_generate_snapshot_validates_with_graph_fields():
    try:
        snap = assembly.generate_assembly_snapshot(
            style_pack_id="style_victorian",
            width=4,
            depth=3,
            floors=2,
            seed=42,
            write=False,
        )
    except (ValueError, FileNotFoundError) as exc:
        pytest.skip(str(exc))
    validate_assembly_snapshot(snap)
    for p in snap["module_placements"]:
        assert p.get("node_id")
        assert p.get("material_profile")


def test_update_placement_material():
    snap = {
        "schema_version": 1,
        "assembly_id": "test_4x3_s1_abcd",
        "style_pack_id": "style_victorian",
        "source_tier": "lod0",
        "seed": 1,
        "footprint": {"width": 4, "depth": 3, "floors": 2},
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
    updated = assembly.update_placement(
        snap,
        node_id,
        material_profile="brick_red_01",
        semantic_tags={"location": ["street_facing"], "architectural": ["residential"]},
    )
    p = updated["module_placements"][0]
    assert p["material_profile"] == "brick_red_01"
    assert p.get("semantic_tags")
    assert "street_facing" in p["semantic_tags"].get("location", [])
