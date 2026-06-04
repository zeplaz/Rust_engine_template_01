"""Write AssetSpec + geometry job JSON for kit_greybox_001 under assets/staging/."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SPECS_DIR = ROOT / "assets" / "staging" / "specs"
JOBS_DIR = ROOT / "tools" / "mcp" / "schemas" / "examples"
SEED = 42

WALLS = [
    ("wall_concrete_2u", "style_industrial_west", [2, 1], (8, 3, 0.3), "concrete_grey_01", []),
    ("wall_wood_1u", "style_rural", [1, 1], (4, 3, 0.25), "wood_plank_01", []),
    ("wall_steel_1u", "style_industrial_west", [1, 1], (4, 3, 0.15), "steel_panel_01", []),
    ("wall_glass_1u", "style_modern", [1, 1], (4, 3, 0.1), "glass_panel_01", []),
]

ROOFS = [
    ("roof_flat_2u", "style_industrial_west", [2, 2], (8, 0.2, 8), "roof_metal_01", []),
    (
        "roof_pitched_2u",
        "style_rural",
        [2, 2],
        (8, 1.5, 8),
        "roof_tile_01",
        ["greybox:pitched_via_slab"],
    ),
    ("roof_industrial_shed_2u", "style_industrial_west", [2, 3], (8, 0.3, 10), "roof_metal_shed_01", []),
]

DOORS = [
    ("door_industrial_1u", "style_industrial_west", [1, 1], (2, 3, 0.2), "steel_door_01"),
    ("door_warehouse_2u", "style_industrial_west", [2, 1], (4, 4, 0.25), "steel_door_warehouse_01"),
    ("door_shop_1u", "style_modern", [1, 1], (2, 2.5, 0.15), "shop_door_01"),
]


def wall_spec(aid, style, grid, dims, mat, refs):
    w, h, d = dims
    return {
        "schema_version": 1,
        "asset_id": aid,
        "archetype": "module_wall",
        "style_pack": style,
        "module": {"grid_units": grid, "snap": "floor_edge", "pivot": "bottom_center"},
        "dimensions_m": {"w": w, "h": h, "d": d},
        "material_profile": mat,
        "references": refs,
    }


def roof_spec(aid, style, grid, dims, mat, refs):
    w, h, d = dims
    return {
        "schema_version": 1,
        "asset_id": aid,
        "archetype": "module_roof",
        "style_pack": style,
        "module": {"grid_units": grid, "snap": "roof_ridge", "pivot": "bottom_center"},
        "dimensions_m": {"w": w, "h": h, "d": d},
        "material_profile": mat,
        "references": refs,
    }


def door_spec(aid, style, grid, dims, mat):
    w, h, d = dims
    return {
        "schema_version": 1,
        "asset_id": aid,
        "archetype": "module_door",
        "style_pack": style,
        "module": {"grid_units": grid, "snap": "floor_edge", "pivot": "bottom_center"},
        "dimensions_m": {"w": w, "h": h, "d": d},
        "material_profile": mat,
        "references": [],
    }


def wall_job(aid, dims, mat):
    w, h, d = dims
    jid = f"{aid}_run001"
    return {
        "schema_version": 1,
        "job_id": jid,
        "spec_ref": f"assets/staging/specs/{aid}.json",
        "operation": "module_wall",
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


def roof_job(aid, dims, mat):
    w, thick, d = dims
    jid = f"{aid}_run001"
    return {
        "schema_version": 1,
        "job_id": jid,
        "spec_ref": f"assets/staging/specs/{aid}.json",
        "operation": "module_roof",
        "params": {
            "width_m": w,
            "thickness_m": thick,
            "depth_m": d,
            "material_profile": mat,
            "seed": SEED,
        },
        "output": {
            "glb": f"assets/staging/{jid}/model.glb",
            "thumbnail": f"assets/staging/{jid}/preview.png",
        },
    }


def door_job(aid, dims, mat):
    w, h, d = dims
    jid = f"{aid}_run001"
    return {
        "schema_version": 1,
        "job_id": jid,
        "spec_ref": f"assets/staging/specs/{aid}.json",
        "operation": "module_door",
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
    for row in WALLS:
        aid, style, grid, dims, mat, refs = row
        spec = wall_spec(aid, style, grid, dims, mat, refs)
        job = wall_job(aid, dims, mat)
        (SPECS_DIR / f"{aid}.json").write_text(json.dumps(spec, indent=2) + "\n", encoding="utf-8")
        (JOBS_DIR / f"{aid}_run001.json").write_text(json.dumps(job, indent=2) + "\n", encoding="utf-8")
    for row in ROOFS:
        aid, style, grid, dims, mat, refs = row
        spec = roof_spec(aid, style, grid, dims, mat, refs)
        job = roof_job(aid, dims, mat)
        (SPECS_DIR / f"{aid}.json").write_text(json.dumps(spec, indent=2) + "\n", encoding="utf-8")
        (JOBS_DIR / f"{aid}_run001.json").write_text(json.dumps(job, indent=2) + "\n", encoding="utf-8")
    for aid, style, grid, dims, mat in DOORS:
        spec = door_spec(aid, style, grid, dims, mat)
        job = door_job(aid, dims, mat)
        (SPECS_DIR / f"{aid}.json").write_text(json.dumps(spec, indent=2) + "\n", encoding="utf-8")
        (JOBS_DIR / f"{aid}_run001.json").write_text(json.dumps(job, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"written_specs": len(WALLS) + len(ROOFS) + len(DOORS), "specs_dir": str(SPECS_DIR)}))


if __name__ == "__main__":
    main()
