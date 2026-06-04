#!/usr/bin/env python3
"""kit_production_001 — Victorian rowhouse production pilot (MCP-PROD-KIT-001)."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[3]
JOBS_DIR = ROOT / "tools" / "mcp" / "schemas" / "examples"
MANIFEST = JOBS_DIR / "batch_kit_production_001.manifest.json"
BATCH_ID = "kit_production_001"

# module_id → production job json stem
PRODUCTION_JOBS: list[tuple[str, str]] = [
    ("wall_brick_1u", "wall_brick_1u_production_run001"),
    ("corner_L", "corner_L_production_run001"),
    ("door_residential", "door_residential_production_run001"),
    ("roof_pitched_gable", "roof_pitched_gable_production_run001"),
    ("prop_chimney", "prop_chimney_production_run001"),
]


def sync_manifest() -> dict[str, Any]:
    """Ensure batch manifest lists job_id per module (witness G5)."""
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    by_id = {mid: jid for mid, jid in PRODUCTION_JOBS}
    modules = []
    for mod in manifest.get("modules") or []:
        mid = str(mod.get("module_id") or "")
        row = dict(mod)
        row["asset_id"] = mid
        row["job_id"] = by_id.get(mid, f"{mid}_production_run001")
        row["status"] = row.get("status", "spec_ready")
        modules.append(row)
    manifest["modules"] = modules
    manifest["rules_applied"] = [
        "no_ai_generated_images",
        "deterministic_output",
        "batch_processing",
        "grid_alignment",
        "canonical_module_ids_only",
        "tier_production_pbr_shipped",
    ]
    manifest["witness"] = "debug_runs/art_pipeline/kit_production_001_live.json"
    manifest["gate"] = manifest.get("gate", "G1")
    MANIFEST.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return manifest


def validate_jobs() -> None:
    sys.path.insert(0, str(ROOT / "tools" / "mcp" / "python"))
    from rust_engine_mcp.validators.asset import validate_asset_glb
    from rust_engine_mcp.validators.mcp_schema import validate_mcp_job
    from rust_engine_mcp.validators.tier import tier_issues_for_job

    for _mid, jid in PRODUCTION_JOBS:
        job_path = JOBS_DIR / f"{jid}.json"
        if not job_path.is_file():
            raise FileNotFoundError(job_path)
        jr = validate_mcp_job(job_path, compression_level=1)
        if jr.status == "failed":
            raise RuntimeError(f"job schema failed {jid}: {jr.summary}")
        job = json.loads(job_path.read_text(encoding="utf-8"))
        issues = tier_issues_for_job(job, job_path)
        if any(i.severity == "error" for i in issues):
            raise RuntimeError(f"tier job failed {jid}: {[i.hint for i in issues]}")
        glb = ROOT / "assets" / "staging" / jid / "model.glb"
        if glb.is_file():
            ar = validate_asset_glb(glb, compression_level=1)
            if ar.status == "failed":
                raise RuntimeError(f"asset validation failed {jid}: {ar.summary}")


def run_geometry() -> list[dict[str, Any]]:
    sys.path.insert(0, str(ROOT / "tools" / "mcp" / "python"))
    from rust_engine_mcp import blender_runner

    results = []
    for _mid, jid in PRODUCTION_JOBS:
        job_path = JOBS_DIR / f"{jid}.json"
        r = blender_runner.run_geometry_job(job_path)
        glb = ROOT / "assets" / "staging" / jid / "model.glb"
        results.append({"job_id": jid, "status": r.status, "glb": glb.is_file()})
        if r.status != "done" or not glb.is_file():
            raise RuntimeError(f"geometry failed {jid}: {r.status} {getattr(r, 'error', '')}")
    return results


def promote_batch() -> dict[str, Any]:
    sys.path.insert(0, str(ROOT / "tools" / "mcp" / "python"))
    from rust_engine_mcp import promote
    from rust_engine_mcp.library import write_module_index
    from rust_engine_mcp.validators.asset import validate_asset_glb

    sync_manifest()
    promoted = []
    for _mid, jid in PRODUCTION_JOBS:
        glb = ROOT / "assets" / "staging" / jid / "model.glb"
        report = validate_asset_glb(glb, compression_level=1)
        if report.status == "failed":
            raise RuntimeError(f"asset validation failed {jid}: {report.summary}")
        promoted.append(promote.promote_module(jid, register=True))
    idx = write_module_index()
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    for m in manifest["modules"]:
        m["status"] = "promoted"
    manifest["gate"] = "G5"
    MANIFEST.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    from rust_engine_mcp import witness

    wit = witness.write_batch_witness(BATCH_ID)
    return {
        "batch_id": BATCH_ID,
        "promoted": len(promoted),
        "index_entries": idx["entry_count"],
        "witness": wit,
        "G5": wit["gates"]["G5"],
    }


def run_batch(phase: str) -> dict[str, Any]:
    out: dict[str, Any] = {"batch_id": BATCH_ID, "phase": phase}
    if phase in ("sync", "g0g1", "all", "full"):
        out["manifest"] = sync_manifest()
        validate_jobs()
        out["g0_g1"] = "pass"
    if phase in ("geometry", "g2g3", "all", "full"):
        out["geometry"] = run_geometry()
    if phase in ("promote", "g4g5", "all", "full"):
        out["promote"] = promote_batch()
    return out


def main() -> None:
    p = argparse.ArgumentParser(description="kit_production_001 batch runner (MCP-PROD-KIT-001)")
    p.add_argument(
        "--phase",
        default="full",
        choices=["sync", "geometry", "promote", "full"],
        help="pipeline phase",
    )
    args = p.parse_args()
    result = run_batch(args.phase)
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
