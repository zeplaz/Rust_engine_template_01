"""kit_lod0_001 — 5 canonical modules, development_tier lod0, real profiles, no greybox cheats."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SPECS_DIR = ROOT / "assets" / "staging" / "specs"
JOBS_DIR = ROOT / "tools" / "mcp" / "schemas" / "examples"
BATCH_ID = "kit_lod0_001"
TIER = "lod0"
SEED = 42

# (module_id, archetype, style, grid, dims, snap, mat, pbr_status, profile, notes)
MODULES = [
    (
        "wall_brick_1u",
        "module_wall",
        "style_industrial_west",
        [1, 1],
        (4, 3, 0.3),
        "floor_edge",
        "brick_red_01",
        "shipped",
        "flat",
        [],
    ),
    (
        "wall_concrete_1u",
        "module_wall",
        "style_industrial_west",
        [1, 1],
        (4, 3, 0.3),
        "floor_edge",
        "concrete_grey_01",
        "shipped",
        "flat",
        [],
    ),
    (
        "roof_pitched_gable",
        "module_roof",
        "style_rural",
        [1, 1],
        (4, 1.5, 4),
        "roof_ridge",
        "roof_tile_01",
        "deferred",
        "pitched_gable",
        [],
    ),
    (
        "door_residential",
        "module_door",
        "style_rural",
        [1, 1],
        (1.0, 2.1, 0.15),
        "floor_edge",
        "wood_plank_01",
        "shipped",
        "residential",
        [],
    ),
    (
        "win_single_1u",
        "module_window",
        "style_rural",
        [1, 1],
        (4, 1.2, 0.12),
        "floor_edge",
        "glass_panel_01",
        "deferred",
        "frame_mullion",
        [],
    ),
]


def spec_row(module_id, archetype, style, grid, dims, snap, mat, pbr_status, refs):
    w, h, d = dims
    return {
        "schema_version": 1,
        "asset_id": module_id,
        "archetype": archetype,
        "style_pack": style,
        "development_tier": TIER,
        "pbr_status": pbr_status,
        "module": {"grid_units": grid, "snap": snap, "pivot": "bottom_center"},
        "dimensions_m": {"w": w, "h": h, "d": d},
        "material_profile": mat,
        "references": refs,
    }


def job_row(module_id, archetype, dims, mat, profile, pbr_status):
    w, h, d = dims
    jid = f"{module_id}_lod0_run001"
    params: dict = {
        "material_profile": mat,
        "seed": SEED,
        "profile": profile,
    }
    if archetype == "module_roof":
        params = {
            "width_m": w,
            "thickness_m": h,
            "depth_m": d,
            "pitch_height_m": h,
            "material_profile": mat,
            "seed": SEED,
            "profile": profile,
        }
    elif archetype in ("module_wall", "module_door", "module_window"):
        params = {
            "width_m": w,
            "height_m": h,
            "depth_m": d,
            "material_profile": mat,
            "seed": SEED,
            "profile": profile,
        }
    return {
        "schema_version": 1,
        "job_id": jid,
        "batch_id": BATCH_ID,
        "development_tier": TIER,
        "spec_ref": f"assets/staging/specs/{module_id}.json",
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
        mid, archetype, style, grid, dims, snap, mat, pbr_status, profile, refs = row
        spec = spec_row(mid, archetype, style, grid, dims, snap, mat, pbr_status, refs)
        job = job_row(mid, archetype, dims, mat, profile, pbr_status)
        (SPECS_DIR / f"{mid}.json").write_text(json.dumps(spec, indent=2) + "\n", encoding="utf-8")
        (JOBS_DIR / f"{mid}_lod0_run001.json").write_text(json.dumps(job, indent=2) + "\n", encoding="utf-8")
        manifest_modules.append(
            {
                "module_id": mid,
                "asset_id": mid,
                "job_id": f"{mid}_lod0_run001",
                "archetype": archetype,
                "development_tier": TIER,
                "pbr_status": pbr_status,
                "profile": profile,
                "status": "spec_pending",
            }
        )

    manifest = {
        "schema_version": 1,
        "batch_id": BATCH_ID,
        "description": "LOD0 canonical slice — 5 kit inventory modules, real profiles, no greybox cheats",
        "plan_ref": "docs/archive/2026-06-src-dev/plans/plan_module_kit_production_tier_v1.md",
        "kit_ref": "docs/archive/2026-06-src-dev/plans/design_procedural_module_kit_v1.md",
        "development_tier": TIER,
        "rules_applied": [
            "no_ai_generated_images",
            "deterministic_output",
            "batch_processing",
            "grid_alignment",
            "canonical_module_ids_only",
        ],
        "modules": manifest_modules,
        "next_agent": "designer-mcp",
        "gate": "G1",
    }
    out = JOBS_DIR / "batch_kit_lod0_001.manifest.json"
    out.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"batch_id": BATCH_ID, "modules": len(MODULES), "manifest": str(out)}))


if __name__ == "__main__":
    main()
