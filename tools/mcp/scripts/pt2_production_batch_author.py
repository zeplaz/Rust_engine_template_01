#!/usr/bin/env python3
"""PT-2 — author production tile_batch + variant_set from variant_matrix YAML (4 pilots)."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
EXAMPLES = REPO / "tools/mcp/schemas/examples"
MATRIX_DIR = REPO / "debug_runs/art_pipeline"
sys.path.insert(0, str(REPO / "tools/mcp/python"))

from rust_engine_mcp.assembly import generate_assembly_snapshot  # noqa: E402
from rust_engine_mcp.variant_matrix_expand import (  # noqa: E402
    expand_matrix_to_tile_batch,
    variant_set_rows,
)

PILOTS = [
    {
        "slug": "rowhouse_victorian",
        "matrix": "variant_matrix_rowhouse_v1.yaml",
        "style_pack_id": "style_victorian",
        "base": "stone",
        "seed": 42,
        "assembly_snapshot": "tools/mcp/schemas/examples/assembly_snapshot_rowhouse_victorian_v1.json",
    },
    {
        "slug": "warehouse_industrial_west",
        "matrix": "variant_matrix_warehouse_v1.yaml",
        "style_pack_id": "style_industrial_west",
        "base": "metal_plate",
        "seed": 43,
        "assembly_snapshot": "tools/mcp/schemas/examples/assembly_snapshot_warehouse_industrial_west_v1.json",
    },
    {
        "slug": "shopfront_colonial",
        "matrix": "variant_matrix_shopfront_v1.yaml",
        "style_pack_id": "style_colonial",
        "base": "stone",
        "seed": 44,
        "assembly_snapshot": "tools/mcp/schemas/examples/assembly_snapshot_shopfront_colonial_v1.json",
    },
    {
        "slug": "bunker_military",
        "matrix": "variant_matrix_bunker_v1.yaml",
        "style_pack_id": "style_military",
        "base": "concrete",
        "seed": 46,
        "assembly_snapshot": "tools/mcp/schemas/examples/assembly_snapshot_bunker_military_v1.json",
    },
]


def main() -> int:
    authored = []
    for pilot in PILOTS:
        matrix_path = MATRIX_DIR / pilot["matrix"]
        batch = expand_matrix_to_tile_batch(
            matrix_path,
            pilot_slug=pilot["slug"],
            style_pack_id=pilot["style_pack_id"],
            assembly_snapshot_rel=pilot["assembly_snapshot"],
            base=pilot["base"],
            seed=pilot["seed"],
            minimum_only=False,
            include_fire_row=True,
        )
        keys = [v["variant_key"] for v in batch["variants"]]
        asm_path = REPO / pilot["assembly_snapshot"]
        if asm_path.is_file():
            assembly_id = json.loads(asm_path.read_text(encoding="utf-8"))["assembly_id"]
        else:
            snap = generate_assembly_snapshot(
                style_pack_id=pilot["style_pack_id"],
                width=batch["assembly_ref"]["footprint"]["width"],
                depth=batch["assembly_ref"]["footprint"]["depth"],
                floors=batch["assembly_ref"]["footprint"]["floors"],
                seed=pilot["seed"],
                write=True,
            )
            assembly_id = snap["assembly_id"]
            asm_path.write_text(json.dumps(snap, indent=2) + "\n", encoding="utf-8")

        variant_set = {
            "schema_version": 1,
            "variant_set_id": f"{pilot['slug']}_production_v1",
            "assembly_id": assembly_id,
            "style_pack_id": pilot["style_pack_id"],
            "seed": pilot["seed"],
            "variants": variant_set_rows(keys),
        }
        batch_name = f"tile_batch_{pilot['slug']}_production_v1.json"
        var_name = f"variant_set_{pilot['slug']}_production_v1.json"
        (EXAMPLES / batch_name).write_text(json.dumps(batch, indent=2) + "\n", encoding="utf-8")
        (EXAMPLES / var_name).write_text(json.dumps(variant_set, indent=2) + "\n", encoding="utf-8")
        authored.append(
            {
                "slug": pilot["slug"],
                "batch_id": batch["batch_id"],
                "atlas_id": batch["atlas"]["atlas_id"],
                "variant_count": len(keys),
                "assembly_id": assembly_id,
                "tile_batch": f"tools/mcp/schemas/examples/{batch_name}",
                "variant_set": f"tools/mcp/schemas/examples/{var_name}",
            }
        )
        print(f"OK {pilot['slug']} {len(keys)} variants -> {batch_name}")

    catalog = {
        "program_id": "MCP-PT-2-001",
        "task": "production batch author",
        "green": True,
        "pilots": authored,
    }
    out = REPO / "debug_runs/art_pipeline/pt2_production_batch_catalog_live.json"
    out.write_text(json.dumps(catalog, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
