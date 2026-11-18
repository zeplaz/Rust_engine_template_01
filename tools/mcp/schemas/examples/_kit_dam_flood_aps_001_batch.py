"""One-shot batch: AssetSpecs + geometry jobs for kit_dam_flood_aps_001."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
SPECS_DIR = ROOT / "assets" / "staging" / "specs"
JOBS_DIR = Path(__file__).resolve().parent
BATCH = "kit_dam_flood_aps_001"
SEED = 42
# 4 m per tile (same as wall 1u modules).
TILE_M = 4.0

# (asset_id, grid_x, grid_y, height_tiles, prop_kind, seed_offset)
MODULES = [
    ("dam_embankment_v1", 12, 2, 3, "dam_embankment", 0),
    ("dam_gravity_v1", 10, 2, 4, "dam_gravity", 1),
    ("dam_weir_v1", 6, 1, 1, "dam_weir", 2),
    ("dam_arch_v1", 6, 2, 5, "dam_arch", 3),
]


def main() -> None:
    SPECS_DIR.mkdir(parents=True, exist_ok=True)
    for asset_id, gx, gy, ht, kind, off in MODULES:
        w = gx * TILE_M
        d = gy * TILE_M
        h = max(ht * TILE_M * 0.75, 2.0)
        job_id = f"{asset_id}_production_run001"
        spec = {
            "schema_version": 1,
            "asset_id": asset_id,
            "archetype": "module_prop",
            "style_pack": "style_industrial_west",
            "development_tier": "production",
            "place_feel_claim": "operational_envelope",
            "pbr_status": "shipped",
            "batch_id": BATCH,
            "module": {
                "grid_units": [gx, gy],
                "snap": "floor_edge",
                "pivot": "bottom_center",
            },
            "dimensions_m": {"w": w, "h": h, "d": d},
            "material_profile": "concrete_grey_01",
            "utility_role": "dam",
            "catalog_id_planned": asset_id,
            "feeds": BATCH,
            "references": [
                "ref:plan:plan_dam_flood_benchmark_v1",
                "ref:park:dam_flood_aps_park_v1",
                f"ref:kit:{BATCH}",
                f"ref:catalog:assets/configs/buildings/{asset_id}.json",
            ],
        }
        spec_path = SPECS_DIR / f"{asset_id}_production.json"
        spec_path.write_text(json.dumps(spec, indent=2) + "\n", encoding="utf-8")

        job = {
            "schema_version": 1,
            "job_id": job_id,
            "batch_id": BATCH,
            "development_tier": "production",
            "place_feel_claim": "operational_envelope",
            "spec_ref": f"assets/staging/specs/{asset_id}_production.json",
            "operation": "module_prop",
            "params": {
                "width_m": w,
                "height_m": h,
                "depth_m": d,
                "material_profile": "concrete_grey_01",
                "seed": SEED + off,
                "prop_kind": kind,
                "name": asset_id,
            },
            "output": {
                "glb": f"assets/staging/{job_id}/model.glb",
                "thumbnail": f"assets/staging/{job_id}/preview.png",
            },
        }
        job_path = JOBS_DIR / f"{job_id}.json"
        job_path.write_text(json.dumps(job, indent=2) + "\n", encoding="utf-8")
        print(f"wrote {spec_path.relative_to(ROOT)} + {job_path.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
