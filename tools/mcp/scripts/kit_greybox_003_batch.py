"""Write AssetSpec + geometry job JSON for kit_greybox_003 — kit gaps not in _module_index."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SPECS_DIR = ROOT / "assets" / "staging" / "specs"
JOBS_DIR = ROOT / "tools" / "mcp" / "schemas" / "examples"
SEED = 42

# (asset_id, archetype, style, grid, dims w,h,d, snap, mat, kit_ref, refs)
MODULES = [
    ("wall_brick_2u", "module_wall", "style_industrial_west", [2, 1], (8, 3, 0.3), "floor_edge", "brick_red_01", "wall_brick_2u", []),
    ("wall_concrete_1u", "module_wall", "style_industrial_west", [1, 1], (4, 3, 0.3), "floor_edge", "concrete_grey_01", "wall_concrete_1u", []),
    ("wall_wood_2u", "module_wall", "style_rural", [2, 1], (8, 3, 0.25), "floor_edge", "wood_plank_01", "wall_wood_2u", []),
    ("wall_industrial_panel_2u", "module_wall", "style_industrial_west", [2, 1], (8, 3, 0.12), "floor_edge", "steel_panel_01", "wall_industrial_panel_2u", []),
    ("window_single_1u", "module_window", "style_rural", [1, 1], (4, 1.2, 0.1), "floor_edge", "glass_shop_01", "win_single_1u", []),
    ("window_double_1u", "module_window", "style_modern", [1, 1], (4, 1.5, 0.1), "floor_edge", "glass_shop_01", "win_double_1u", []),
    ("door_residential_1u", "module_door", "style_rural", [1, 1], (1.2, 2.4, 0.12), "floor_edge", "wood_plank_01", "door_residential", []),
    ("door_garage_2u", "module_door", "style_industrial_west", [2, 1], (4, 3, 0.2), "floor_edge", "steel_door_01", "door_garage", []),
    ("roof_sawtooth_2u", "module_roof", "style_industrial_west", [2, 2], (8, 0.4, 8), "roof_ridge", "roof_metal_01", "roof_sawtooth", ["greybox:sawtooth_via_slab"]),
    ("prop_chimney_1u", "module_prop", "style_rural", [1, 1], (1.5, 4, 1.5), "roof_ridge", "brick_red_01", "prop_chimney", []),
]


def spec_row(asset_id, archetype, style, grid, dims, snap, mat, kit_ref, refs):
    w, h, d = dims
    refs_out = list(refs)
    if kit_ref:
        refs_out.insert(0, f"kit:{kit_ref}")
    return {
        "schema_version": 1,
        "asset_id": asset_id,
        "archetype": archetype,
        "style_pack": style,
        "module": {"grid_units": grid, "snap": snap, "pivot": "bottom_center"},
        "dimensions_m": {"w": w, "h": h, "d": d},
        "material_profile": mat,
        "references": refs_out,
    }


def job_row(asset_id, archetype, dims, mat):
    w, h, d = dims
    jid = f"{asset_id}_run001"
    params = {
        "width_m": w,
        "height_m": h,
        "depth_m": d,
        "material_profile": mat,
        "seed": SEED,
    }
    if archetype == "module_roof":
        params = {
            "width_m": w,
            "thickness_m": h,
            "depth_m": d,
            "material_profile": mat,
            "seed": SEED,
        }
    return {
        "schema_version": 1,
        "job_id": jid,
        "spec_ref": f"assets/staging/specs/{asset_id}.json",
        "operation": archetype,
        "params": params,
        "output": {
            "glb": f"assets/staging/{jid}/model.glb",
            "thumbnail": f"assets/staging/{jid}/preview.png",
        },
    }


def main() -> None:
    SPECS_DIR.mkdir(parents=True, exist_ok=True)
    manifest_modules = []
    for row in MODULES:
        aid, archetype, style, grid, dims, snap, mat, kit_ref, refs = row
        spec = spec_row(aid, archetype, style, grid, dims, snap, mat, kit_ref, refs)
        job = job_row(aid, archetype, dims, mat)
        (SPECS_DIR / f"{aid}.json").write_text(json.dumps(spec, indent=2) + "\n", encoding="utf-8")
        (JOBS_DIR / f"{aid}_run001.json").write_text(json.dumps(job, indent=2) + "\n", encoding="utf-8")
        manifest_modules.append(
            {
                "asset_id": aid,
                "job_id": f"{aid}_run001",
                "kit_ref": kit_ref,
                "archetype": archetype,
                "status": "spec_pending",
            }
        )

    manifest = {
        "schema_version": 1,
        "batch_id": "kit_greybox_003",
        "description": "Kit gap slice — modules from design_procedural_module_kit_v1 not in _module_index",
        "plan_ref": "src/dev/design_procedural_module_kit_v1.md",
        "rules_applied": [
            "no_ai_generated_images",
            "deterministic_output",
            "batch_processing",
            "grid_alignment",
        ],
        "geometry_operations": [
            "module_wall",
            "module_roof",
            "module_door",
            "module_window",
            "module_prop",
        ],
        "modules": manifest_modules,
        "next_agent": "designer-mcp",
        "gate": "G1",
    }
    manifest_path = JOBS_DIR / "batch_kit_greybox_003.manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"written": len(MODULES), "manifest": str(manifest_path)}))


if __name__ == "__main__":
    main()
