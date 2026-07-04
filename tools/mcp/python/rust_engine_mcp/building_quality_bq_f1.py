"""BQ-F1-BAKE-001 — roof flush + wall sill; rebake/promote witness."""

from __future__ import annotations

import json
import struct
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

TASK_ID = "BQ-F1-BAKE-001"
WITNESS_REL = "debug_runs/building_quality_bq_f1_live.json"
FLAT_WALL_PROFILES = frozenset({"flat", "panel", "panel_flat"})
ROOF_BOTTOM_AXIS = 2
ROOF_BOTTOM_EPS = 0.002
WALL_DEPTH_AXIS = 1
WALL_DEPTH_EPS = 0.004


def _examples_dir(repo: Path) -> Path:
    return repo / "tools/mcp/schemas/examples"


def _job_path(job_id: str, *, repo: Path) -> Path:
    return _examples_dir(repo) / f"{job_id}.json"


def _promoted_glb(job_id: str, *, repo: Path) -> Path:
    return repo / "assets/models/modules" / job_id / "model.glb"


def _staging_glb(job_id: str, *, repo: Path) -> Path:
    return repo / "assets/staging" / job_id / "model.glb"


def _load_job(job_id: str, *, repo: Path) -> dict[str, Any]:
    path = _job_path(job_id, repo=repo)
    if not path.is_file():
        raise FileNotFoundError(path)
    return json.loads(path.read_text(encoding="utf-8"))


def _wall_profile(params: dict[str, Any]) -> str:
    raw = params.get("profile") or params.get("panel")
    if raw is None:
        return "recess"
    return str(raw).lower()


def _is_bq_f1_wall_job(job: dict[str, Any]) -> bool:
    if str(job.get("operation") or "") != "module_wall":
        return False
    params = job.get("params") or {}
    raw = params.get("profile") or params.get("panel")
    if raw is None:
        return False
    return str(raw).lower() in FLAT_WALL_PROFILES


def _is_bq_f1_roof_job(job: dict[str, Any]) -> bool:
    return str(job.get("operation") or "") == "module_roof"


def discover_bq_f1_job_ids(*, repo: Path | None = None) -> list[str]:
    """Curated BQ-F1 rebake set: all module_roof + flat-profile module_wall run001 jobs."""
    root = repo or repo_root()
    jobs: set[str] = set()
    for path in sorted(_examples_dir(root).glob("*run001.json")):
        try:
            job = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            continue
        job_id = str(job.get("job_id") or path.stem)
        if _is_bq_f1_roof_job(job) or _is_bq_f1_wall_job(job):
            jobs.add(job_id)
    return sorted(jobs)


def audit_bpy_sources(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    wall_src = (root / "tools/mcp/blender/scripts/ops/module_wall.py").read_text(encoding="utf-8")
    roof_src = (root / "tools/mcp/blender/scripts/ops/module_roof.py").read_text(encoding="utf-8")
    issues: list[str] = []
    if "d * 1.05" in wall_src or "d*1.05" in wall_src:
        issues.append("module_wall.py still contains d*1.05 sill depth factor")
    if "_snap_roof_seat_plane_to_y_zero" not in roof_src:
        issues.append("module_roof.py missing _snap_roof_seat_plane_to_y_zero helper")
    if "build(params" in roof_src and "_snap_roof_seat_plane_to_y_zero(obj)" not in roof_src:
        issues.append("module_roof.build() does not snap roof seat plane to Y=0")
    return {"green": not issues, "issues": issues}


def _read_glb_json(path: Path) -> dict[str, Any] | None:
    if not path.is_file():
        return None
    data = path.read_bytes()
    if len(data) < 12 or data[:4] != b"glTF":
        return None
    offset = 12
    while offset + 8 <= len(data):
        chunk_len, chunk_type = struct.unpack("<I4s", data[offset : offset + 8])
        offset += 8
        chunk = data[offset : offset + chunk_len]
        offset += chunk_len
        if chunk_type == b"JSON":
            return json.loads(chunk.decode("utf-8"))
    return None


def glb_position_bounds(path: Path) -> dict[str, Any] | None:
    gltf = _read_glb_json(path)
    if gltf is None:
        return None
    accessors = gltf.get("accessors") or []
    meshes = gltf.get("meshes") or []
    for mesh in meshes:
        for prim in mesh.get("primitives") or []:
            acc_idx = prim.get("attributes", {}).get("POSITION")
            if acc_idx is None or acc_idx >= len(accessors):
                continue
            acc = accessors[acc_idx]
            min_v = acc.get("min")
            max_v = acc.get("max")
            if min_v and max_v:
                return {"min": min_v, "max": max_v}
    return None


def _roof_bottom_ok(bounds: dict[str, Any] | None) -> bool:
    if not bounds:
        return False
    min_v = bounds.get("min") or []
    if len(min_v) <= ROOF_BOTTOM_AXIS:
        return False
    return float(min_v[ROOF_BOTTOM_AXIS]) >= -ROOF_BOTTOM_EPS


def _wall_depth_ok(job: dict[str, Any], bounds: dict[str, Any] | None) -> bool:
    if not bounds:
        return False
    depth_m = float((job.get("params") or {}).get("depth_m") or 0.0)
    if depth_m <= 0:
        return True
    min_v = bounds.get("min") or []
    max_v = bounds.get("max") or []
    if len(min_v) <= WALL_DEPTH_AXIS or len(max_v) <= WALL_DEPTH_AXIS:
        return False
    half = depth_m * 0.5
    extent = max(abs(float(min_v[WALL_DEPTH_AXIS])), abs(float(max_v[WALL_DEPTH_AXIS])))
    return extent <= half + WALL_DEPTH_EPS


def _job_geometry_ok(job: dict[str, Any], glb: Path) -> dict[str, Any]:
    bounds = glb_position_bounds(glb)
    if _is_bq_f1_roof_job(job):
        ok = _roof_bottom_ok(bounds)
        detail = {"check": "roof_bottom_flush", "axis": ROOF_BOTTOM_AXIS, "bounds": bounds}
    elif _is_bq_f1_wall_job(job):
        ok = _wall_depth_ok(job, bounds)
        depth_m = float((job.get("params") or {}).get("depth_m") or 0.0)
        detail = {
            "check": "wall_depth_exact",
            "axis": WALL_DEPTH_AXIS,
            "depth_m": depth_m,
            "bounds": bounds,
        }
    else:
        ok = False
        detail = {"check": "unknown"}
    return {"ok": ok, **detail}


def rebake_job(job_id: str, *, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp import blender_runner

    root = repo or repo_root()
    job_path = _job_path(job_id, repo=root)
    result = blender_runner.run_geometry_job(job_path)
    staging = _staging_glb(job_id, repo=root)
    return {
        "job_id": job_id,
        "ok": result.status == "done" and staging.is_file(),
        "status": result.status,
        "staging_glb": staging.is_file(),
        "error": getattr(result, "error", None),
    }


def promote_job(job_id: str, *, repo: Path | None = None, register: bool = True) -> dict[str, Any]:
    from rust_engine_mcp import promote
    from rust_engine_mcp.validators.asset import validate_asset_glb

    root = repo or repo_root()
    staging = _staging_glb(job_id, repo=root)
    if not staging.is_file():
        raise FileNotFoundError(f"staging glb missing: {staging}")
    report = validate_asset_glb(staging, compression_level=1)
    if report.status == "failed":
        raise RuntimeError(f"asset validation failed for {job_id}: {report.summary}")
    manifest = promote.promote_module(job_id, register=register)
    promoted = _promoted_glb(job_id, repo=root)
    return {
        "job_id": job_id,
        "ok": promoted.is_file(),
        "promoted_glb": str(promoted.relative_to(root)).replace("\\", "/"),
        "manifest": manifest,
    }


def run_bq_f1_rebake_promote(
    *,
    repo: Path | None = None,
    job_ids: list[str] | None = None,
    register: bool = True,
) -> dict[str, Any]:
    from rust_engine_mcp.library import write_module_index

    root = repo or repo_root()
    targets = job_ids or discover_bq_f1_job_ids(repo=root)
    rebakes: list[dict[str, Any]] = []
    promotes: list[dict[str, Any]] = []
    errors: list[str] = []

    for job_id in targets:
        try:
            rebake = rebake_job(job_id, repo=root)
            rebakes.append(rebake)
            if not rebake.get("ok"):
                errors.append(f"{job_id}: rebake failed ({rebake.get('status')})")
                continue
            promo = promote_job(job_id, repo=root, register=register)
            promotes.append(promo)
            if not promo.get("ok"):
                errors.append(f"{job_id}: promote failed")
        except (FileNotFoundError, RuntimeError, OSError) as exc:
            errors.append(f"{job_id}: {exc}")
            rebakes.append({"job_id": job_id, "ok": False, "error": str(exc)})

    index = write_module_index()
    return {
        "task_id": TASK_ID,
        "job_count": len(targets),
        "rebake_ok": sum(1 for r in rebakes if r.get("ok")),
        "promote_ok": sum(1 for p in promotes if p.get("ok")),
        "errors": errors,
        "rebakes": rebakes,
        "promotes": promotes,
        "index_entries": index.get("entry_count"),
    }


def bq_f1_status(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    audit = audit_bpy_sources(repo=root)
    rows: list[dict[str, Any]] = []
    for job_id in discover_bq_f1_job_ids(repo=root):
        job = _load_job(job_id, repo=root)
        promoted = _promoted_glb(job_id, repo=root)
        staging = _staging_glb(job_id, repo=root)
        geom = _job_geometry_ok(job, promoted) if promoted.is_file() else {"ok": False, "check": "missing_glb"}
        rows.append(
            {
                "job_id": job_id,
                "operation": job.get("operation"),
                "profile": _wall_profile(job.get("params") or {}) if job.get("operation") == "module_wall" else job.get("params", {}).get("profile"),
                "promoted_glb": promoted.is_file(),
                "staging_glb": staging.is_file(),
                "geometry_ok": geom.get("ok"),
                "geometry_check": geom.get("check"),
            }
        )
    promoted_ok = all(r.get("promoted_glb") for r in rows)
    geometry_ok = all(r.get("geometry_ok") for r in rows) if rows else False
    return {
        "task_id": TASK_ID,
        "audit": audit,
        "job_count": len(rows),
        "promoted_ok": promoted_ok,
        "geometry_ok": geometry_ok,
        "rows": rows,
    }


def write_bq_f1_witness(*, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    root = repo or repo_root()
    status = bq_f1_status(repo=root)
    audit = status.get("audit") or {}
    green = bool(
        audit.get("green")
        and status.get("promoted_ok")
        and status.get("geometry_ok")
        and status.get("job_count", 0) > 0
    )
    body: dict[str, Any] = {
        "task_id": TASK_ID,
        "green": green,
        "bpy_source_audit_green": audit.get("green"),
        "bpy_source_issues": audit.get("issues") or [],
        "promoted_ok": status.get("promoted_ok"),
        "geometry_ok": status.get("geometry_ok"),
        "job_count": status.get("job_count"),
        "rows": status.get("rows") or [],
        "plan_ref": "src/dev/plan_building_quality_v1.md#BQ-F1",
    }
    return write_aps_live_witness(
        body,
        WITNESS_REL,
        schema="building_quality_bq_f1_live_v1",
        profile="BQ_F1_BAKE",
        source_system="building_quality_bq_f1",
        ritual="BLANG:WIT-HON BQ-F1-BAKE-001" if green else None,
        repo=root,
    )
