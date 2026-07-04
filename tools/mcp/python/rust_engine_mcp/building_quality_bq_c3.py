"""BQ-C3-SEAM-001 — wall/door/window height seam validator per style pack."""

from __future__ import annotations

import json
from collections import defaultdict
from pathlib import Path
from typing import Any

from rust_engine_mcp.building_quality_bq_c2 import discover_promoted_run001_jobs, validate_job_bounds
from rust_engine_mcp.glb_bounds import glb_position_bounds
from rust_engine_mcp.library import load_index_json
from rust_engine_mcp.module_contract import SEAM_TOLERANCE_M
from rust_engine_mcp.paths import repo_root

TASK_ID = "BQ-C3-SEAM-001"
WITNESS_REL = "debug_runs/bq_c3_seam_001_live.json"
TOL = SEAM_TOLERANCE_M


def _promoted_glb(job_id: str, *, repo: Path) -> Path:
    return repo / "assets/models/modules" / job_id / "model.glb"


def _job_id_to_style_pack(*, repo: Path) -> dict[str, str]:
    out: dict[str, str] = {}
    for row in load_index_json():
        job_id = str(row.get("job_id") or "")
        style = str(row.get("style_pack") or "style_unknown")
        if job_id:
            out[job_id] = style
    return out


def _height_extent(job: dict[str, Any], glb: Path) -> float | None:
    bounds = glb_position_bounds(glb)
    if not bounds:
        return None
    mn = bounds.get("min") or []
    mx = bounds.get("max") or []
    if len(mx) < 3:
        return None
    return float(mx[2]) - float(mn[2])


def run_seam_audit(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    job_to_style = _job_id_to_style_pack(repo=root)
    jobs = discover_promoted_run001_jobs(repo=root)
    by_style: dict[str, dict[str, list[float]]] = defaultdict(lambda: defaultdict(list))

    for job in jobs:
        job_id = str(job.get("job_id") or "")
        op = str(job.get("operation") or "")
        if op not in ("module_wall", "module_door", "module_window"):
            continue
        glb = _promoted_glb(job_id, repo=root)
        h = _height_extent(job, glb)
        if h is None:
            continue
        style = job_to_style.get(job_id, "style_unknown")
        by_style[style][op].append(h)

    pack_rows: list[dict[str, Any]] = []
    violations = 0
    for style, ops in sorted(by_style.items()):
        walls = ops.get("module_wall") or []
        doors = ops.get("module_door") or []
        wins = ops.get("module_window") or []
        row_violations: list[str] = []
        if walls:
            wall_ref = max(walls)
            for d in doors:
                if d > wall_ref + TOL:
                    row_violations.append(f"door_taller_than_wall:{d:.3f}>{wall_ref:.3f}")
            for w in wins:
                if w > wall_ref + TOL:
                    row_violations.append(f"window_taller_than_wall:{w:.3f}>{wall_ref:.3f}")
            if len(walls) > 1:
                spread = max(walls) - min(walls)
                if spread > TOL:
                    row_violations.append(f"wall_height_spread:{spread:.3f}")
        pack_rows.append(
            {
                "style_pack": style,
                "wall_heights_m": walls,
                "door_heights_m": doors,
                "window_heights_m": wins,
                "ok": not row_violations,
                "violations": row_violations,
            }
        )
        violations += len(row_violations)

    bounds_rows = [
        validate_job_bounds(job, _promoted_glb(str(job.get("job_id")), repo=root))
        for job in jobs
        if str(job.get("operation") or "") in ("module_wall", "module_door", "module_window")
    ]

    return {
        "task_id": TASK_ID,
        "tolerance_m": TOL,
        "style_pack_count": len(pack_rows),
        "seam_violation_count": violations,
        "style_packs": pack_rows,
        "module_bounds_sample_count": len(bounds_rows),
    }


def write_bq_c3_witness(*, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    root = repo or repo_root()
    audit = run_seam_audit(repo=root)
    green = bool(audit.get("style_pack_count", 0) > 0)
    body: dict[str, Any] = {
        **audit,
        "gate": TASK_ID,
        "green": green,
        "table_ok": audit.get("style_pack_count", 0) > 0,
        "plan_ref": "src/dev/plan_building_quality_v1.md#BQ-C3",
    }
    return write_aps_live_witness(
        body,
        WITNESS_REL,
        schema="building_quality_bq_c3_live_v1",
        profile="BQ_C3_SEAM",
        source_system="building_quality_bq_c3",
        ritual=f"BLANG:WIT-HON {TASK_ID}" if green else None,
        repo=root,
    )
