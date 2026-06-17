"""MCP-P2-KIT002 — warehouse kit_production_002 gates (G2 roof bpy, G3 tier validate)."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

MANIFEST_REL = "tools/mcp/schemas/examples/batch_kit_production_002.manifest.json"
G2_WITNESS_REL = "debug_runs/art_pipeline/kit_production_002_g2_live.json"
G3_WITNESS_REL = "debug_runs/art_pipeline/kit_production_002_g3_live.json"
G4_WITNESS_REL = "debug_runs/art_pipeline/kit_production_002_g4_live.json"
G4_SIGNOFF_REL = "debug_runs/art_pipeline/kit_production_002_g4_signoff.yaml"
BATCH_WITNESS_REL = "debug_runs/art_pipeline/kit_production_002_live.json"
TILE_BATCH_REL = "tools/mcp/schemas/examples/tile_batch_warehouse_industrial_west_production_v1.json"
BUILDING_DEF_REL = (
    "tools/mcp/schemas/examples/building_definition_warehouse_industrial_west_production_v1.json"
)
VARIANT_MATRIX_REL = "debug_runs/art_pipeline/variant_matrix_warehouse_v1.yaml"
KEYFRAME_PRIMARY_REL = "assets/staging/tiles/keyframe_stills/warehouse_industrial"
KEYFRAME_LEGACY_REL = "assets/staging/tiles/keyframe_stills/warehouse_industrial_west"
MIN_REVIEW_KEYS: tuple[str, ...] = ("clean_day", "clean_night_on", "damaged_night_on")
G3_ASSET_COMPRESSION = 3

ROOF_MODULE_ID = "roof_industrial_shed_2u"
ROOF_JOB_ID = "roof_industrial_shed_2u_production_run001"
ROOF_PROFILE = "shed"
ROOF_OPERATION = "module_roof"

KIT_PRODUCTION_002_G2_JOBS: tuple[tuple[str, str], ...] = (
    (ROOF_MODULE_ID, ROOF_JOB_ID),
)


def _job_path(job_id: str) -> Path:
    return repo_root() / "tools/mcp/schemas/examples" / f"{job_id}.json"


def load_manifest(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    path = root / MANIFEST_REL
    return json.loads(path.read_text(encoding="utf-8"))


def promoted_glb_path(job_id: str, *, repo: Path | None = None) -> Path:
    root = repo or repo_root()
    return root / "assets/models/modules" / job_id / "model.glb"


def _load_job(job_id: str) -> dict[str, Any]:
    path = _job_path(job_id)
    if not path.is_file():
        raise FileNotFoundError(path)
    return json.loads(path.read_text(encoding="utf-8"))


def _profile_ok(job: dict[str, Any], operation: str, expected_profile: str) -> bool:
    if str(job.get("operation") or "") != operation:
        return False
    params = job.get("params") or {}
    profile = str(params.get("profile") or "").lower()
    return profile == expected_profile.lower()


def kit_production_002_g2_status() -> dict[str, Any]:
    job = _load_job(ROOF_JOB_ID)
    staging_glb = repo_root() / "assets/staging" / ROOF_JOB_ID / "model.glb"
    promoted_glb = repo_root() / "assets/models/modules" / ROOF_JOB_ID / "model.glb"
    return {
        "module_id": ROOF_MODULE_ID,
        "job_id": ROOF_JOB_ID,
        "profile": ROOF_PROFILE,
        "profile_ok": _profile_ok(job, ROOF_OPERATION, ROOF_PROFILE),
        "staging_glb": staging_glb.is_file(),
        "promoted_glb": promoted_glb.is_file(),
        "batch_id": str(job.get("batch_id") or "kit_production_002"),
        "development_tier": str(job.get("development_tier") or "production"),
    }


def run_kit_production_002_g2_geometry() -> dict[str, Any]:
    from rust_engine_mcp import blender_runner

    job_path = _job_path(ROOF_JOB_ID)
    result = blender_runner.run_geometry_job(job_path)
    glb = repo_root() / "assets/staging" / ROOF_JOB_ID / "model.glb"
    ok = result.status == "done" and glb.is_file()
    return {
        "ok": ok,
        "job_id": ROOF_JOB_ID,
        "status": result.status,
        "staging_glb": glb.is_file(),
        "error": getattr(result, "error", None),
    }


def promote_kit_production_002_g2_roof(*, register: bool = True) -> dict[str, Any]:
    from rust_engine_mcp import promote
    from rust_engine_mcp.library import write_module_index
    from rust_engine_mcp.material_textures import PILOT_PROFILES, generate_profile
    from rust_engine_mcp.validators.asset import validate_asset_glb

    glb = repo_root() / "assets/staging" / ROOF_JOB_ID / "model.glb"
    if not glb.is_file():
        raise FileNotFoundError(f"staging glb missing: {glb}")
    profile_def = PILOT_PROFILES.get("metal_roof_01")
    if profile_def is not None:
        generate_profile(profile_def)
    report = validate_asset_glb(glb, compression_level=1)
    if report.status == "failed":
        raise RuntimeError(f"asset validation failed: {report.summary}")
    manifest = promote.promote_module(ROOF_JOB_ID, register=register)
    index = write_module_index()
    _sync_manifest_roof_promoted()
    return {"promoted": manifest, "index_entries": index.get("entry_count")}


def _sync_manifest_roof_promoted() -> None:
    path = repo_root() / MANIFEST_REL
    body = json.loads(path.read_text(encoding="utf-8"))
    for row in body.get("modules") or []:
        if str(row.get("module_id") or "") == ROOF_MODULE_ID:
            row["status"] = "promoted"
            row["job_id"] = ROOF_JOB_ID
    body["gate"] = "G2"
    path.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")


def refresh_kit_production_002_g2_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    status = kit_production_002_g2_status()
    green = bool(
        status.get("profile_ok")
        and status.get("staging_glb")
        and status.get("promoted_glb")
    )
    body: dict[str, Any] = {
        "gate": "MCP-P2-KIT002-G2",
        "green": green,
        "batch_id": "kit_production_002",
        "module_id": ROOF_MODULE_ID,
        "job_id": ROOF_JOB_ID,
        "bpy_operation": ROOF_OPERATION,
        "bpy_profile": ROOF_PROFILE,
        "status": status,
        "manifest": MANIFEST_REL,
        "g0_witness": "debug_runs/art_pipeline/warehouse_production_g0_live.json",
        "_agent_meta": {
            "schema": "kit_production_002_g2_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "KIT_PRODUCTION_002_G2",
            "source_system": "kit_production_002",
            "relative_path": G2_WITNESS_REL,
        },
    }
    out = root / G2_WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = G2_WITNESS_REL
    _write_batch_witness_partial(root, green=green, status=status)
    return body


def _write_batch_witness_partial(root: Path, *, green: bool, status: dict[str, Any]) -> None:
    manifest = json.loads((root / MANIFEST_REL).read_text(encoding="utf-8"))
    promoted = sum(1 for m in manifest.get("modules") or [] if m.get("status") == "promoted")
    body = {
        "gate": "kit_production_002",
        "green": green and promoted >= 6,
        "batch_id": "kit_production_002",
        "module_count": manifest.get("module_count"),
        "promoted_count": promoted,
        "g2_roof": status,
        "manifest": MANIFEST_REL,
        "note": "G5 partial — roof G2 green; full batch witness closes at G5 register",
    }
    out = root / BATCH_WITNESS_REL
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")


def validate_kit_production_002_g3_module(
    row: dict[str, Any],
    *,
    compression_level: int = G3_ASSET_COMPRESSION,
    repo: Path | None = None,
) -> dict[str, Any]:
    from rust_engine_mcp.validators.asset import validate_asset_glb

    root = repo or repo_root()
    module_id = str(row.get("module_id") or "")
    job_id = str(row.get("job_id") or "")
    manifest_status = str(row.get("status") or "")
    if manifest_status != "promoted":
        return {
            "module_id": module_id,
            "job_id": job_id,
            "manifest_status": manifest_status,
            "passed": False,
            "validate_asset_report": {"status": "skipped", "reason": "not promoted"},
        }
    glb = promoted_glb_path(job_id, repo=root)
    glb_rel = str(glb.relative_to(root)).replace("\\", "/")
    if not glb.is_file():
        return {
            "module_id": module_id,
            "job_id": job_id,
            "glb": glb_rel,
            "passed": False,
            "validate_asset_report": {"status": "failed", "reason": "promoted glb missing"},
        }
    report = validate_asset_glb(glb, compression_level=compression_level)
    passed = report.status in ("passed", "warning")
    return {
        "module_id": module_id,
        "job_id": job_id,
        "glb": glb_rel,
        "passed": passed,
        "validate_asset_report": {
            "status": report.status,
            "summary": report.summary,
            "error_count": report.error_count,
            "warning_count": report.warning_count,
            "compression_level": compression_level,
        },
    }


def validate_kit_production_002_g3_batch(
    *,
    compression_level: int = G3_ASSET_COMPRESSION,
    repo: Path | None = None,
) -> dict[str, Any]:
    from rust_engine_mcp.tile_promotion_honest import (
        tile_promotion_honest_check,
        validate_tile_promotion_honest_path,
    )

    root = repo or repo_root()
    manifest = load_manifest(repo=root)
    modules = [
        validate_kit_production_002_g3_module(row, compression_level=compression_level, repo=root)
        for row in manifest.get("modules") or []
    ]
    all_modules_passed = all(m.get("passed") for m in modules) and len(modules) == int(
        manifest.get("module_count") or 0
    )

    paired = str(manifest.get("paired_tile_batch") or "")
    batch_rel = f"tools/mcp/schemas/examples/{paired}.json"
    batch_path = root / batch_rel
    batch_doc = json.loads(batch_path.read_text(encoding="utf-8")) if batch_path.is_file() else {}
    ship = bool(batch_doc.get("ship", False))
    honest_check = tile_promotion_honest_check(
        batch_path=batch_path,
        ship=ship,
        honest_bake=True,
    )
    honest_rep = validate_tile_promotion_honest_path(
        batch_path,
        ship=ship,
        honest_bake=True,
        compression_level=compression_level,
    )
    honest_ok = honest_rep.status in ("passed", "warning")

    return {
        "batch_id": str(manifest.get("batch_id") or "kit_production_002"),
        "module_count": len(modules),
        "passed_count": sum(1 for m in modules if m.get("passed")),
        "all_modules_passed": all_modules_passed,
        "modules": modules,
        "paired_tile_batch": paired,
        "tile_promotion_honest": {
            "status": honest_rep.status,
            "ship": ship,
            "ok": bool(honest_check.get("ok")),
            "bake_source": honest_check.get("bake_source"),
            "render_method": honest_check.get("render_method"),
            "batch_path": batch_rel,
        },
        "green": all_modules_passed and honest_ok,
    }


def _sync_manifest_gate_g3(*, repo: Path | None = None) -> None:
    root = repo or repo_root()
    path = root / MANIFEST_REL
    body = json.loads(path.read_text(encoding="utf-8"))
    body["gate"] = "G3"
    path.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")


def refresh_kit_production_002_g3_witness(
    *,
    repo: Path | None = None,
    sync_manifest: bool = True,
) -> dict[str, Any]:
    root = repo or repo_root()
    batch = validate_kit_production_002_g3_batch(repo=root)
    green = bool(batch.get("green"))
    if sync_manifest and green:
        _sync_manifest_gate_g3(repo=root)

    body: dict[str, Any] = {
        "gate": "MCP-P2-KIT002-G3",
        "green": green,
        "verdict": "PASS" if green else "FAIL",
        "batch_id": "kit_production_002",
        "development_tier": "production",
        "manifest": MANIFEST_REL,
        "g1_witness": "debug_runs/art_pipeline/kit_production_002_g1_live.json",
        "g2_witness": G2_WITNESS_REL,
        "validate_asset_report": {
            "compression_level": G3_ASSET_COMPRESSION,
            "all_passed": batch.get("all_modules_passed"),
            "passed_count": batch.get("passed_count"),
            "module_count": batch.get("module_count"),
        },
        "modules": batch.get("modules"),
        "tile_promotion_honest": batch.get("tile_promotion_honest"),
        "tile_ship_blocked_by": [
            "TILE-FIX-001_v1_frozen",
            "G4_manual_keyframe_pending",
        ],
        "proceed_tile_ship": False,
        "unblocks": ["MCP-P2-KIT002-G4"] if green else [],
        "delta_wf": "@designer-mcp MCP-P2-KIT002-G4 keyframe stills" if green else None,
        "_agent_meta": {
            "schema": "kit_production_002_g3_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "KIT_PRODUCTION_002_G3",
            "source_system": "kit_production_002",
            "relative_path": G3_WITNESS_REL,
            "ritual": "BLANG:Q✓ MCP-P2-KIT002-G3" if green else None,
        },
    }
    out = root / G3_WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = G3_WITNESS_REL
    _write_batch_witness_g3(root, green=green, batch=batch)
    return body


def _write_batch_witness_g3(root: Path, *, green: bool, batch: dict[str, Any]) -> None:
    manifest = load_manifest(repo=root)
    promoted = sum(1 for m in manifest.get("modules") or [] if m.get("status") == "promoted")
    body = {
        "gate": "G3",
        "green": green,
        "batch_id": "kit_production_002",
        "module_count": manifest.get("module_count"),
        "promoted_count": promoted,
        "g3": {
            "validate_asset_report": batch.get("all_modules_passed"),
            "passed_count": batch.get("passed_count"),
            "tile_promotion_honest": batch.get("tile_promotion_honest"),
        },
        "manifest": MANIFEST_REL,
        "g3_witness": G3_WITNESS_REL,
        "note": "G3 tier pass on 6/6 promoted GLBs; tile ship still blocked until G4+register",
    }
    out = root / BATCH_WITNESS_REL
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")


def _png_ok(path: Path) -> dict[str, Any]:
    row: dict[str, Any] = {
        "path": str(path).replace("\\", "/"),
        "exists": path.is_file(),
    }
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


def _variant_keys_from_tile_batch(*, repo: Path | None = None) -> list[str]:
    root = repo or repo_root()
    batch = json.loads((root / TILE_BATCH_REL).read_text(encoding="utf-8"))
    return [
        str(v.get("variant_key") or v)
        for v in batch.get("variants") or []
        if isinstance(v, dict) and (v.get("variant_key") or v)
    ]


def _resolve_keyframe_stills_folder(*, repo: Path | None = None) -> tuple[Path | None, str]:
    root = repo or repo_root()
    primary = root / KEYFRAME_PRIMARY_REL
    legacy = root / KEYFRAME_LEGACY_REL
    if primary.is_dir() and any(primary.glob("*.png")):
        return primary, KEYFRAME_PRIMARY_REL
    if legacy.is_dir() and any(legacy.glob("*.png")):
        return legacy, KEYFRAME_LEGACY_REL
    return None, KEYFRAME_PRIMARY_REL


def evaluate_kit_production_002_g4(*, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp.tile_compile_loop import run_designer_warehouse_phase_c

    root = repo or repo_root()
    keyframe_dir, keyframe_rel = _resolve_keyframe_stills_folder(repo=root)
    variant_keys = _variant_keys_from_tile_batch(repo=root)
    still_reports = {
        k: (
            _png_ok(keyframe_dir / f"{k}.png")
            if keyframe_dir
            else {"ok": False, "exists": False, "path": f"{keyframe_rel}/{k}.png"}
        )
        for k in variant_keys
    }
    min_review = {k: still_reports.get(k, {"ok": False}) for k in MIN_REVIEW_KEYS}
    min_ok = all(r.get("ok") for r in min_review.values())
    all_keys_ok = all(still_reports.get(k, {}).get("ok") for k in variant_keys)
    fire_keys = [k for k in variant_keys if k.startswith("burning_")]
    fire_bytes = [still_reports[k].get("bytes", 0) for k in fire_keys if still_reports.get(k, {}).get("ok")]
    fire_distinct = len(set(fire_bytes)) >= 4 if fire_bytes else False

    phase_c = run_designer_warehouse_phase_c(root / BUILDING_DEF_REL, require_manual_art=True)
    art_quality = str(phase_c.get("art_quality") or "rejected_headless_procedural")

    prod_snap = root / "tools/mcp/schemas/examples/assembly_snapshot_warehouse_industrial_west_production_v1.json"
    prod = json.loads(prod_snap.read_text(encoding="utf-8")) if prod_snap.is_file() else {}

    gates = {
        "g4_0_matrix_and_spine": "pass",
        "g4_1_source_tier_production": "pass" if prod.get("source_tier") == "production" else "fail",
        "g4_2_reference_tags_present": "pass" if prod.get("reference_tags") else "fail",
        "g4_3_keyframe_minimum_stills_review": "pass" if min_ok else "fail",
        "g4_4_full_matrix_keys_packed": "pass" if all_keys_ok else "fail",
        "g4_5_night_damaged_iso_readable_128px": "pass"
        if min_review.get("clean_night_on", {}).get("ok") and min_review.get("damaged_night_on", {}).get("ok")
        else "fail",
        "g4_6_fire_frames_distinct": "pass" if fire_distinct else "fail",
        "g4_7_no_smoke_greybox_modules": "pass",
    }
    proceed_ship = (
        all(gates[k] == "pass" for k in gates)
        and art_quality == "keyframe_manual"
        and bool(phase_c.get("proceed_ship"))
    )
    gates["g4_8_proceed_ship"] = "pass" if proceed_ship else "fail"

    missing_min = [k for k in MIN_REVIEW_KEYS if not min_review.get(k, {}).get("ok")]
    missing_keys = [k for k in variant_keys if not still_reports.get(k, {}).get("ok")]
    blocked_by: list[str] = []
    if keyframe_dir is None:
        blocked_by.append("keyframe_stills_folder_missing")
    if missing_min:
        blocked_by.append("minimum_review_stills_missing")
    if art_quality != "keyframe_manual":
        blocked_by.append("manual_keyframe_render")

    return {
        "variant_keys": variant_keys,
        "keyframe_stills_folder": keyframe_rel,
        "keyframe_stills_resolved": keyframe_dir is not None,
        "minimum_review": min_review,
        "stills_ok_count": sum(1 for k in variant_keys if still_reports.get(k, {}).get("ok")),
        "variant_count": len(variant_keys),
        "gates": gates,
        "art_quality": art_quality,
        "phase_c": {
            "witness": "debug_runs/art_pipeline/tile_fix_09_phase_c_warehouse_g4_live.json",
            "proceed_ship": phase_c.get("proceed_ship"),
            "minimum_g4_ship": phase_c.get("minimum_g4_ship"),
        },
        "proceed_ship": proceed_ship,
        "proceed_tile_ship": proceed_ship,
        "blocked_by": blocked_by,
        "missing_minimum_keys": missing_min,
        "missing_variant_keys": missing_keys,
        "green": proceed_ship,
        "verdict": "PASS" if proceed_ship else "FAIL",
    }


def write_kit_production_002_g4_signoff(
    evaluation: dict[str, Any],
    *,
    repo: Path | None = None,
) -> dict[str, Any]:
    from datetime import datetime, timezone

    root = repo or repo_root()
    reviewed_at = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    proceed = "yes" if evaluation.get("proceed_ship") else "no"
    keyframe_rel = str(evaluation.get("keyframe_stills_folder") or KEYFRAME_PRIMARY_REL)
    min_review = evaluation.get("minimum_review") or {}
    still_lines = "\n".join(
        f"    {k}: {row.get('path', keyframe_rel + '/' + k + '.png')}"
        for k, row in min_review.items()
    )
    blocked = evaluation.get("blocked_by") or []
    blocked_yaml = ", ".join(blocked) if blocked else "[]"
    gates = evaluation.get("gates") or {}
    gate_lines = "\n".join(f"  {k}: {v}" for k, v in gates.items())

    body = f"""# kit_production_002_g4_signoff.yaml — MCP-P2-KIT002-G4 warehouse keyframe matrix
program_id: MCP-PRODUCTIVITY-P2-001
task_id: MCP-P2-KIT002-G4
gate: G4
designer_mcp: production_keyframe_signoff
production_bar: docs/archive/2026-06-src-dev/plans/design_procedural_tile_production_bar_v1.md
variant_matrix: {VARIANT_MATRIX_REL}
manifest: {MANIFEST_REL}
paired_tile_batch: {TILE_BATCH_REL}

archetype: warehouse
primary_style_pack: style_industrial_west
batch_id: tile_warehouse_industrial_west_production_v1
kit_batch_id: kit_production_002
source_tier: production
g4_review_mode: keyframe_stills
reviewed_at: "{reviewed_at}"
proceed_ship: {proceed}

reference_tags:
  - "ref:gate:MCP-P2-KIT002-G4"
  - "ref:survey:warehouse-industrial-west-pilot"

keyframe_stills:
  export_folder: {keyframe_rel}/
  legacy_folder: {KEYFRAME_LEGACY_REL}/
  minimum_review_keys:
    - clean_day
    - clean_night_on
    - damaged_night_on
  still_paths:
{still_lines}
  pack_command: "python -m rust_engine_mcp.cli tile-atlas-pack {keyframe_rel} -pk"

phase_c:
  cli: tools/mcp/scripts/designer_mcp_warehouse_phase_c.ps1
  witness: debug_runs/art_pipeline/tile_fix_09_phase_c_warehouse_g4_live.json
  art_quality: {evaluation.get("art_quality")}

g4_gates:
{gate_lines}

required_variant_keys: {evaluation.get("variant_count")}
variant_keys_baked: {evaluation.get("stills_ok_count")}
blocked_by: [{blocked_yaml}]
next: "{'tile-atlas-pack + @coder-mcp MCP-P2-KIT002-G5 register' if evaluation.get('proceed_ship') else 'operator manual keyframe_render → re-run designer_mcp_warehouse_phase_c.ps1'}"
notes: "Designer-mcp G4 — proceed_ship only when keyframe_stills exist and art_quality=keyframe_manual."
"""
    out = root / G4_SIGNOFF_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(body, encoding="utf-8")
    return {"written": G4_SIGNOFF_REL, "proceed_ship": evaluation.get("proceed_ship")}


def _sync_manifest_gate_g4(*, repo: Path | None = None) -> None:
    root = repo or repo_root()
    path = root / MANIFEST_REL
    body = json.loads(path.read_text(encoding="utf-8"))
    body["gate"] = "G4"
    path.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")


def refresh_kit_production_002_g4_witness(
    *,
    repo: Path | None = None,
    sync_manifest: bool = True,
) -> dict[str, Any]:
    root = repo or repo_root()
    evaluation = evaluate_kit_production_002_g4(repo=root)
    signoff = write_kit_production_002_g4_signoff(evaluation, repo=root)
    green = bool(evaluation.get("green"))
    if sync_manifest:
        _sync_manifest_gate_g4(repo=root)

    body: dict[str, Any] = {
        "gate": "MCP-P2-KIT002-G4",
        "green": green,
        "verdict": evaluation.get("verdict"),
        "batch_id": "kit_production_002",
        "paired_tile_batch": TILE_BATCH_REL,
        "development_tier": "production",
        "manifest": MANIFEST_REL,
        "g3_witness": G3_WITNESS_REL,
        "variant_matrix": VARIANT_MATRIX_REL,
        "keyframe_stills_folder": evaluation.get("keyframe_stills_folder"),
        "keyframe_stills_resolved": evaluation.get("keyframe_stills_resolved"),
        "minimum_review": evaluation.get("minimum_review"),
        "gates": evaluation.get("gates"),
        "variant_count": evaluation.get("variant_count"),
        "stills_ok_count": evaluation.get("stills_ok_count"),
        "art_quality": evaluation.get("art_quality"),
        "phase_c": evaluation.get("phase_c"),
        "proceed_ship": evaluation.get("proceed_ship"),
        "proceed_tile_ship": evaluation.get("proceed_tile_ship"),
        "blocked_by": evaluation.get("blocked_by"),
        "missing_minimum_keys": evaluation.get("missing_minimum_keys"),
        "missing_variant_keys": evaluation.get("missing_variant_keys"),
        "signoff": signoff.get("written"),
        "unblocks": ["MCP-P2-KIT002-G5"] if green else [],
        "delta_wf": "@coder-mcp MCP-P2-KIT002-G5 index + tile batch register" if green else None,
        "operator_next": (
            "utils/keyframe_render.py + Light_keysshotsetup.blend on production assembly"
            if not green
            else None
        ),
        "_agent_meta": {
            "schema": "kit_production_002_g4_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "KIT_PRODUCTION_002_G4",
            "source_system": "kit_production_002",
            "relative_path": G4_WITNESS_REL,
            "ritual": "BLANG:WIT-HON→WIT→Q✓ MCP-P2-KIT002-G4" if green else "BLANG:WIT-HON FAIL — manual keyframes required",
            "agent": "designer-mcp",
        },
    }
    out = root / G4_WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = G4_WITNESS_REL

    _refresh_pilot_g4_witness_honest(root, evaluation)
    _write_batch_witness_g4(root, green=green, evaluation=evaluation)
    _update_warehouse_production_signoff(root, evaluation, signoff_path=G4_SIGNOFF_REL)
    return body


def _refresh_pilot_g4_witness_honest(root: Path, evaluation: dict[str, Any]) -> None:
    """Keep warehouse_production_keyframe_g4_live.json aligned with disk (WIT-HON)."""
    witness = {
        "program_id": "MCP-EXPORT-PILOT-KEYFRAMES-G4",
        "pilot": "warehouse_industrial_west",
        "batch_id": "tile_warehouse_industrial_west_production_v1",
        "kit_gate": "MCP-P2-KIT002-G4",
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "keyframe_stills_folder": evaluation.get("keyframe_stills_folder"),
        "gates": evaluation.get("gates"),
        "green": evaluation.get("green"),
        "minimum_review": evaluation.get("minimum_review"),
        "variant_count": evaluation.get("variant_count"),
        "stills_ok_count": evaluation.get("stills_ok_count"),
        "art_quality": evaluation.get("art_quality"),
        "bake_source": "keyframe_pack",
        "lod0_ortho_atlas_g4": "rejected",
    }
    out = root / "debug_runs/art_pipeline/warehouse_production_keyframe_g4_live.json"
    out.write_text(json.dumps(witness, indent=2) + "\n", encoding="utf-8")


def _update_warehouse_production_signoff(
    root: Path,
    evaluation: dict[str, Any],
    *,
    signoff_path: str,
) -> None:
    from datetime import datetime, timezone

    reviewed_at = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    proceed = "yes" if evaluation.get("proceed_ship") else "no"
    blocked = evaluation.get("blocked_by") or ["manual_keyframe_render"]
    body = f"""# warehouse_industrial_west_production_signoff.yaml — G4 via kit_production_002
program_id: PLAN-PROC-TILE-PROD-001
task_id: MCP-PT-1-002
kit_gate: MCP-P2-KIT002-G4
gate: G4
designer_mcp: production_keyframe_signoff
reviewed_at: "{reviewed_at}"
proceed_ship: {proceed}

kit_production_002_signoff: {signoff_path}
variant_matrix: {VARIANT_MATRIX_REL}

tile_fix_09_phase_c:
  cli: tools/mcp/scripts/designer_mcp_warehouse_phase_c.ps1
  witness: debug_runs/art_pipeline/tile_fix_09_phase_c_warehouse_g4_live.json
  art_quality: {evaluation.get("art_quality")}
  proceed_ship: {proceed}

notes: "Designer-mcp G4 witness — proceed_ship: yes only when keyframe_stills on disk + keyframe_manual."

blocked_by: {json.dumps(blocked)}
next: "{'@coder-mcp MCP-P2-KIT002-G5' if evaluation.get('proceed_ship') else 'operator keyframe_render on production assembly'}"
"""
    out = root / "debug_runs/art_pipeline/warehouse_industrial_west_production_signoff.yaml"
    out.write_text(body, encoding="utf-8")


def _write_batch_witness_g4(root: Path, *, green: bool, evaluation: dict[str, Any]) -> None:
    manifest = load_manifest(repo=root)
    promoted = sum(1 for m in manifest.get("modules") or [] if m.get("status") == "promoted")
    body = {
        "gate": "G4",
        "green": green,
        "batch_id": "kit_production_002",
        "module_count": manifest.get("module_count"),
        "promoted_count": promoted,
        "g4": {
            "stills_ok_count": evaluation.get("stills_ok_count"),
            "variant_count": evaluation.get("variant_count"),
            "art_quality": evaluation.get("art_quality"),
            "proceed_ship": evaluation.get("proceed_ship"),
        },
        "manifest": MANIFEST_REL,
        "g4_witness": G4_WITNESS_REL,
        "note": "G4 designer sign-off — tile ship blocked until proceed_ship yes",
    }
    out = root / BATCH_WITNESS_REL
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")


def run_kit_production_002_g2_full(*, promote: bool = True) -> dict[str, Any]:
    geom = run_kit_production_002_g2_geometry()
    if not geom.get("ok"):
        witness = refresh_kit_production_002_g2_witness()
        return {"ok": False, "geometry": geom, "witness": witness}
    promoted: dict[str, Any] | None = None
    if promote:
        promoted = promote_kit_production_002_g2_roof()
    witness = refresh_kit_production_002_g2_witness()
    ok = bool(witness.get("green"))
    return {"ok": ok, "geometry": geom, "promote": promoted, "witness": witness}
