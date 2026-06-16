"""ARCH-MAT-001 — Python-side material authority before every Blender bake job."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .assembly_build_worker import ensure_snapshot_material_textures
from .paths import repo_root


def snapshot_rel_path(snapshot_path: str | Path) -> str:
    raw = Path(snapshot_path)
    path = raw if raw.is_file() else repo_root() / raw
    try:
        return path.relative_to(repo_root()).as_posix()
    except ValueError:
        return path.as_posix()


def ensure_snapshot_for_bake(snapshot_path: str | Path) -> dict[str, Any]:
    """Generate missing PBR textures before any bake that reads material_profile."""
    from . import assembly

    path = Path(snapshot_path)
    snap_path = path if path.is_file() else repo_root() / path
    snapshot = assembly.load_assembly_snapshot(snap_path)
    mat = ensure_snapshot_material_textures(snapshot)
    return {
        "snapshot": snap_path,
        "snapshot_rel": snapshot_rel_path(snap_path),
        "assembly_id": str(snapshot.get("assembly_id") or snap_path.stem),
        "materials": mat,
        "ok": bool(mat.get("ok")),
    }


def annotate_tile_bake_job(
    job: dict[str, Any],
    *,
    snapshot_path: str | Path | None = None,
    ensure_textures: bool = True,
) -> dict[str, Any]:
    """Inject assembly_snapshot + ensure textures on tile_variant_bake / assembly_build jobs."""
    out = dict(job)
    snap_rel = out.get("assembly_snapshot")
    if snapshot_path is not None:
        prep = ensure_snapshot_for_bake(snapshot_path) if ensure_textures else {
            "snapshot_rel": snapshot_rel_path(snapshot_path),
            "ok": True,
            "materials": {"skipped": True},
        }
        if ensure_textures and not prep.get("ok"):
            out["_material_prep_failed"] = prep
            return out
        out["assembly_snapshot"] = prep["snapshot_rel"]
        out["material_authority"] = "snapshot material_profile"
    elif snap_rel and ensure_textures:
        prep = ensure_snapshot_for_bake(snap_rel)
        if not prep.get("ok"):
            out["_material_prep_failed"] = prep
            return out
    return out


def write_bake_job(job_path: Path, job: dict[str, Any]) -> Path:
    job_path.parent.mkdir(parents=True, exist_ok=True)
    job_path.write_text(json.dumps(job, indent=2) + "\n", encoding="utf-8")
    return job_path
