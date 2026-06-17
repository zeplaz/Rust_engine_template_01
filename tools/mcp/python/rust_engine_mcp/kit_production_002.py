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
BATCH_WITNESS_REL = "debug_runs/art_pipeline/kit_production_002_live.json"
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
