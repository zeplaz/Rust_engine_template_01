"""Blender headless — import assembly snapshot GLBs into collection ASSEMBLY."""

from __future__ import annotations

import json
from pathlib import Path

import bpy


def _repo_from_job(job_path: Path) -> Path:
    repo_guess = job_path
    for _ in range(10):
        if (repo_guess / "Cargo.toml").is_file():
            return repo_guess
        if repo_guess.parent == repo_guess:
            break
        repo_guess = repo_guess.parent
    return job_path.parent


def _reset_scene() -> None:
    bpy.ops.wm.read_factory_settings(use_empty=True)


def _ensure_collection(name: str) -> bpy.types.Collection:
    coll = bpy.data.collections.get(name)
    if coll is None:
        coll = bpy.data.collections.new(name)
        bpy.context.scene.collection.children.link(coll)
    return coll


def _meshes_in_hierarchy(root: bpy.types.Object) -> list:
    meshes: list = []
    if root.type == "MESH":
        meshes.append(root)
    for child in root.children_recursive:
        if child.type == "MESH":
            meshes.append(child)
    return meshes


def _import_glb(glb_path: Path, collection: bpy.types.Collection) -> bpy.types.Object:
    bpy.ops.import_scene.gltf(filepath=str(glb_path))
    objs = list(bpy.context.selected_objects)
    if not objs:
        raise RuntimeError(f"GLB import produced no objects: {glb_path}")
    root = objs[0]
    for obj in objs:
        if obj.name not in collection.objects:
            collection.objects.link(obj)
        if obj.users_collection:
            for uc in list(obj.users_collection):
                if uc != collection:
                    uc.objects.unlink(obj)
    return root


def build(job: dict, *, repo_root: Path) -> Path:
    snap_rel = job["assembly_snapshot"]
    snap_path = Path(snap_rel)
    if not snap_path.is_absolute():
        snap_path = (repo_root / snap_rel).resolve()
    snapshot = json.loads(snap_path.read_text(encoding="utf-8"))

    _reset_scene()
    assembly_coll = _ensure_collection("ASSEMBLY")

    for placement in snapshot.get("module_placements") or []:
        glb_rel = placement.get("glb_path") or ""
        glb_path = Path(glb_rel)
        if not glb_path.is_absolute():
            glb_path = (repo_root / glb_rel).resolve()
        if not glb_path.is_file():
            raise FileNotFoundError(f"Missing GLB: {glb_path}")
        root = _import_glb(glb_path, assembly_coll)
        new_meshes = _meshes_in_hierarchy(root)
        pos = placement.get("position") or [0.0, 0.0, 0.0]
        rot = placement.get("rotation_euler") or [0.0, 0.0, 0.0]
        root.location = (float(pos[0]), float(pos[2]), float(pos[1]))
        root.rotation_euler = (float(rot[0]), float(rot[1]), float(rot[2]))
        root.name = f"{placement.get('module_id', 'mod')}_{placement.get('grid_x')}_{placement.get('grid_y')}"
        profile = placement.get("material_profile")
        if profile and new_meshes:
            try:
                from ops.export_glb import apply_material_profile_to_meshes

                mode = apply_material_profile_to_meshes(
                    new_meshes,
                    material_profile=str(profile),
                    repo_root=repo_root,
                )
                print(f"ASSEMBLY_MATERIAL {root.name} {profile} mode={mode}")
            except Exception as exc:  # noqa: BLE001
                print(f"ASSEMBLY_MATERIAL_WARN {root.name} {profile}: {exc}")

    from ops.material_authority import apply_snapshot_material_profiles

    apply_snapshot_material_profiles(snapshot, repo_root=repo_root)

    # Assembly blends are ASSEMBLY-only. Iso rig (utils/Tile_iso_rig_v1.blend) is appended at bake/render time.

    out = job.get("output") or {}
    blend_rel = out.get("blend")
    if not blend_rel:
        raise ValueError("assembly_build output.blend required")
    blend_path = Path(blend_rel)
    if not blend_path.is_absolute():
        blend_path = (repo_root / blend_rel).resolve()
    blend_path.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.wm.save_as_mainfile(filepath=str(blend_path))
    print(f"ASSEMBLY_BLEND_OK {blend_path}")
    return blend_path
