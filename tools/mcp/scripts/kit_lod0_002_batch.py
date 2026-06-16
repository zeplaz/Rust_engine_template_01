"""kit_lod0_002 — 5 canonical modules, development_tier lod0, real profiles, no greybox cheats."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SPECS_DIR = ROOT / "assets" / "staging" / "specs"
JOBS_DIR = ROOT / "tools" / "mcp" / "schemas" / "examples"
RULES_DIR = ROOT / "debug_runs" / "art_pipeline"
BATCH_ID = "kit_lod0_002"
TIER = "lod0"
SEED = 42

# (module_id, archetype, style, grid, dims, snap, mat, pbr_status, profile)
MODULES = [
    (
        "wall_wood_1u",
        "module_wall",
        "style_rural",
        [1, 1],
        (4, 3, 0.25),
        "floor_edge",
        "wood_plank_01",
        "shipped",
        "flat",
    ),
    (
        "wall_steel_1u",
        "module_wall",
        "style_industrial_west",
        [1, 1],
        (4, 3, 0.2),
        "floor_edge",
        "steel_panel_01",
        "shipped",
        "flat",
    ),
    (
        "roof_flat",
        "module_roof",
        "style_industrial_west",
        [1, 1],
        (4, 0.3, 4),
        "roof_ridge",
        "roof_metal_01",
        "deferred",
        "flat",
    ),
    (
        "door_shop",
        "module_door",
        "style_rural",
        [1, 1],
        (2.0, 2.5, 0.15),
        "floor_edge",
        "wood_plank_01",
        "shipped",
        "shop",
    ),
    (
        "win_double_1u",
        "module_window",
        "style_rural",
        [1, 1],
        (4, 1.2, 0.12),
        "floor_edge",
        "glass_panel_01",
        "deferred",
        "frame_mullion",
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
    if archetype == "module_roof":
        params = {
            "width_m": w,
            "thickness_m": h,
            "depth_m": d,
            "material_profile": mat,
            "seed": SEED,
            "profile": profile,
        }
    else:
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


def g0_rules_yaml() -> str:
    return "\n".join(
        [
            f"batch_id: {BATCH_ID}",
            "gate: G0",
            "development_tier: lod0",
            "rules:",
            f"  batch_id: {BATCH_ID}",
            "  canonical_module_ids_only: true",
            "  no_greybox_refs: true",
            "  seed: 42",
            "notes:",
            f'  - "batch_id {BATCH_ID} — not kit_greybox_* (frozen smoke). development_tier lod0 on every spec/job."',
            '  - "Profiles: wall flat, roof flat, door shop, window frame_mullion double bay."',
        ]
    ) + "\n"


def main() -> None:
    SPECS_DIR.mkdir(parents=True, exist_ok=True)
    RULES_DIR.mkdir(parents=True, exist_ok=True)
    manifest_modules = []
    for row in MODULES:
        mid, archetype, style, grid, dims, snap, mat, pbr_status, profile = row
        spec = spec_row(mid, archetype, style, grid, dims, snap, mat, pbr_status, [])
        job = job_row(mid, archetype, dims, mat, profile, pbr_status)
        (SPECS_DIR / f"{mid}.json").write_text(json.dumps(spec, indent=2) + "\n", encoding="utf-8")
        (JOBS_DIR / f"{mid}_lod0_run001.json").write_text(json.dumps(job, indent=2) + "\n", encoding="utf-8")
        manifest_modules.append(
            {
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
        "description": "LOD0 wave 2 — 5 canonical kit modules (wood/steel walls, flat roof, shop door, double window)",
        "plan_ref": "docs/archive/2026-06-src-dev/plans/plan_kit_lod0_roadmap_v1.md",
        "kit_ref": "docs/archive/2026-06-src-dev/plans/design_procedural_module_kit_v1.md",
        "development_tier": TIER,
        "witness": f"debug_runs/art_pipeline/{BATCH_ID}_live.json",
        "signoff": f"debug_runs/art_pipeline/{BATCH_ID}_signoff.yaml",
        "rules_applied": [
            "no_ai_generated_images",
            "deterministic_output",
            "batch_processing",
            "grid_alignment",
            "canonical_module_ids_only",
        ],
        "modules": manifest_modules,
        "next_agent": "@coder-mcp",
        "gate": "G1",
    }
    manifest_path = JOBS_DIR / f"batch_{BATCH_ID}.manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    rules_path = RULES_DIR / f"{BATCH_ID}_g0_rules.yaml"
    rules_path.write_text(g0_rules_yaml(), encoding="utf-8")
    print(json.dumps({"batch_id": BATCH_ID, "modules": len(MODULES), "manifest": str(manifest_path)}))


if __name__ == "__main__":
    main()
