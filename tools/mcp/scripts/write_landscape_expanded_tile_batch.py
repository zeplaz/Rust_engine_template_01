#!/usr/bin/env python3
"""Emit tile_batch_landscape_expanded_v1.json — DMCP-TILE-BATCH-EXPAND-SPEC-001."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / "assets/staging/specs/tile_batch_landscape_expanded_v1.json"

ROWS = [
    ("topology_patch", "Patch", "clean", 0.0, "off", "full", "day", None),
    ("topology_patch_scar", "Patch", "damaged", 0.35, "off", "empty", "day", None),
    ("topology_patch_burn_00", "Patch", "damaged", 0.55, "on", "half", "night_on", 0),
    ("topology_patch_burn_04", "Patch", "damaged", 0.63, "on", "half", "night_on", 4),
    ("topology_patch_regrowth_grass", "Patch", "clean", 0.05, "off", "quarter", "day", None),
    ("topology_patch_regrowth_shrub", "Patch", "clean", 0.1, "off", "half", "day", None),
    ("topology_corridor", "Corridor", "clean", 0.0, "off", "half", "day", None),
    ("topology_corridor_scar", "Corridor", "damaged", 0.35, "off", "empty", "day", None),
    ("topology_corridor_burn_00", "Corridor", "damaged", 0.55, "on", "half", "night_on", 0),
    ("topology_corridor_burn_04", "Corridor", "damaged", 0.63, "on", "half", "night_on", 4),
    ("topology_ring", "Ring", "clean", 0.0, "off", "quarter", "day", None),
    ("topology_ring_burn_00", "Ring", "damaged", 0.55, "on", "half", "night_on", 0),
    ("topology_cluster", "Cluster", "clean", 0.0, "off", "half", "day", None),
    ("topology_cluster_regrowth_grass", "Cluster", "clean", 0.08, "off", "quarter", "day", None),
    ("topology_fringe", "Fringe", "clean", 0.0, "off", "empty", "day", None),
    ("topology_fringe_regrowth_grass", "Fringe", "clean", 0.05, "off", "quarter", "day", None),
]

assert len(ROWS) == 16, len(ROWS)

variants = []
for key, topo, state, damage, power, fill, lighting, fire_frame in ROWS:
    row = {
        "variant_key": key,
        "state": state,
        "damage": damage,
        "power": power,
        "fill": fill,
        "lighting": lighting,
        "topology_kind": topo,
    }
    if fire_frame is not None:
        row["fire_frame"] = fire_frame
    variants.append(row)

batch = {
    "_meta": {
        "teaches": ["landscape_lg5_expanded", "burn_scar_regrowth"],
        "not_a_ship_target": True,
        "charter": "src/dev/design_landscape_lg5_expansion_matrix_v1.md",
        "designer_mcp": "DMCP-TILE-BATCH-EXPAND-SPEC-001",
        "pilot_reuse": "tools/mcp/schemas/examples/tile_batch_landscape_lg5_pilot_v1.json",
    },
    "schema_version": 1,
    "batch_id": "tile_landscape_expanded_v1",
    "tile_id": "landscape_lg5",
    "base": "dirt",
    "status": "expanded_pilot",
    "ship": False,
    "frozen": False,
    "bake_source": "keyframe_pack",
    "source_tier": "pilot",
    "development_tier": "pilot",
    "atlas_domain": "landscape",
    "landscape_program_ref": "assets/configs/landscape/presets/fire_recovery_v0.json",
    "matrix_ref": "src/dev/design_landscape_lg5_expansion_matrix_v1.md",
    "keyframe_rename_pk": True,
    "rules_applied": [
        "no_ai_generated_images",
        "deterministic_output",
        "batch_processing",
        "grid_alignment",
    ],
    "render": {
        "method": "blender_keyframe_light_rig",
        "isometric": True,
        "seed": 550005,
        "tile_size_px": 64,
        "light_blend": "utils/Tile_iso_rig_v1.blend",
    },
    "variants": variants,
    "atlas": {
        "atlas_id": "landscape_lg5_expanded_v1",
        "columns": 4,
        "rows": 4,
        "tile_px": 64,
        "padding_px": 2,
        "output_png": "assets/textures/landscape/staging/landscape_lg5_expanded_v1_atlas.png",
        "meta_json": "assets/staging/tiles/tile_landscape_expanded_v1/atlas_meta.json",
    },
    "expected_outputs": ["{variant_key}.png", "atlas_meta.json", "landscape_lg5_expanded_v1_atlas.png"],
    "pre_baked_folder": "assets/staging/tiles/keyframe_stills/tile_landscape_expanded_v1",
    "note": "Expanded LG-5 — 16 topology×state cells per designer-mcp matrix charter. ship:false until G4 manual keyframes.",
}

OUT.parent.mkdir(parents=True, exist_ok=True)
OUT.write_text(json.dumps(batch, indent=2) + "\n", encoding="utf-8")
print(f"wrote {OUT} variants={len(variants)}")
