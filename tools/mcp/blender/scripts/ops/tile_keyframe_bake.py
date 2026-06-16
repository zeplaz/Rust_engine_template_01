"""Headless keyframe still export — Tile_iso_rig_v1 (TILE_ISO_RIG) + variant state. CI only — manual keyframe_render for ship.

Optional until manual keyframe path is G4-green. Enable via RUST_ENGINE_TILE_KEYFRAME_HEADLESS=1
on keyframe_pack batches (render.method blender_keyframe_light_rig).
"""

from __future__ import annotations

import math
from pathlib import Path

import bpy

from ops.tile_ortho_bake import (
    _append_light_rig,
    _apply_variant_to_assembly,
    _ensure_iso_camera,
    _frame_assembly_camera,
    _setup_render,
)


def _variant_frame(variant: dict) -> int:
    if variant.get("fire_frame") is not None:
        return int(variant["fire_frame"]) + 1
    return 1


def _assembly_root_objects() -> list[bpy.types.Object]:
    """Top-level assembly roots — ASSEMBLY collection or unparented meshes."""
    coll = bpy.data.collections.get("ASSEMBLY")
    if coll is not None:
        roots = [o for o in coll.all_objects if o.parent is None and o.type != "EMPTY"]
        if roots:
            return roots
    return [o for o in bpy.data.objects if o.type == "MESH" and o.parent is None]


def _apply_facing_yaw(yaw_deg: float) -> None:
    """Rotate assembly around footprint center for iso facing bake (TILE-FIX-008)."""
    from ops.tile_ortho_bake import _assembly_mesh_objects, _world_bounds

    roots = _assembly_root_objects()
    meshes = _assembly_mesh_objects()
    if not roots or not meshes:
        print("TILE_KEYFRAME_WARN facing_yaw skipped — no assembly roots/meshes")
        return

    mins, maxs = _world_bounds(meshes)
    center = (mins + maxs) * 0.5
    rad = math.radians(float(yaw_deg))

    pivot = bpy.data.objects.get("ASSEMBLY_FACING_PIVOT")
    if pivot is None:
        coll = bpy.data.collections.get("ASSEMBLY")
        pivot = bpy.data.objects.new("ASSEMBLY_FACING_PIVOT", None)
        if coll is not None:
            coll.objects.link(pivot)
        else:
            bpy.context.scene.collection.objects.link(pivot)
        pivot.location = center
        for obj in roots:
            if obj == pivot:
                continue
            world = obj.matrix_world.copy()
            obj.parent = pivot
            obj.matrix_world = world

    pivot.rotation_euler[2] = rad


def _apply_night_emission(variant: dict) -> None:
    """Boost window emissive for PG-3 / G4 night-on reads (tactical brightness)."""
    lighting = str(variant.get("lighting") or "day")
    power = str(variant.get("power") or "off")
    if lighting not in ("night_on", "night_off") or power not in ("on", "partial"):
        return
    strength = float(variant.get("emissive_strength") or 0.9)
    if lighting == "night_off":
        strength *= 0.15
    for mat in bpy.data.materials:
        if not mat.use_nodes:
            continue
        bsdf = mat.node_tree.nodes.get("Principled BSDF")
        if not bsdf:
            continue
        col = bsdf.inputs["Base Color"].default_value
        bsdf.inputs["Emission Color"].default_value = (
            min(1.0, col[0] + 0.25 * strength),
            min(1.0, col[1] + 0.18 * strength),
            min(1.0, col[2] + 0.08 * strength),
            1.0,
        )
        bsdf.inputs["Emission Strength"].default_value = strength


def bake(job: dict, *, repo_root: Path) -> Path:
    """Render one variant still using appended light rig (perspective), not ortho override."""
    mode = str(job.get("mode") or "assembly")
    if mode != "assembly":
        raise ValueError("tile_keyframe_bake requires mode assembly")

    variant = dict(job.get("variant") or {})
    render = dict(job.get("render") or {})
    tile_px = int(render.get("tile_size_px") or 128)
    elev = float(render.get("camera_elevation_deg") or 35.264)

    out = job.get("output") or {}
    png_rel = out.get("png")
    if not png_rel:
        raise ValueError("tile_keyframe_bake output.png required")
    png_path = Path(png_rel)
    if not png_path.is_absolute():
        png_path = (repo_root / png_rel).resolve()
    png_path.parent.mkdir(parents=True, exist_ok=True)

    light_rel = job.get("light_blend") or ""
    light_path = Path(light_rel) if light_rel else Path()
    if light_rel and not light_path.is_absolute():
        light_path = (repo_root / light_rel).resolve()

    blend_rel = job.get("assembly_blend") or ""
    blend_path = Path(blend_rel)
    if not blend_path.is_absolute():
        blend_path = (repo_root / blend_rel).resolve()
    if not blend_path.is_file():
        raise FileNotFoundError(f"Missing assembly blend: {blend_path}")

    bpy.ops.wm.open_mainfile(filepath=str(blend_path))
    from ops.material_authority import apply_from_job

    applied = apply_from_job(job, repo_root=repo_root)
    if applied:
        print(f"ARCH_MAT_KEYFRAME applied={len(applied)}")
    _apply_variant_to_assembly(variant)
    _apply_night_emission(variant)
    facing_yaw = render.get("facing_yaw_deg")
    if facing_yaw is not None:
        _apply_facing_yaw(float(facing_yaw))

    if light_path.is_file():
        _append_light_rig(light_path)

    # Always re-frame assembly (fixes bunker 6×3 edge-on when rig camera predates import).
    _frame_assembly_camera(elev)
    if bpy.context.scene.camera is None:
        _ensure_iso_camera(elev)

    bpy.context.scene.frame_set(_variant_frame(variant))
    _setup_render(tile_px)
    bpy.context.scene.render.filepath = str(png_path.with_suffix(""))
    bpy.ops.render.render(write_still=True)
    print(f"TILE_KEYFRAME_OK {png_path}")
    return png_path
