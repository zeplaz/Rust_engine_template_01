"""kit_greybox_002 — windows, corners, props (requires module_window + module_prop bpy ops)."""
from __future__ import annotations

import json
from pathlib import Path

SPECS_DIR = Path(__file__).resolve().parent / "specs"
JOBS_DIR = Path(__file__).resolve().parent
SEED = 42

# (asset_id, archetype, style, grid, dims w,h,d, snap, mat)
MODULES = [
    ("window_industrial_1u", "module_window", "style_industrial_west", [1, 1], (4, 1.5, 0.12), "floor_edge", "glass_industrial_01"),
    ("window_shop_1u", "module_window", "style_modern", [1, 1], (4, 2.0, 0.1), "floor_edge", "glass_shop_01"),
    ("window_warehouse_2u", "module_window", "style_industrial_west", [2, 1], (8, 2.5, 0.15), "floor_edge", "glass_warehouse_01"),
    ("window_glass_curtain_2u", "module_window", "style_modern", [2, 1], (8, 3.0, 0.08), "floor_edge", "glass_curtain_01"),
    ("corner_concrete_outer", "module_prop", "style_industrial_west", [1, 1], (2, 3, 2), "floor_edge", "concrete_corner_01"),
    ("corner_brick_outer", "module_prop", "style_rural", [1, 1], (2, 3, 2), "floor_edge", "brick_corner_01"),
    ("corner_steel_inner", "module_prop", "style_industrial_west", [1, 1], (2, 3, 0.15), "floor_edge", "steel_corner_01"),
    ("prop_vent_roof_1u", "module_prop", "style_industrial_west", [1, 1], (2, 0.6, 2), "roof_ridge", "vent_metal_01"),
    ("prop_ac_unit_1u", "module_prop", "style_modern", [1, 1], (1.5, 1.2, 1.5), "roof_ridge", "ac_unit_01"),
    ("corner_wood_porch", "module_prop", "style_rural", [1, 1], (3, 2.5, 3), "floor_edge", "wood_porch_corner_01"),
]


def spec_row(asset_id, archetype, style, grid, dims, snap, mat):
    w, h, d = dims
    return {
        "schema_version": 1,
        "asset_id": asset_id,
        "archetype": archetype,
        "style_pack": style,
        "module": {"grid_units": grid, "snap": snap, "pivot": "bottom_center"},
        "dimensions_m": {"w": w, "h": h, "d": d},
        "material_profile": mat,
        "references": [],
    }


def job_row(asset_id, archetype, dims, mat):
    w, h, d = dims
    jid = f"{asset_id}_run001"
    return {
        "schema_version": 1,
        "job_id": jid,
        "spec_ref": f"assets/staging/specs/{asset_id}.json",
        "operation": archetype,
        "params": {
            "width_m": w,
            "height_m": h,
            "depth_m": d,
            "material_profile": mat,
            "seed": SEED,
        },
        "output": {
            "glb": f"assets/staging/{jid}/model.glb",
            "thumbnail": f"assets/staging/{jid}/preview.png",
        },
    }


def main() -> None:
    SPECS_DIR.mkdir(parents=True, exist_ok=True)
    for row in MODULES:
        aid, archetype, style, grid, dims, snap, mat = row
        spec = spec_row(aid, archetype, style, grid, dims, snap, mat)
        job = job_row(aid, archetype, dims, mat)
        (SPECS_DIR / f"{aid}.json").write_text(json.dumps(spec, indent=2) + "\n", encoding="utf-8")
        (JOBS_DIR / f"{aid}_run001.json").write_text(json.dumps(job, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"batch": "kit_greybox_002", "modules": len(MODULES)}))


if __name__ == "__main__":
    main()
