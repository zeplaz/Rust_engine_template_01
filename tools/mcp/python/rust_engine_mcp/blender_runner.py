"""Run Blender headless geometry jobs + job status files."""

from __future__ import annotations

import json
import subprocess
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .paths import art_pipeline_log_dir, blender_exe, blender_scripts_dir, jobs_root, repo_root
from .schemas import (
    load_json_file,
    validate_assembly_build_job,
    validate_geometry_job,
    validate_tile_variant_bake_job,
)


@dataclass
class JobResult:
    job_id: str
    status: str
    log_path: str
    outputs: list[str]
    error: str | None = None


def _status_path(job_id: str) -> Path:
    return jobs_root() / f"{job_id}.status.json"


def write_status(job_id: str, payload: dict[str, Any]) -> None:
    payload["updated_at"] = time.time()
    _status_path(job_id).write_text(json.dumps(payload, indent=2), encoding="utf-8")


def read_status(job_id: str) -> dict[str, Any] | None:
    p = _status_path(job_id)
    if not p.is_file():
        return None
    return json.loads(p.read_text(encoding="utf-8"))


def _enforce_seed_in_params(job: dict[str, Any]) -> None:
    params = job.get("params") or {}
    if "seed" in params:
        return
    variation_keys = {"material_profile", "prop_kind", "style_variant"}
    if variation_keys.intersection(params.keys()):
        raise ValueError(
            "geometry job params require 'seed' when material_profile or variation fields are set"
        )


def run_geometry_job(job_path: Path) -> JobResult:
    job_path = job_path.resolve()
    job = load_json_file(job_path)
    validate_geometry_job(job)
    _enforce_seed_in_params(job)
    job_id = str(job["job_id"])

    log_dir = art_pipeline_log_dir()
    log_path = log_dir / f"{job_id}.log"
    spec_ref = job.get("spec_ref")
    write_status(
        job_id,
        {
            "job_id": job_id,
            "status": "running",
            "log_path": str(log_path),
            "outputs": [],
            "spec_ref": spec_ref,
            "job_path": str(job_path),
        },
    )

    run_script = blender_scripts_dir() / "run_job.py"
    cmd = [
        str(blender_exe()),
        "--background",
        "--python",
        str(run_script),
        "--",
        "--job",
        str(job_path),
    ]

    proc = subprocess.run(
        cmd,
        cwd=str(repo_root()),
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    log_text = (proc.stdout or "") + (proc.stderr or "")
    log_path.write_text(log_text, encoding="utf-8")

    outputs: list[str] = []
    glb = (job.get("output") or {}).get("glb")
    if glb:
        glb_path = Path(glb)
        if not glb_path.is_absolute():
            glb_path = repo_root() / glb
        if glb_path.is_file():
            outputs.append(str(glb_path.resolve()))

    ok = proc.returncode == 0 and outputs
    status = "done" if ok else "failed"
    error = None if ok else (log_text.strip()[-2000:] or f"exit {proc.returncode}")

    write_status(
        job_id,
        {
            "job_id": job_id,
            "status": status,
            "log_path": str(log_path),
            "outputs": outputs,
            "error": error,
            "spec_ref": spec_ref,
            "job_path": str(job_path),
        },
    )
    return JobResult(job_id=job_id, status=status, log_path=str(log_path), outputs=outputs, error=error)


def build_iso_rig_blend(*, procedural_only: bool = False) -> dict:
    """Build utils/Tile_iso_rig_v1.blend (camera + lights only)."""
    run_script = blender_scripts_dir() / "build_iso_rig.py"
    extra = ["--procedural-only"] if procedural_only else []
    cmd = [
        str(blender_exe()),
        "--background",
        "--python",
        str(run_script),
        "--",
        "--repo",
        str(repo_root()),
        *extra,
    ]
    proc = subprocess.run(
        cmd,
        cwd=str(repo_root()),
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    dest = repo_root() / "utils" / "Tile_iso_rig_v1.blend"
    ok = proc.returncode == 0 and dest.is_file()
    return {
        "ok": ok,
        "exit_code": proc.returncode,
        "blend_path": str(dest) if dest.is_file() else None,
        "log": ((proc.stdout or "") + (proc.stderr or "")).strip()[-4000:],
    }


def run_tile_job(job_path: Path) -> JobResult:
    """Run assembly_build or tile_variant_bake job in Blender headless."""
    job_path = job_path.resolve()
    job = load_json_file(job_path)
    operation = str(job.get("operation") or "")
    if operation == "assembly_build":
        validate_assembly_build_job(job)
    elif operation == "tile_variant_bake":
        validate_tile_variant_bake_job(job)
    else:
        raise ValueError(f"Unknown tile operation: {operation!r}")

    job_id = str(job["job_id"])
    log_dir = art_pipeline_log_dir()
    log_path = log_dir / f"{job_id}.log"
    write_status(
        job_id,
        {
            "job_id": job_id,
            "status": "running",
            "log_path": str(log_path),
            "outputs": [],
            "operation": operation,
            "job_path": str(job_path),
        },
    )

    run_script = blender_scripts_dir() / "run_tile_job.py"
    cmd = [
        str(blender_exe()),
        "--background",
        "--python",
        str(run_script),
        "--",
        "--job",
        str(job_path),
    ]
    proc = subprocess.run(
        cmd,
        cwd=str(repo_root()),
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    log_text = (proc.stdout or "") + (proc.stderr or "")
    log_path.write_text(log_text, encoding="utf-8")

    outputs: list[str] = []
    out = job.get("output") or {}
    for key in ("blend", "png"):
        rel = out.get(key)
        if not rel:
            continue
        out_path = Path(rel)
        if not out_path.is_absolute():
            out_path = repo_root() / rel
        if out_path.is_file():
            outputs.append(str(out_path.resolve()))

    ok = proc.returncode == 0 and bool(outputs)
    status = "done" if ok else "failed"
    error = None if ok else (log_text.strip()[-2000:] or f"exit {proc.returncode}")

    write_status(
        job_id,
        {
            "job_id": job_id,
            "status": status,
            "log_path": str(log_path),
            "outputs": outputs,
            "error": error,
            "operation": operation,
            "job_path": str(job_path),
        },
    )
    return JobResult(job_id=job_id, status=status, log_path=str(log_path), outputs=outputs, error=error)
