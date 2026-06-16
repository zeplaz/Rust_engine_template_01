"""ARCH-MAT-001 / BUILD-WORKER-001 — snapshot material_profile authority on every bpy path."""

from __future__ import annotations

import json
from pathlib import Path

import bpy

from ops.export_glb import apply_material_profile_to_meshes


def _meshes_in_hierarchy(root: bpy.types.Object) -> list:
    meshes: list = []
    if root.type == "MESH":
        meshes.append(root)
    for child in root.children_recursive:
        if child.type == "MESH":
            meshes.append(child)
    return meshes


def _placement_object_name(placement: dict) -> str:
    module_id = str(placement.get("module_id") or "mod")
    gx = placement.get("grid_x", 0)
    gy = placement.get("grid_y", 0)
    return f"{module_id}_{gx}_{gy}"


def _find_placement_root(placement: dict) -> bpy.types.Object | None:
    """Match assembly_import root naming: {module_id}_{grid_x}_{grid_y}."""
    want = _placement_object_name(placement)
    obj = bpy.data.objects.get(want)
    if obj is not None:
        return obj
    module_id = str(placement.get("module_id") or "")
    if not module_id:
        return None
    prefix = f"{module_id}_"
    for candidate in bpy.data.objects:
        if candidate.name.startswith(prefix) and candidate.type in ("EMPTY", "MESH"):
            if candidate.parent is None:
                return candidate
    return None


def apply_snapshot_material_profiles(
    snapshot: dict,
    *,
    repo_root: Path,
) -> dict[str, str]:
    """Re-apply material_profile from snapshot onto ASSEMBLY scene (pre-bake authority)."""
    applied: dict[str, str] = {}
    for placement in snapshot.get("module_placements") or []:
        profile = str(placement.get("material_profile") or "").strip()
        if not profile:
            continue
        root = _find_placement_root(placement)
        if root is None:
            print(f"ARCH_MAT_WARN missing object {_placement_object_name(placement)}")
            continue
        meshes = _meshes_in_hierarchy(root)
        if not meshes:
            continue
        mode = apply_material_profile_to_meshes(
            meshes,
            material_profile=profile,
            repo_root=repo_root,
        )
        key = root.name
        applied[key] = f"{profile}:{mode}"
        print(f"ARCH_MAT_APPLY {key} {profile} mode={mode}")
    return applied


def apply_snapshot_from_path(snapshot_path: Path, *, repo_root: Path) -> dict[str, str]:
    snapshot = json.loads(snapshot_path.read_text(encoding="utf-8"))
    return apply_snapshot_material_profiles(snapshot, repo_root=repo_root)


def apply_from_job(job: dict, *, repo_root: Path) -> dict[str, str]:
    """Apply when job carries assembly_snapshot (tile_variant_bake / assembly_build)."""
    snap_rel = job.get("assembly_snapshot") or ""
    if not snap_rel:
        return {}
    snap_path = Path(str(snap_rel))
    if not snap_path.is_absolute():
        snap_path = (repo_root / snap_rel).resolve()
    if not snap_path.is_file():
        print(f"ARCH_MAT_WARN snapshot missing {snap_path}")
        return {}
    return apply_snapshot_from_path(snap_path, repo_root=repo_root)
