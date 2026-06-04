#!/usr/bin/env python3
"""PT-2 dev bridge — ortho headless seeds staging PNGs before keyframe_pack register.

Production ship batches require `bake_source: keyframe_pack`. Until artist keyframe stills
exist, this script runs `smoke_ortho_headless` into the same staging folder, then you can
run `tile-batch-run` on the production JSON (keyframe_pack) to pack + register.

  $env:RUST_ENGINE_TILE_DRY_RUN = '0'
  python tools/mcp/scripts/pt2_production_ortho_seed.py
  python -m rust_engine_mcp.cli tile-batch-run tools/mcp/schemas/examples/tile_batch_rowhouse_victorian_production_v1.json
"""

from __future__ import annotations

import json
import sys
from copy import deepcopy
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
EXAMPLES = REPO / "tools/mcp/schemas/examples"
sys.path.insert(0, str(REPO / "tools/mcp/python"))

from rust_engine_mcp.tile_pipeline import tile_batch_run  # noqa: E402

BATCHES = [
    "tile_batch_rowhouse_victorian_production_v1.json",
    "tile_batch_warehouse_industrial_west_production_v1.json",
    "tile_batch_shopfront_colonial_production_v1.json",
    "tile_batch_bunker_military_production_v1.json",
]


def main() -> int:
    results = []
    for name in BATCHES:
        src = EXAMPLES / name
        batch = json.loads(src.read_text(encoding="utf-8"))
        seed_doc = deepcopy(batch)
        seed_doc["ship"] = False
        seed_doc["bake_source"] = "smoke_ortho_headless"
        render = dict(seed_doc.get("render") or {})
        render["method"] = "blender_orthographic_iso"
        render.setdefault("isometric", True)
        render.setdefault("tile_size_px", 128)
        render.setdefault("camera_elevation_deg", 35.264)
        seed_doc["render"] = render
        tmp = EXAMPLES / f".{name}.ortho_seed.json"
        tmp.write_text(json.dumps(seed_doc, indent=2) + "\n", encoding="utf-8")
        print(f"Ortho seed {batch['batch_id']} ({len(batch.get('variants') or [])} variants)...")
        result = tile_batch_run(tmp)
        tmp.unlink(missing_ok=True)
        results.append({"batch_id": batch["batch_id"], "ok": result.get("ok"), "status": result.get("status")})
        print(json.dumps({k: result.get(k) for k in ("ok", "status", "variant_count", "atlas_path")}, indent=2))
        if not result.get("ok"):
            return 1
    catalog = {"program_id": "MCP-PT-2-ORTHO-SEED", "green": True, "batches": results}
    out = REPO / "debug_runs/art_pipeline/pt2_production_ortho_seed_live.json"
    out.write_text(json.dumps(catalog, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
