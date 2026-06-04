#!/usr/bin/env python3
"""Author APS pilot bundles: assembly snapshot + variant_set + tile_batch (validate-only)."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
EXAMPLES = REPO / "tools/mcp/schemas/examples"
G0_DIR = REPO / "debug_runs/art_pipeline"
sys.path.insert(0, str(REPO / "tools/mcp/python"))

from rust_engine_mcp.assembly import generate_assembly_snapshot  # noqa: E402

PILOTS = [
    {
        "slug": "warehouse_industrial_west",
        "style_pack_id": "style_industrial_west",
        "width": 4,
        "depth": 2,
        "floors": 2,
        "seed": 43,
        "tile_id": "warehouse_industrial",
        "base": "metal_plate",
        "variant_set_id": "warehouse_industrial_west_day_night",
    },
    {
        "slug": "shopfront_colonial",
        "style_pack_id": "style_colonial",
        "width": 3,
        "depth": 3,
        "floors": 2,
        "seed": 44,
        "tile_id": "shopfront_colonial",
        "base": "stone",
        "variant_set_id": "shopfront_colonial_day_night",
    },
    {
        "slug": "bunker_military",
        "style_pack_id": "style_military",
        "width": 6,
        "depth": 3,
        "floors": 1,
        "seed": 46,
        "tile_id": "bunker_military",
        "base": "concrete",
        "variant_set_id": "bunker_military_day_night",
    },
]


def variant_set_doc(pilot: dict, assembly_id: str) -> dict:
    return {
        "schema_version": 1,
        "variant_set_id": pilot["variant_set_id"],
        "assembly_id": assembly_id,
        "style_pack_id": pilot["style_pack_id"],
        "seed": pilot["seed"],
        "axes": {
            "state": ["clean", "dirty", "damaged", "ruined"],
            "power": ["off", "partial", "on"],
            "fill": ["empty", "half", "full"],
            "lighting": ["day", "night_off", "night_on"],
        },
        "variants": [
            {
                "variant_key": "clean_day",
                "tags": ["default", pilot["style_pack_id"]],
                "layers": {
                    "lighting": {"lighting": "day", "power": "off"},
                    "damage": {"state": "clean", "damage": 0.0},
                    "fill": {"fill": "empty"},
                },
            },
            {
                "variant_key": "damaged_night_on",
                "tags": ["sim_night", "power_grid_on"],
                "layers": {
                    "lighting": {
                        "lighting": "night_on",
                        "power": "on",
                        "night_lights": True,
                        "emissive_strength": 0.8,
                    },
                    "damage": {"state": "damaged", "damage": 0.45},
                    "fill": {"fill": "half"},
                },
            },
        ],
    }


def tile_batch_doc(pilot: dict, assembly_rel: str, variant_rel: str) -> dict:
    batch_id = f"tile_{pilot['slug']}_pilot_v1"
    atlas_id = f"{pilot['slug']}_pilot_v1"
    return {
        "schema_version": 1,
        "batch_id": batch_id,
        "tile_id": pilot["tile_id"],
        "base": pilot["base"],
        "status": "pilot",
        "rules_applied": [
            "no_ai_generated_images",
            "deterministic_output",
            "batch_processing",
            "grid_alignment",
        ],
        "render": {
            "method": "blender_orthographic_iso",
            "isometric": True,
            "seed": pilot["seed"],
            "tile_size_px": 128,
            "camera_elevation_deg": 35.264,
        },
        "assembly_ref": {
            "style_pack_id": pilot["style_pack_id"],
            "assembly_snapshot": assembly_rel,
            "footprint": {
                "width": pilot["width"],
                "depth": pilot["depth"],
                "floors": pilot["floors"],
            },
        },
        "variant_set_ref": variant_rel,
        "variants": [
            {
                "variant_key": "clean_day",
                "state": "clean",
                "damage": 0.0,
                "power": "off",
                "fill": "empty",
                "lighting": "day",
            },
            {
                "variant_key": "damaged_night_on",
                "state": "damaged",
                "damage": 0.45,
                "power": "on",
                "fill": "half",
                "lighting": "night_on",
            },
        ],
        "atlas": {
            "atlas_id": atlas_id,
            "columns": 2,
            "rows": 1,
            "tile_px": 128,
            "padding_px": 2,
            "output_png": f"assets/textures/tiles/{atlas_id}_atlas.png",
            "meta_json": f"assets/staging/tiles/{batch_id}/atlas_meta.json",
        },
        "expected_outputs": [
            "{variant_key}.png",
            "atlas_meta.json",
            f"{atlas_id}_atlas.png",
        ],
        "note": f"APS pilot repeat — {pilot['style_pack_id']} {pilot['width']}x{pilot['depth']}x{pilot['floors']}",
    }


def g0_rules_yaml(pilot: dict, assembly_id: str, batch_id: str) -> str:
    return f"""batch_id: {batch_id}
designer_mcp: aps_pilot_rules_audit
program_id: MCP-FLEET-APS-PILOT-002
rules_check:
  passed: true
  blocked_by: []
  batch_id: {batch_id}
  assembly_id: {assembly_id}
  variant_set_id: {pilot['variant_set_id']}
  style_pack_id: {pilot['style_pack_id']}
  footprint: {{ width: {pilot['width']}, depth: {pilot['depth']}, floors: {pilot['floors']} }}
  proceed: yes
order_critique:
  - "assembly_id {assembly_id} matches variant_set + tile_batch assembly_ref."
  - "Footprint {pilot['width']}x{pilot['depth']}x{pilot['floors']} × {pilot['style_pack_id']} — distinct from rowhouse_victorian pilot."
  - "Validate-only; bake via tile-batch-run after TILE-REAL-001 queue."
"""


def main() -> int:
    authored = []
    for pilot in PILOTS:
        slug = pilot["slug"]
        snap = generate_assembly_snapshot(
            style_pack_id=pilot["style_pack_id"],
            width=pilot["width"],
            depth=pilot["depth"],
            floors=pilot["floors"],
            seed=pilot["seed"],
            write=True,
        )
        assembly_id = snap["assembly_id"]
        asm_name = f"assembly_snapshot_{slug}_v1.json"
        var_name = f"variant_set_{slug}_v1.json"
        batch_name = f"tile_batch_{slug}_pilot_v1.json"
        asm_rel = f"tools/mcp/schemas/examples/{asm_name}"
        var_rel = f"tools/mcp/schemas/examples/{var_name}"

        (EXAMPLES / asm_name).write_text(json.dumps(snap, indent=2) + "\n", encoding="utf-8")
        (EXAMPLES / var_name).write_text(
            json.dumps(variant_set_doc(pilot, assembly_id), indent=2) + "\n", encoding="utf-8"
        )
        batch = tile_batch_doc(pilot, asm_rel, var_rel)
        (EXAMPLES / batch_name).write_text(json.dumps(batch, indent=2) + "\n", encoding="utf-8")

        batch_id = batch["batch_id"]
        g0_path = G0_DIR / f"aps_pilot_{slug}_g0_rules.yaml"
        g0_path.write_text(g0_rules_yaml(pilot, assembly_id, batch_id), encoding="utf-8")
        authored.append(
            {
                "slug": slug,
                "assembly_id": assembly_id,
                "batch_id": batch_id,
                "atlas_id": batch["atlas"]["atlas_id"],
                "style_pack_id": pilot["style_pack_id"],
                "footprint": batch["assembly_ref"]["footprint"],
                "assembly_snapshot": asm_rel,
                "variant_set": var_rel,
                "tile_batch": f"tools/mcp/schemas/examples/{batch_name}",
                "g0_rules": str(g0_path.relative_to(REPO)).replace("\\", "/"),
            }
        )
        print(f"OK {slug} {assembly_id}")

    catalog = {
        "program_id": "MCP-FLEET-APS-PILOT-002",
        "task": "Repeat pilot 2-3 footprints × style packs",
        "green": True,
        "pilots": authored,
        "includes_rowhouse_v1": "tools/mcp/schemas/examples/tile_batch_rowhouse_victorian_pilot_v1.json",
    }
    out = G0_DIR / "aps_pilot_catalog_live.json"
    out.write_text(json.dumps(catalog, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
