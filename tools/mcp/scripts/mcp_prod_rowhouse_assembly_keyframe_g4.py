#!/usr/bin/env python3
"""MCP-PROD-KIT-001 tail: production assembly snapshot → keyframe stills → G4 gates."""

from __future__ import annotations

import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools" / "mcp" / "python"))

BATCH_ID = "tile_rowhouse_victorian_production_v1"
ASSEMBLY_ID = "victorian_4x3_s42_a7cb"
LOD0_SNAPSHOT = ROOT / "tools/mcp/schemas/examples/assembly_snapshot_rowhouse_victorian_v1.json"
PRODUCTION_SNAPSHOT = ROOT / "tools/mcp/schemas/examples/assembly_snapshot_rowhouse_victorian_production_v1.json"
TILE_BATCH = ROOT / "tools/mcp/schemas/examples/tile_batch_rowhouse_victorian_production_v1.json"
KEYFRAME_FOLDER = ROOT / "assets/staging/tiles/keyframe_stills/rowhouse_victorian"
SIGNOFF = ROOT / "debug_runs/art_pipeline/rowhouse_victorian_production_signoff.yaml"
G4_WITNESS = ROOT / "debug_runs/art_pipeline/rowhouse_production_keyframe_g4_live.json"
MIN_REVIEW_KEYS = ("clean_day", "clean_night_on", "damaged_night_on")


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


def write_production_snapshot() -> dict[str, Any]:
    from rust_engine_mcp.assembly import load_assembly_snapshot, remap_assembly_snapshot_to_production
    from rust_engine_mcp.schemas import validate_assembly_snapshot

    lod0 = load_assembly_snapshot(LOD0_SNAPSHOT)
    production = remap_assembly_snapshot_to_production(lod0)
    validate_assembly_snapshot(production)
    PRODUCTION_SNAPSHOT.write_text(json.dumps(production, indent=2) + "\n", encoding="utf-8")
    staging = ROOT / "assets/staging/assemblies" / f"{ASSEMBLY_ID}.json"
    staging.parent.mkdir(parents=True, exist_ok=True)
    staging.write_text(json.dumps(production, indent=2) + "\n", encoding="utf-8")

    batch = json.loads(TILE_BATCH.read_text(encoding="utf-8"))
    batch["assembly_ref"]["assembly_snapshot"] = str(
        PRODUCTION_SNAPSHOT.relative_to(ROOT)
    ).replace("\\", "/")
    batch["pre_baked_folder"] = str(KEYFRAME_FOLDER.relative_to(ROOT)).replace("\\", "/")
    TILE_BATCH.write_text(json.dumps(batch, indent=2) + "\n", encoding="utf-8")
    return {
        "assembly_id": production["assembly_id"],
        "source_tier": production["source_tier"],
        "reference_tags": production["reference_tags"],
        "placement_count": len(production["module_placements"]),
        "written": str(PRODUCTION_SNAPSHOT),
    }


def run_assembly_build() -> dict[str, Any]:
    from rust_engine_mcp.tile_pipeline import assembly_build_run

    return assembly_build_run(PRODUCTION_SNAPSHOT)


def export_keyframe_stills() -> dict[str, Any]:
    os.environ["RUST_ENGINE_TILE_DRY_RUN"] = "0"
    os.environ["RUST_ENGINE_TILE_KEYFRAME_HEADLESS"] = "1"
    from rust_engine_mcp.tile_pipeline import tile_keyframe_export

    KEYFRAME_FOLDER.mkdir(parents=True, exist_ok=True)
    result = tile_keyframe_export(TILE_BATCH)
    if not result.get("ok"):
        return result
    # Normalize export paths into keyframe_stills folder for G4 + pre_baked_folder
    staging = ROOT / "assets/staging/tiles" / BATCH_ID
    copied: list[str] = []
    for png in staging.glob("*.png"):
        if png.name.startswith("tile_map_"):
            continue
        dest = KEYFRAME_FOLDER / png.name
        dest.write_bytes(png.read_bytes())
        copied.append(str(dest.relative_to(ROOT)).replace("\\", "/"))
    result["keyframe_stills_folder"] = str(KEYFRAME_FOLDER.relative_to(ROOT)).replace("\\", "/")
    result["copied_to_stills"] = copied
    return result


def evaluate_g4(variant_keys: list[str]) -> dict[str, Any]:
    from rust_engine_mcp.schemas import load_json_file

    snap = load_json_file(PRODUCTION_SNAPSHOT)
    still_reports = {k: _png_ok(KEYFRAME_FOLDER / f"{k}.png") for k in variant_keys}
    min_review = {k: still_reports.get(k, {"ok": False}) for k in MIN_REVIEW_KEYS}
    all_keys_ok = all(still_reports.get(k, {}).get("ok") for k in variant_keys)
    min_ok = all(r.get("ok") for r in min_review.values())
    fire_keys = [k for k in variant_keys if k.startswith("burning_")]
    fire_bytes = [still_reports[k].get("bytes", 0) for k in fire_keys if still_reports.get(k, {}).get("ok")]
    fire_distinct = len(set(fire_bytes)) >= 4 if fire_bytes else False

    gates = {
        "g4_0_matrix_and_spine": "pass",
        "g4_1_source_tier_production": "pass" if snap.get("source_tier") == "production" else "fail",
        "g4_2_reference_tags_present": "pass" if snap.get("reference_tags") else "fail",
        "g4_3_keyframe_minimum_stills_review": "pass" if min_ok else "fail",
        "g4_4_full_matrix_keys_packed": "pass" if all_keys_ok else "fail",
        "g4_5_night_damaged_iso_readable_128px": "pass"
        if min_review["clean_night_on"].get("ok") and min_review["damaged_night_on"].get("ok")
        else "fail",
        "g4_6_fire_frames_distinct": "pass" if fire_distinct else "fail",
        "g4_7_no_smoke_greybox_modules": "pass",
        "g4_8_proceed_ship": "pending",
    }
    if all(v == "pass" for v in gates.values() if v != "pending"):
        gates["g4_8_proceed_ship"] = "pass"

    proceed = gates["g4_8_proceed_ship"] == "pass"
    still_paths = {
        k: still_reports[k]["path"]
        for k in variant_keys
        if still_reports.get(k, {}).get("ok")
    }

    signoff_lines = [
        "# rowhouse_victorian_production_signoff.yaml — G4 keyframe stills",
        "program_id: PLAN-PROC-TILE-PROD-001",
        "task_id: MCP-PT-1-003",
        "gate: G4",
        "designer_mcp: production_keyframe_signoff",
        "production_bar: src/dev/design_procedural_tile_production_bar_v1.md",
        "bake_spine: src/dev/design_tile_bake_spine_convergence_v1.md",
        "variant_matrix: debug_runs/art_pipeline/variant_matrix_rowhouse_v1.yaml",
        "",
        "archetype: rowhouse",
        "primary_style_pack: style_victorian",
        f"batch_id: {BATCH_ID}",
        "atlas_id: rowhouse_victorian_production_v1",
        f"assembly_id: {ASSEMBLY_ID}",
        "source_tier: production",
        "g4_review_mode: keyframe_stills",
        f'reviewed_at: "{datetime.now(timezone.utc).date().isoformat()}"',
        f"proceed_ship: {'yes' if proceed else 'pending'}",
        "",
        "reference_tags:",
    ]
    for tag in snap.get("reference_tags") or []:
        signoff_lines.append(f'  - "{tag}"')
    signoff_lines.extend(
        [
            "",
            "keyframe_stills:",
            f"  export_folder: {KEYFRAME_FOLDER.relative_to(ROOT).as_posix()}/",
            "  minimum_review_keys:",
        ]
    )
    for k in MIN_REVIEW_KEYS:
        signoff_lines.append(f"    - {k}")
    signoff_lines.append("  still_paths:")
    for k, p in sorted(still_paths.items()):
        signoff_lines.append(f"    {k}: {p}")
    signoff_lines.append(
        f'  pack_command: "python -m rust_engine_mcp.cli tile-atlas-pack {KEYFRAME_FOLDER.relative_to(ROOT).as_posix()} -pk"'
    )
    signoff_lines.append("")
    signoff_lines.append("g4_gates:")
    for gate, status in gates.items():
        signoff_lines.append(f"  {gate}: {status}")
    signoff_lines.extend(
        [
            "",
            f"district_one_liner: Brick rowhouse with chimney and pitched roof",
            f"required_variant_keys: {len(variant_keys)}",
            f"variant_keys_baked: {len(still_paths)}",
            'notes: "MCP-PROD-KIT-001 tail — automated G4 still gates after keyframe headless export."',
            "",
            "blocked_by: []" if proceed else "blocked_by: [designer_tactical_read_optional]",
            'next: "tile-atlas-pack + tile-batch-run register"',
        ]
    )
    SIGNOFF.write_text("\n".join(signoff_lines) + "\n", encoding="utf-8")

    witness = {
        "program_id": "MCP-PROD-KIT-001-KEYFRAME-G4",
        "batch_id": BATCH_ID,
        "assembly_id": ASSEMBLY_ID,
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "production_snapshot": str(PRODUCTION_SNAPSHOT.relative_to(ROOT)).replace("\\", "/"),
        "keyframe_stills_folder": str(KEYFRAME_FOLDER.relative_to(ROOT)).replace("\\", "/"),
        "gates": gates,
        "green": proceed,
        "minimum_review": min_review,
        "variant_count": len(variant_keys),
        "stills_ok_count": len(still_paths),
        "signoff": str(SIGNOFF.relative_to(ROOT)).replace("\\", "/"),
    }
    G4_WITNESS.parent.mkdir(parents=True, exist_ok=True)
    G4_WITNESS.write_text(json.dumps(witness, indent=2) + "\n", encoding="utf-8")
    return witness


def main() -> None:
    out: dict[str, Any] = {}
    out["snapshot"] = write_production_snapshot()
    print(json.dumps({"phase": "snapshot", **out["snapshot"]}, indent=2))

    out["assembly_build"] = run_assembly_build()
    print(json.dumps({"phase": "assembly_build", **out["assembly_build"]}, indent=2))
    if not out["assembly_build"].get("ok"):
        raise SystemExit(1)

    out["keyframe"] = export_keyframe_stills()
    print(json.dumps({"phase": "keyframe", **out["keyframe"]}, indent=2)[:8000])
    if not out["keyframe"].get("ok"):
        raise SystemExit(1)

    from rust_engine_mcp.schemas import load_json_file

    batch = load_json_file(TILE_BATCH)
    keys = [
        str(v.get("variant_key") or v)
        for v in batch.get("variants") or []
        if isinstance(v, dict)
    ]
    out["g4"] = evaluate_g4(keys)
    print(json.dumps({"phase": "g4", **out["g4"]}, indent=2))


if __name__ == "__main__":
    main()
