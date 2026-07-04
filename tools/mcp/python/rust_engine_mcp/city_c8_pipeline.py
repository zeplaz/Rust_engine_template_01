"""CITY-C8-PIPELINE-001 — bake-time module variant merge witness."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from rust_engine_mcp.glb_bounds import glb_position_bounds
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.schemas import load_json_file, validate_geometry_job

TASK_ID = "CITY-C8-PIPELINE-001"
WITNESS_REL = "debug_runs/city_c8_pipeline_001_live.json"
JOB_REL = "tools/mcp/schemas/examples/city_c8_pilot_merge_run001.json"
STAGING_GLB = "assets/staging/city_c8_pilot_merge_run001/model.glb"


def pilot_job(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    job = load_json_file(root / JOB_REL)
    validate_geometry_job(job)
    return job


def run_city_c8_merge(*, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp import blender_runner

    root = repo or repo_root()
    job_path = root / JOB_REL
    job = pilot_job(repo=root)
    for part in job.get("params", {}).get("parts") or []:
        glb = root / str(part.get("glb_path"))
        if not glb.is_file():
            return {"ok": False, "error": f"missing_part_glb:{glb}"}

    result = blender_runner.run_geometry_job(job_path)
    out = root / STAGING_GLB
    ok = out.is_file() and result.status == "done"
    bounds = glb_position_bounds(out) if ok else None
    return {
        "ok": ok,
        "job_id": job.get("job_id"),
        "staging_glb": STAGING_GLB,
        "bounds": bounds,
        "blender_status": result.status,
        "blender_error": result.error,
    }


def write_city_c8_witness(*, repo: Path | None = None, run_merge: bool = True) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    root = repo or repo_root()
    op_path = root / "tools/mcp/blender/scripts/ops/module_variant_merge.py"
    run_job = root / "tools/mcp/blender/scripts/run_job.py"
    schema_ok = False
    try:
        pilot_job(repo=root)
        schema_ok = True
    except Exception as exc:  # noqa: BLE001
        schema_error = str(exc)
    else:
        schema_error = None

    merge_result: dict[str, Any] = {"ok": False, "skipped": True}
    if run_merge and op_path.is_file():
        try:
            merge_result = run_city_c8_merge(repo=root)
        except Exception as exc:  # noqa: BLE001
            merge_result = {"ok": False, "error": str(exc)}

    op_registered = "module_variant_merge" in run_job.read_text(encoding="utf-8")
    green = schema_ok and op_registered and bool(merge_result.get("ok"))
    body: dict[str, Any] = {
        "task_id": TASK_ID,
        "gate": TASK_ID,
        "green": green,
        "schema_ok": schema_ok,
        "schema_error": schema_error,
        "op_registered": op_registered,
        "job_path": JOB_REL,
        "merge": merge_result,
        "plan_ref": "src/dev/plan_city_grammar_upgrade_v1.md#CITY-C8",
    }
    return write_aps_live_witness(
        body,
        WITNESS_REL,
        schema="city_c8_pipeline_live_v1",
        profile="CITY_C8_PIPELINE",
        source_system="city_c8_pipeline",
        ritual=f"BLANG:WIT-HON {TASK_ID}" if green else None,
        repo=root,
    )


def write_bq_f1_tail_witness(*, repo: Path | None = None) -> dict[str, Any]:
    """BQ-F1-TAIL-001 — flat-profile brick lod0 tier path unblocked."""
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness
    from rust_engine_mcp.validators.tier import resolve_asset_context, tier_issues_for_asset

    root = repo or repo_root()
    tail_jobs = [
        "wall_brick_1u_lod0_run001",
        "wall_brick_2u_lod0_run001",
    ]
    rows: list[dict[str, Any]] = []
    for job_id in tail_jobs:
        staging = root / "assets/staging" / job_id / "model.glb"
        row: dict[str, Any] = {"job_id": job_id, "staging_glb": staging.is_file()}
        if staging.is_file():
            ctx = resolve_asset_context(staging)
            issues = tier_issues_for_asset(ctx, vertex_count=ctx.vertex_count)
            sil = [i for i in issues if i.kind == "SilhouetteInsufficient"]
            row["silhouette_blocked"] = bool(sil)
            row["tier_ok"] = not sil
        rows.append(row)

    green = all(r.get("tier_ok") for r in rows if r.get("staging_glb"))
    body = {
        "task_id": "BQ-F1-TAIL-001",
        "gate": "BQ-F1-TAIL-001",
        "green": green,
        "rows": rows,
        "fix": "explicit flat profile wins over wall_brick id hint in tier TIER-002",
    }
    return write_aps_live_witness(
        body,
        "debug_runs/bq_f1_tail_001_live.json",
        schema="building_quality_bq_f1_tail_live_v1",
        profile="BQ_F1_TAIL",
        source_system="city_c8_pipeline",
        ritual="BLANG:WIT-HON BQ-F1-TAIL-001" if green else None,
        repo=root,
    )
