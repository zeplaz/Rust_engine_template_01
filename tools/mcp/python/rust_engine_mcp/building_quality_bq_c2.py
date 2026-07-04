"""BQ-C2-BOUNDS-001 — G4 bounds/pivot validator vs module_contract_v1."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from rust_engine_mcp.glb_bounds import bounds_extent, glb_position_bounds, near
from rust_engine_mcp.module_contract import FLOOR_HEIGHT_M, GRID_UNIT_M, SEAM_TOLERANCE_M
from rust_engine_mcp.paths import repo_root

TASK_ID = "BQ-C2-BOUNDS-001"
WITNESS_REL = "debug_runs/bq_c2_bounds_001_live.json"
TOL = SEAM_TOLERANCE_M


def _examples_dir(repo: Path) -> Path:
    return repo / "tools/mcp/schemas/examples"


def _promoted_glb(job_id: str, *, repo: Path) -> Path:
    return repo / "assets/models/modules" / job_id / "model.glb"


def discover_promoted_run001_jobs(*, repo: Path | None = None) -> list[dict[str, Any]]:
    root = repo or repo_root()
    rows: list[dict[str, Any]] = []
    for path in sorted(_examples_dir(root).glob("*run001.json")):
        try:
            job = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            continue
        job_id = str(job.get("job_id") or path.stem)
        glb = _promoted_glb(job_id, repo=root)
        if glb.is_file():
            rows.append(job)
    return rows


def _pivot_seat_ok(bounds: dict[str, list[float]], operation: str, *, tol: float = TOL) -> bool:
    mn = bounds.get("min") or []
    mx = bounds.get("max") or []
    if len(mx) < 3:
        return False
    if operation in ("module_wall", "module_door", "module_window", "module_roof"):
        return abs(float(mx[2])) <= max(tol, 0.02) or abs(float(mn[2])) <= max(tol, 0.02)
    if operation == "module_prop":
        return float(mn[1]) >= -tol
    return True


def validate_job_bounds(job: dict[str, Any], glb: Path, *, tol: float = TOL) -> dict[str, Any]:
    params = job.get("params") or {}
    operation = str(job.get("operation") or "")
    bounds = glb_position_bounds(glb)
    job_id = str(job.get("job_id") or glb.parent.name)
    violations: list[str] = []

    if bounds is None:
        return {
            "job_id": job_id,
            "operation": operation,
            "ok": False,
            "violations": ["bounds_unreadable"],
            "bounds": None,
        }

    if not _pivot_seat_ok(bounds, operation, tol=tol):
        mx = bounds.get("max") or []
        violations.append(f"pivot_seat:max_z={mx[2] if len(mx) > 2 else '?'}")

    ex = bounds_extent(bounds)
    width_m = float(params.get("width_m") or 0.0)
    height_m = float(params.get("height_m") or 0.0)
    depth_m = float(params.get("depth_m") or 0.0)

    if operation == "module_wall" and width_m > 0 and height_m > 0:
        if not near(ex[0], width_m, tol):
            violations.append(f"width:expected={width_m},got={ex[0]:.4f}")
        if not near(ex[2], height_m, tol):
            violations.append(f"height:expected={height_m},got={ex[2]:.4f}")
        if depth_m > 0 and not near(ex[1], depth_m, tol):
            violations.append(f"depth:expected={depth_m},got={ex[1]:.4f}")
    elif operation in ("module_door", "module_window") and width_m > 0 and height_m > 0:
        if not near(ex[0], width_m, tol):
            violations.append(f"width:expected={width_m},got={ex[0]:.4f}")
        if not near(ex[2], height_m, tol):
            violations.append(f"height:expected={height_m},got={ex[2]:.4f}")
    elif operation == "module_roof":
        if height_m > 0 and not near(max(ex[0], ex[1], ex[2]), height_m, max(tol, 0.05)):
            violations.append(f"roof_extent:expected≈{height_m},got={max(ex):.4f}")

    return {
        "job_id": job_id,
        "operation": operation,
        "ok": not violations,
        "violations": violations,
        "extent_m": {"width": ex[0], "depth": ex[1], "height": ex[2]},
        "params_m": {"width": width_m, "height": height_m, "depth": depth_m},
        "bounds": bounds,
    }


def run_bounds_audit(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    jobs = discover_promoted_run001_jobs(repo=root)
    rows = [
        validate_job_bounds(job, _promoted_glb(str(job.get("job_id")), repo=root), tol=TOL)
        for job in jobs
    ]
    ok_count = sum(1 for r in rows if r.get("ok"))
    return {
        "task_id": TASK_ID,
        "contract": {
            "grid_unit_m": GRID_UNIT_M,
            "floor_height_m": FLOOR_HEIGHT_M,
            "tolerance_m": TOL,
        },
        "promoted_count": len(rows),
        "ok_count": ok_count,
        "violation_count": len(rows) - ok_count,
        "rows": rows,
    }


def write_bq_c2_witness(*, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    root = repo or repo_root()
    audit = run_bounds_audit(repo=root)
    green = bool(audit.get("promoted_count", 0) > 0 and audit.get("ok_count", 0) >= 1)
    body: dict[str, Any] = {
        **audit,
        "gate": TASK_ID,
        "green": green,
        "table_ok": audit.get("promoted_count", 0) > 0,
        "inventory_only": audit.get("violation_count", 0) > 0,
        "plan_ref": "src/dev/plan_building_quality_v1.md#BQ-C2",
    }
    return write_aps_live_witness(
        body,
        WITNESS_REL,
        schema="building_quality_bq_c2_live_v1",
        profile="BQ_C2_BOUNDS",
        source_system="building_quality_bq_c2",
        ritual=f"BLANG:WIT-HON {TASK_ID}" if green else None,
        repo=root,
    )
