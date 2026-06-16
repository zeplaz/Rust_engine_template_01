"""Parametric kit_lod0 batch runner — G0–G5 from plan_kit_lod0_roadmap_v1.md (MCP-C0-012)."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[3]
SPECS_DIR = ROOT / "assets" / "staging" / "specs"
JOBS_DIR = ROOT / "tools" / "mcp" / "schemas" / "examples"
RULES_DIR = ROOT / "debug_runs" / "art_pipeline"
TIER = "lod0"
SEED = 42

# (module_id, archetype, style, grid, dims, snap, mat, pbr, profile, extra_job_params)
ModuleRow = tuple[str, str, str, list[int], tuple[float, float, float], str, str, str, str, dict[str, Any]]

ROADMAP: dict[str, list[ModuleRow]] = {
    "kit_lod0_003": [
        ("wall_concrete_2u", "module_wall", "style_industrial_west", [2, 1], (8, 3, 0.3), "floor_edge", "concrete_grey_01", "deferred", "flat", {}),
        ("roof_sawtooth", "module_roof", "style_industrial_west", [2, 2], (8, 0.3, 8), "roof_ridge", "roof_metal_01", "deferred", "sawtooth", {}),
        ("door_warehouse", "module_door", "style_industrial_west", [2, 1], (4, 3, 0.25), "floor_edge", "steel_door_warehouse_01", "deferred", "frame", {}),
        ("win_industrial_3u", "module_window", "style_industrial_west", [3, 1], (12, 1.2, 0.12), "floor_edge", "glass_panel_01", "deferred", "strip", {}),
        ("prop_vent", "module_prop", "style_industrial_west", [1, 1], (1, 0.4, 1), "roof_ridge", "vent_metal_01", "deferred", "box", {"prop_kind": "vent"}),
    ],
    "kit_lod0_004": [
        ("wall_brick_2u", "module_wall", "style_industrial_west", [2, 1], (8, 3, 0.3), "floor_edge", "brick_red_01", "deferred", "flat", {}),
        ("wall_wood_2u", "module_wall", "style_rural", [2, 1], (8, 3, 0.25), "floor_edge", "wood_plank_01", "deferred", "flat", {}),
        ("roof_pitched_hip", "module_roof", "style_rural", [2, 1], (8, 1.5, 8), "roof_ridge", "roof_tile_01", "deferred", "pitched_gable", {}),
        ("door_garage", "module_door", "style_rural", [2, 1], (4, 2.5, 0.2), "floor_edge", "steel_panel_01", "deferred", "frame", {}),
        ("win_arched_1u", "module_window", "style_rural", [1, 1], (4, 1.4, 0.12), "floor_edge", "glass_panel_01", "deferred", "arched", {}),
    ],
    "kit_lod0_005": [
        ("wall_glass_curtain_1u", "module_wall", "style_modern", [1, 1], (4, 3, 0.1), "floor_edge", "glass_panel_01", "deferred", "flat", {}),
        ("wall_industrial_panel_2u", "module_wall", "style_industrial_west", [2, 1], (8, 3, 0.25), "floor_edge", "steel_panel_01", "deferred", "flat", {}),
        ("roof_shed", "module_roof", "style_industrial_west", [2, 1], (8, 0.3, 4), "roof_ridge", "roof_metal_01", "deferred", "shed", {"pitch_height_m": 1.2}),
        ("door_office", "module_door", "style_modern", [1, 1], (1.2, 2.2, 0.12), "floor_edge", "glass_panel_01", "deferred", "frame", {}),
        ("win_strip_2u", "module_window", "style_modern", [2, 1], (8, 1.0, 0.1), "floor_edge", "glass_panel_01", "deferred", "strip", {}),
    ],
    "kit_lod0_006": [
        ("wall_military_bunker_1u", "module_wall", "style_military", [1, 1], (4, 2.5, 0.6), "floor_edge", "concrete_grey_01", "deferred", "flat", {}),
        ("roof_parapet", "module_roof", "style_military", [2, 1], (8, 0.5, 4), "roof_ridge", "concrete_grey_01", "deferred", "flat", {}),
        ("door_civic", "module_door", "style_colonial", [1, 1], (1.5, 2.4, 0.15), "floor_edge", "wood_plank_01", "deferred", "frame", {}),
        ("win_shop_2u", "module_window", "style_rural", [2, 1], (8, 1.5, 0.12), "floor_edge", "glass_shop_01", "deferred", "frame_mullion", {}),
        ("prop_light", "module_prop", "style_industrial_west", [1, 1], (0.4, 2.5, 0.4), "roof_ridge", "steel_panel_01", "deferred", "box", {}),
    ],
    "kit_lod0_007": [
        ("roof_metal_low", "module_roof", "style_industrial_west", [2, 1], (8, 0.25, 4), "roof_ridge", "roof_metal_01", "deferred", "flat", {}),
        ("roof_tile", "module_roof", "style_rural", [2, 1], (8, 0.35, 4), "roof_ridge", "roof_tile_01", "deferred", "flat", {}),
        ("door_military", "module_door", "style_military", [1, 1], (1.2, 2.2, 0.25), "floor_edge", "steel_panel_01", "deferred", "frame", {}),
        ("win_house_1u", "module_window", "style_rural", [1, 1], (4, 1.3, 0.12), "floor_edge", "glass_panel_01", "deferred", "frame_mullion", {}),
        ("corner_L", "module_prop", "style_industrial_west", [1, 1], (2, 3, 2), "floor_edge", "concrete_grey_01", "deferred", "box", {"prop_kind": "l_corner"}),
    ],
    "kit_lod0_008": [
        ("roof_bunker", "module_roof", "style_military", [2, 1], (8, 0.6, 4), "roof_ridge", "concrete_grey_01", "deferred", "flat", {}),
        ("roof_canopy", "module_roof", "style_industrial_west", [2, 1], (8, 0.2, 4), "roof_ridge", "roof_metal_01", "deferred", "shed", {}),
        ("door_factory", "module_door", "style_industrial_west", [2, 1], (4, 3, 0.2), "floor_edge", "steel_panel_01", "deferred", "frame", {}),
        ("win_office_1u", "module_window", "style_modern", [1, 1], (4, 1.2, 0.1), "floor_edge", "glass_panel_01", "deferred", "frame_mullion", {}),
        ("corner_T", "module_prop", "style_industrial_west", [1, 1], (2, 3, 2), "floor_edge", "concrete_grey_01", "deferred", "box", {"prop_kind": "l_corner"}),
    ],
    "kit_lod0_009": [
        ("door_double_shop", "module_door", "style_rural", [2, 1], (4, 2.5, 0.15), "floor_edge", "wood_plank_01", "deferred", "frame", {}),
        ("door_gate_industrial", "module_door", "style_industrial_west", [4, 1], (16, 4, 0.3), "floor_edge", "steel_panel_01", "deferred", "frame", {}),
        ("win_bunker_slit", "module_window", "style_military", [1, 1], (4, 0.3, 0.6), "floor_edge", "concrete_grey_01", "deferred", "strip", {}),
        ("win_skylight_1u", "module_window", "style_modern", [1, 1], (4, 0.15, 4), "roof_ridge", "glass_panel_01", "deferred", "flat", {}),
        ("corner_parapet", "module_prop", "style_military", [1, 1], (2, 1.2, 2), "floor_edge", "concrete_grey_01", "deferred", "box", {"prop_kind": "l_corner"}),
    ],
    "kit_lod0_010": [
        ("prop_fence", "module_prop", "style_rural", [2, 1], (8, 1.2, 0.1), "floor_edge", "wood_plank_01", "deferred", "box", {}),
        ("prop_tank", "module_prop", "style_industrial_west", [1, 1], (2, 2, 2), "floor_edge", "steel_panel_01", "deferred", "box", {}),
        ("prop_transformer", "module_prop", "style_industrial_west", [1, 1], (2, 1.5, 1.5), "floor_edge", "steel_panel_01", "deferred", "box", {}),
        ("prop_ac", "module_prop", "style_industrial_west", [1, 1], (1.2, 0.8, 1.2), "roof_ridge", "vent_metal_01", "deferred", "box", {"prop_kind": "ac"}),
        ("prop_chimney", "module_prop", "style_rural", [1, 1], (1, 2.5, 1), "roof_ridge", "brick_red_01", "deferred", "box", {}),
    ],
}

LOD0_BATCHES = tuple(f"kit_lod0_{n:03d}" for n in range(3, 11))


def job_id_for(module_id: str) -> str:
    return f"{module_id}_lod0_run001"


def spec_row(row: ModuleRow, batch_id: str) -> dict[str, Any]:
    mid, archetype, style, grid, dims, snap, mat, pbr, _profile, _extra = row
    w, h, d = dims
    return {
        "schema_version": 1,
        "asset_id": mid,
        "archetype": archetype,
        "style_pack": style,
        "development_tier": TIER,
        "pbr_status": pbr,
        "batch_id": batch_id,
        "module": {"grid_units": grid, "snap": snap, "pivot": "bottom_center"},
        "dimensions_m": {"w": w, "h": h, "d": d},
        "material_profile": mat,
        "references": [],
    }


def job_row(row: ModuleRow, batch_id: str) -> dict[str, Any]:
    mid, archetype, _style, _grid, dims, _snap, mat, _pbr, profile, extra = row
    w, h, d = dims
    jid = job_id_for(mid)
    if archetype == "module_roof":
        params: dict[str, Any] = {
            "width_m": w,
            "thickness_m": h,
            "depth_m": d,
            "material_profile": mat,
            "seed": SEED,
            "profile": profile,
        }
        if profile in ("pitched", "pitched_gable", "gable", "shed", "sawtooth"):
            params["pitch_height_m"] = float(extra.get("pitch_height_m", h if profile != "sawtooth" else max(h, 0.4)))
        params.update(extra)
    elif archetype == "module_prop":
        params = {
            "width_m": w,
            "height_m": h,
            "depth_m": d,
            "material_profile": mat,
            "seed": SEED,
            "profile": profile,
            **extra,
        }
    else:
        params = {
            "width_m": w,
            "height_m": h,
            "depth_m": d,
            "material_profile": mat,
            "seed": SEED,
            "profile": profile,
            **{k: v for k, v in extra.items() if k != "prop_kind" or archetype != "module_wall"},
        }
    return {
        "schema_version": 1,
        "job_id": jid,
        "batch_id": batch_id,
        "development_tier": TIER,
        "spec_ref": f"assets/staging/specs/{mid}.json",
        "operation": archetype,
        "params": params,
        "output": {
            "glb": f"assets/staging/{jid}/model.glb",
            "thumbnail": f"assets/staging/{jid}/preview.png",
        },
    }


def author_g0_g1(batch_id: str) -> dict[str, Any]:
    if batch_id not in ROADMAP:
        raise KeyError(f"Unknown batch_id {batch_id!r}")
    SPECS_DIR.mkdir(parents=True, exist_ok=True)
    RULES_DIR.mkdir(parents=True, exist_ok=True)
    modules = []
    for row in ROADMAP[batch_id]:
        mid = row[0]
        spec = spec_row(row, batch_id)
        job = job_row(row, batch_id)
        (SPECS_DIR / f"{mid}.json").write_text(json.dumps(spec, indent=2) + "\n", encoding="utf-8")
        (JOBS_DIR / f"{mid}_lod0_run001.json").write_text(json.dumps(job, indent=2) + "\n", encoding="utf-8")
        modules.append(
            {
                "asset_id": mid,
                "job_id": job_id_for(mid),
                "archetype": row[1],
                "development_tier": TIER,
                "pbr_status": row[7],
                "profile": row[8],
                "status": "spec_pending",
            }
        )
    rules = "\n".join(
        [
            f"batch_id: {batch_id}",
            "gate: G0",
            "development_tier: lod0",
            "rules:",
            f"  batch_id: {batch_id}",
            "  canonical_module_ids_only: true",
            "  no_greybox_refs: true",
            "  seed: 42",
        ]
    ) + "\n"
    (RULES_DIR / f"{batch_id}_g0_rules.yaml").write_text(rules, encoding="utf-8")
    manifest = {
        "schema_version": 1,
        "batch_id": batch_id,
        "description": f"LOD0 wave — {batch_id} from plan_kit_lod0_roadmap_v1.md",
        "plan_ref": "docs/archive/2026-06-src-dev/plans/plan_kit_lod0_roadmap_v1.md",
        "development_tier": TIER,
        "witness": f"debug_runs/art_pipeline/{batch_id}_live.json",
        "rules_applied": [
            "no_ai_generated_images",
            "deterministic_output",
            "batch_processing",
            "grid_alignment",
            "canonical_module_ids_only",
        ],
        "modules": modules,
        "next_agent": "@coder-mcp",
        "gate": "G1",
    }
    mp = JOBS_DIR / f"batch_{batch_id}.manifest.json"
    mp.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return {"batch_id": batch_id, "manifest": str(mp), "modules": len(modules)}


def validate_g0_g1(batch_id: str) -> None:
    sys.path.insert(0, str(ROOT / "tools" / "mcp" / "python"))
    from rust_engine_mcp.validators.mcp_schema import validate_mcp_job, validate_mcp_spec

    for row in ROADMAP[batch_id]:
        mid = row[0]
        sr = validate_mcp_spec(SPECS_DIR / f"{mid}.json", compression_level=1)
        jr = validate_mcp_job(JOBS_DIR / f"{mid}_lod0_run001.json", compression_level=1)
        if sr.status == "failed" or jr.status == "failed":
            raise RuntimeError(f"validation failed {mid}: spec={sr.status} job={jr.status}")


def run_geometry(batch_id: str) -> list[dict[str, Any]]:
    sys.path.insert(0, str(ROOT / "tools" / "mcp" / "python"))
    from rust_engine_mcp import blender_runner

    results = []
    for row in ROADMAP[batch_id]:
        jid = job_id_for(row[0])
        job_path = JOBS_DIR / f"{row[0]}_lod0_run001.json"
        r = blender_runner.run_geometry_job(job_path)
        glb = ROOT / "assets" / "staging" / jid / "model.glb"
        results.append({"job_id": jid, "status": r.status, "glb": glb.is_file()})
        if r.status != "done" or not glb.is_file():
            raise RuntimeError(f"geometry failed {jid}: {r.status} {r.error}")
    return results


def promote_batch(batch_id: str) -> dict[str, Any]:
    sys.path.insert(0, str(ROOT / "tools" / "mcp" / "python"))
    from rust_engine_mcp import promote
    from rust_engine_mcp.library import write_module_index
    from rust_engine_mcp.validators.asset import validate_asset_glb

    promoted = []
    for row in ROADMAP[batch_id]:
        jid = job_id_for(row[0])
        glb = ROOT / "assets" / "staging" / jid / "model.glb"
        report = validate_asset_glb(glb, compression_level=1)
        if report.status == "failed":
            raise RuntimeError(f"asset validation failed {jid}: {report.summary}")
        promoted.append(promote.promote_module(jid, register=True))
    idx = write_module_index()
    mp = JOBS_DIR / f"batch_{batch_id}.manifest.json"
    manifest = json.loads(mp.read_text(encoding="utf-8"))
    for m in manifest["modules"]:
        m["status"] = "promoted"
    manifest["gate"] = "G5"
    mp.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return {"batch_id": batch_id, "promoted": len(promoted), "index_entries": idx["entry_count"]}


def write_witness(batch_id: str) -> dict[str, Any]:
    sys.path.insert(0, str(ROOT / "tools" / "mcp" / "python"))
    from rust_engine_mcp import witness

    return witness.write_batch_witness(batch_id)


def run_batch(batch_id: str, phase: str) -> dict[str, Any]:
    out: dict[str, Any] = {"batch_id": batch_id, "phase": phase}
    if phase in ("g0g1", "all", "full"):
        out["author"] = author_g0_g1(batch_id)
        validate_g0_g1(batch_id)
        out["g0_g1"] = "pass"
    if phase in ("geometry", "g2g3", "all", "full"):
        out["geometry"] = run_geometry(batch_id)
    if phase in ("promote", "g4g5", "all", "full"):
        out["promote"] = promote_batch(batch_id)
        out["witness"] = write_witness(batch_id)
        out["G5"] = out["witness"]["gates"]["G5"]
    return out


def main() -> None:
    p = argparse.ArgumentParser(description="kit_lod0 batch runner (roadmap 003–010)")
    p.add_argument("--batch", default="kit_lod0_003", help="batch_id or 'all'")
    p.add_argument(
        "--phase",
        default="full",
        choices=["g0g1", "geometry", "promote", "full", "all"],
        help="pipeline phase",
    )
    args = p.parse_args()
    batches = LOD0_BATCHES if args.batch == "all" else [args.batch]
    results = []
    for bid in batches:
        print(f"=== {bid} ({args.phase}) ===", flush=True)
        results.append(run_batch(bid, args.phase))
        print(json.dumps(results[-1], indent=2), flush=True)
    print(json.dumps({"batches": len(results), "results": results}, indent=2))


if __name__ == "__main__":
    main()
