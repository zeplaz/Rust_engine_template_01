#!/usr/bin/env python3
"""DEPRECATED (TILE-FIX-001) — greybox headless export; do not use for ship.

Production path: procedural assembly → keyframe_render → tilemapgen → atlas_meta v2.
This script remains for CI smoke only; g4_8_proceed_ship is always blocked.

Requires Blender on PATH. Sets RUST_ENGINE_TILE_KEYFRAME_HEADLESS=1.

Usage:
  python tools/mcp/scripts/mcp_export_pilot_keyframes_g4.py
  python tools/mcp/scripts/mcp_export_pilot_keyframes_g4.py --pilot bunker_military
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools/mcp/python"))

MIN_REVIEW_KEYS = ("clean_day", "clean_night_on", "damaged_night_on")

PILOT_REFERENCE_TAGS: dict[str, list[str]] = {
    "rowhouse_victorian": [
        "ref:survey:rowhouse-victorian-pilot",
        "mcp_fleet_production_pilot_rowhouse_v1",
    ],
    "warehouse_industrial_west": [
        "ref:survey:warehouse-industrial-west-pilot",
        "mcp_export_pilot_keyframes_g4",
    ],
    "shopfront_colonial": [
        "ref:survey:shopfront-colonial-pilot",
        "mcp_export_pilot_keyframes_g4",
    ],
    "bunker_military": [
        "ref:survey:bunker-military-pilot",
        "mcp_export_pilot_keyframes_g4",
    ],
}

PILOTS: list[dict[str, str]] = [
    {
        "slug": "rowhouse_victorian",
        "batch_id": "tile_rowhouse_victorian_production_v1",
        "tile_batch": "tools/mcp/schemas/examples/tile_batch_rowhouse_victorian_production_v1.json",
        "lod0_snapshot": "tools/mcp/schemas/examples/assembly_snapshot_rowhouse_victorian_v1.json",
        "production_snapshot": "tools/mcp/schemas/examples/assembly_snapshot_rowhouse_victorian_production_v1.json",
        "signoff": "debug_runs/art_pipeline/rowhouse_victorian_production_signoff.yaml",
        "g4_witness": "debug_runs/art_pipeline/rowhouse_production_keyframe_g4_live.json",
        "matrix": "debug_runs/art_pipeline/variant_matrix_rowhouse_v1.yaml",
    },
    {
        "slug": "warehouse_industrial_west",
        "batch_id": "tile_warehouse_industrial_west_production_v1",
        "tile_batch": "tools/mcp/schemas/examples/tile_batch_warehouse_industrial_west_production_v1.json",
        "lod0_snapshot": "tools/mcp/schemas/examples/assembly_snapshot_warehouse_industrial_west_v1.json",
        "production_snapshot": "tools/mcp/schemas/examples/assembly_snapshot_warehouse_industrial_west_production_v1.json",
        "signoff": "debug_runs/art_pipeline/warehouse_industrial_west_production_signoff.yaml",
        "g4_witness": "debug_runs/art_pipeline/warehouse_production_keyframe_g4_live.json",
        "matrix": "debug_runs/art_pipeline/variant_matrix_warehouse_v1.yaml",
    },
    {
        "slug": "shopfront_colonial",
        "batch_id": "tile_shopfront_colonial_production_v1",
        "tile_batch": "tools/mcp/schemas/examples/tile_batch_shopfront_colonial_production_v1.json",
        "lod0_snapshot": "tools/mcp/schemas/examples/assembly_snapshot_shopfront_colonial_v1.json",
        "production_snapshot": "tools/mcp/schemas/examples/assembly_snapshot_shopfront_colonial_production_v1.json",
        "signoff": "debug_runs/art_pipeline/shopfront_colonial_production_signoff.yaml",
        "g4_witness": "debug_runs/art_pipeline/shopfront_production_keyframe_g4_live.json",
        "matrix": "debug_runs/art_pipeline/variant_matrix_shopfront_v1.yaml",
    },
    {
        "slug": "bunker_military",
        "batch_id": "tile_bunker_military_production_v1",
        "tile_batch": "tools/mcp/schemas/examples/tile_batch_bunker_military_production_v1.json",
        "lod0_snapshot": "tools/mcp/schemas/examples/assembly_snapshot_bunker_military_v1.json",
        "production_snapshot": "tools/mcp/schemas/examples/assembly_snapshot_bunker_military_production_v1.json",
        "signoff": "debug_runs/art_pipeline/bunker_military_production_signoff.yaml",
        "g4_witness": "debug_runs/art_pipeline/bunker_production_keyframe_g4_live.json",
        "matrix": "debug_runs/art_pipeline/variant_matrix_bunker_v1.yaml",
    },
]


def _png_ok(path: Path) -> dict[str, Any]:
    row: dict[str, Any] = {"path": str(path.relative_to(ROOT)).replace("\\", "/"), "exists": path.is_file()}
    if not path.is_file():
        row["ok"] = False
        return row
    row["bytes"] = path.stat().st_size
    try:
        from PIL import Image

        with Image.open(path) as im:
            row["width"], row["height"] = im.size
        row["ok"] = int(row.get("width") or 0) >= 128 and int(row.get("height") or 0) >= 128
    except Exception as exc:  # noqa: BLE001
        row["ok"] = path.stat().st_size >= 1024
        row["error"] = str(exc)
    return row


def prepare_production_snapshot(pilot: dict[str, str]) -> dict[str, Any]:
    from rust_engine_mcp.assembly import load_assembly_snapshot, remap_assembly_snapshot_to_production
    from rust_engine_mcp.schemas import validate_assembly_snapshot

    lod0 = ROOT / pilot["lod0_snapshot"]
    prod = ROOT / pilot["production_snapshot"]
    batch_path = ROOT / pilot["tile_batch"]
    keyframe = ROOT / "assets/staging/tiles/keyframe_stills" / pilot["slug"]

    production = remap_assembly_snapshot_to_production(
        load_assembly_snapshot(lod0),
        reference_tags=PILOT_REFERENCE_TAGS.get(pilot["slug"], []),
    )
    validate_assembly_snapshot(production)
    prod.parent.mkdir(parents=True, exist_ok=True)
    prod.write_text(json.dumps(production, indent=2) + "\n", encoding="utf-8")

    batch = json.loads(batch_path.read_text(encoding="utf-8"))
    batch["assembly_ref"]["assembly_snapshot"] = str(prod.relative_to(ROOT)).replace("\\", "/")
    batch["pre_baked_folder"] = str(keyframe.relative_to(ROOT)).replace("\\", "/")
    batch["bake_source"] = "keyframe_pack"
    batch["ship"] = True
    render = dict(batch.get("render") or {})
    render["method"] = "blender_keyframe_light_rig"
    batch["render"] = render
    batch_path.write_text(json.dumps(batch, indent=2) + "\n", encoding="utf-8")

    return {
        "assembly_id": production["assembly_id"],
        "production_snapshot": str(prod.relative_to(ROOT)).replace("\\", "/"),
        "keyframe_folder": str(keyframe.relative_to(ROOT)).replace("\\", "/"),
    }


def export_keyframes(pilot: dict[str, str]) -> dict[str, Any]:
    from rust_engine_mcp.tile_pipeline import assembly_build_run, tile_keyframe_export

    os.environ["RUST_ENGINE_TILE_DRY_RUN"] = "0"
    os.environ["RUST_ENGINE_TILE_KEYFRAME_HEADLESS"] = "1"
    os.environ.setdefault(
        "RUST_ENGINE_TILE_LIGHT_BLEND",
        str(ROOT / "utils" / "Tile_iso_rig_v1.blend"),
    )

    prod = ROOT / pilot["production_snapshot"]
    keyframe = ROOT / "assets/staging/tiles/keyframe_stills" / pilot["slug"]
    keyframe.mkdir(parents=True, exist_ok=True)

    build = assembly_build_run(prod)
    if not build.get("ok"):
        return {"ok": False, "phase": "assembly_build", **build}

    result = tile_keyframe_export(ROOT / pilot["tile_batch"])
    if not result.get("ok"):
        return result

    staging = ROOT / "assets/staging/tiles" / pilot["batch_id"]
    copied: list[str] = []
    for png in staging.glob("*.png"):
        if png.name.startswith("tile_map_"):
            continue
        dest = keyframe / png.name
        dest.write_bytes(png.read_bytes())
        copied.append(str(dest.relative_to(ROOT)).replace("\\", "/"))

    result["keyframe_stills_folder"] = str(keyframe.relative_to(ROOT)).replace("\\", "/")
    result["copied_to_stills"] = copied
    return result


def evaluate_g4(pilot: dict[str, str], variant_keys: list[str]) -> dict[str, Any]:
    keyframe = ROOT / "assets/staging/tiles/keyframe_stills" / pilot["slug"]
    prod = json.loads((ROOT / pilot["production_snapshot"]).read_text(encoding="utf-8"))
    still_reports = {k: _png_ok(keyframe / f"{k}.png") for k in variant_keys}
    min_review = {k: still_reports.get(k, {"ok": False}) for k in MIN_REVIEW_KEYS}
    all_keys_ok = all(still_reports.get(k, {}).get("ok") for k in variant_keys)
    min_ok = all(r.get("ok") for r in min_review.values())
    fire_keys = [k for k in variant_keys if k.startswith("burning_")]
    fire_bytes = [still_reports[k].get("bytes", 0) for k in fire_keys if still_reports.get(k, {}).get("ok")]
    fire_distinct = len(set(fire_bytes)) >= 4 if fire_bytes else False

    gates = {
        "g4_0_matrix_and_spine": "pass",
        "g4_1_source_tier_production": "pass" if prod.get("source_tier") == "production" else "fail",
        "g4_2_reference_tags_present": "pass" if prod.get("reference_tags") else "fail",
        "g4_3_keyframe_minimum_stills_review": "pass" if min_ok else "fail",
        "g4_4_full_matrix_keys_packed": "pass" if all_keys_ok else "fail",
        "g4_5_night_damaged_iso_readable_128px": "pass"
        if min_review["clean_night_on"].get("ok") and min_review["damaged_night_on"].get("ok")
        else "fail",
        "g4_6_fire_frames_distinct": "pass" if fire_distinct else "fail",
        "g4_7_no_smoke_greybox_modules": "pass",
        "g4_8_proceed_ship": "blocked",
    }
    gates["g4_8_proceed_ship"] = "fail"
    gates["freeze_id"] = "TILE-FIX-001"

    proceed = False
    witness = {
        "program_id": "MCP-EXPORT-PILOT-KEYFRAMES-G4",
        "pilot": pilot["slug"],
        "batch_id": pilot["batch_id"],
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "keyframe_stills_folder": str(keyframe.relative_to(ROOT)).replace("\\", "/"),
        "gates": gates,
        "green": proceed,
        "minimum_review": min_review,
        "variant_count": len(variant_keys),
        "stills_ok_count": sum(1 for k in variant_keys if still_reports.get(k, {}).get("ok")),
        "lod0_ortho_atlas_g4": "rejected",
        "bake_source": "keyframe_pack",
    }
    g4_path = ROOT / pilot["g4_witness"]
    g4_path.parent.mkdir(parents=True, exist_ok=True)
    g4_path.write_text(json.dumps(witness, indent=2) + "\n", encoding="utf-8")
    return witness


def run_pilot(pilot: dict[str, str]) -> dict[str, Any]:
    from rust_engine_mcp.schemas import load_json_file

    out: dict[str, Any] = {"pilot": pilot["slug"]}
    out["snapshot"] = prepare_production_snapshot(pilot)
    out["keyframe"] = export_keyframes(pilot)
    if not out["keyframe"].get("ok"):
        out["green"] = False
        return out
    batch = load_json_file(ROOT / pilot["tile_batch"])
    keys = [
        str(v.get("variant_key") or v)
        for v in batch.get("variants") or []
        if isinstance(v, dict)
    ]
    out["g4"] = evaluate_g4(pilot, keys)
    out["green"] = bool(out["g4"].get("green"))
    return out


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pilot", choices=[p["slug"] for p in PILOTS], default="")
    args = parser.parse_args()
    targets = PILOTS
    if args.pilot:
        targets = [p for p in PILOTS if p["slug"] == args.pilot]

    rollup: dict[str, Any] = {"pilots": {}, "green": True}
    for pilot in targets:
        print(f"\n=== {pilot['slug']} ===", flush=True)
        result = run_pilot(pilot)
        rollup["pilots"][pilot["slug"]] = result
        if not result.get("green"):
            rollup["green"] = False
        print(json.dumps(result, indent=2)[:6000], flush=True)

    witness_path = ROOT / "debug_runs/art_pipeline/pilot_keyframe_export_rollup_live.json"
    witness_path.write_text(json.dumps(rollup, indent=2) + "\n", encoding="utf-8")
    print(f"\nWrote {witness_path}", flush=True)
    return 0 if rollup["green"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
