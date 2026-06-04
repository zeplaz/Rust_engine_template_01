#!/usr/bin/env python3
"""MCP-PROD-TILE-001 + MCP-PROD-INDEX-001 — rowhouse production atlas finalize."""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools" / "mcp" / "python"))

BATCH_ID = "tile_rowhouse_victorian_production_v1"
BATCH_JSON = ROOT / "tools/mcp/schemas/examples/tile_batch_rowhouse_victorian_production_v1.json"
STAGING = ROOT / "assets/staging/tiles" / BATCH_ID
ATLAS_PNG = ROOT / "assets/textures/buildings_iso/production/rowhouse_victorian_production_v1_atlas.png"


def restore_clean_day_from_atlas() -> str:
    """Restore missing clean_day.png from packed atlas slot (0,0)."""
    from PIL import Image

    if not ATLAS_PNG.is_file():
        raise FileNotFoundError(ATLAS_PNG)
    meta_path = STAGING / "atlas_meta.json"
    cols, rows, tile_px = 4, 4, 128
    if meta_path.is_file():
        meta = json.loads(meta_path.read_text(encoding="utf-8"))
        cols = int(meta.get("columns") or cols)
        rows = int(meta.get("rows") or rows)
        tile_px = int(meta.get("tile_px") or tile_px)
        for tile in meta.get("tiles") or []:
            if tile.get("variant_key") == "clean_day":
                col, row = tile.get("grid") or [0, 0]
                break
        else:
            col, row = 0, 0
    else:
        col, row = 0, 0

    with Image.open(ATLAS_PNG) as atlas:
        w, h = atlas.size
        cell_w = w // cols
        cell_h = h // rows
        left = col * cell_w
        upper = row * cell_h
        tile = atlas.crop((left, upper, left + min(tile_px, cell_w), upper + min(tile_px, cell_h)))
        out = STAGING / "clean_day.png"
        STAGING.mkdir(parents=True, exist_ok=True)
        tile.save(out)
    return str(out)


def run_tile_batch() -> dict:
    os.environ["RUST_ENGINE_TILE_DRY_RUN"] = "0"
    from rust_engine_mcp.tile_pipeline import tile_batch_run

    return tile_batch_run(BATCH_JSON)


def run_index_register() -> dict:
    from rust_engine_mcp.tile_index import register_tile_atlas_from_batch

    return register_tile_atlas_from_batch(
        BATCH_ID,
        tile_batch_path=BATCH_JSON,
    )


def write_witnesses(tile_result: dict, index_result: dict) -> dict:
    from rust_engine_mcp import witness
    from rust_engine_mcp.schemas import load_json_file

    batch = load_json_file(BATCH_JSON)
    tile_wit = witness.write_tile_batch_witness(
        BATCH_ID,
        batch=batch,
        png_count=len(tile_result.get("png_paths") or []),
        atlas_path=tile_result.get("atlas_path"),
        meta_path=tile_result.get("meta_json"),
        dry_run=bool(tile_result.get("dry_run")),
        tile_index=index_result,
    )
    prod_wit = witness.write_procedural_tiles_production_bake_witness()
    return {"tile_batch_witness": tile_wit, "production_bake_witness": prod_wit}


def main() -> None:
    restored = restore_clean_day_from_atlas()
    print(f"restored: {restored}")
    tile_result = run_tile_batch()
    print(json.dumps({"tile_batch_run": tile_result}, indent=2)[:6000])
    if not tile_result.get("ok"):
        raise SystemExit(1)
    index_result = run_index_register()
    print(json.dumps({"tile_atlas_index": index_result}, indent=2))
    witnesses = write_witnesses(tile_result, index_result)
    rowhouse = next(
        (a for a in witnesses["production_bake_witness"].get("atlases") or [] if a.get("batch_id") == BATCH_ID),
        None,
    )
    print(json.dumps({"witnesses": witnesses, "rowhouse_pass": rowhouse}, indent=2)[:4000])


if __name__ == "__main__":
    main()
