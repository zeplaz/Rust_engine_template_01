"""Blender headless — orthographic iso tile bake (assembly or terrain plane)."""

from __future__ import annotations

import math
from pathlib import Path

import bpy
from mathutils import Vector

from ops.iso_rig import append_iso_rig


BASE_COLORS = {
    "wood": (0.55, 0.38, 0.22, 1.0),
    "stone": (0.5, 0.5, 0.52, 1.0),
    "concrete": (0.62, 0.62, 0.64, 1.0),
    "dirt": (0.42, 0.32, 0.22, 1.0),
    "asphalt": (0.28, 0.28, 0.3, 1.0),
    "metal_plate": (0.45, 0.48, 0.52, 1.0),
}


def _repo_from_path(start: Path) -> Path:
    repo_guess = start
    for _ in range(10):
        if (repo_guess / "Cargo.toml").is_file():
            return repo_guess
        if repo_guess.parent == repo_guess:
            break
        repo_guess = repo_guess.parent
    return start.parent


def _reset_scene() -> None:
    bpy.ops.wm.read_factory_settings(use_empty=True)


def _setup_render(tile_px: int) -> None:
    scene = bpy.context.scene
    scene.render.engine = "BLENDER_EEVEE"
    scene.render.resolution_x = tile_px
    scene.render.resolution_y = tile_px
    scene.render.film_transparent = True
    scene.render.image_settings.file_format = "PNG"
    scene.render.image_settings.color_mode = "RGBA"


def _append_light_rig(light_blend: Path) -> None:
    append_iso_rig(light_blend)


def _assembly_mesh_objects() -> list[bpy.types.Object]:
    coll = bpy.data.collections.get("ASSEMBLY")
    if coll is not None:
        return [o for o in coll.all_objects if o.type == "MESH"]
    return [o for o in bpy.data.objects if o.type == "MESH" and o.name != "TERRAIN_TILE"]


def _world_bounds(objects: list[bpy.types.Object]) -> tuple[Vector, Vector]:
    mins = Vector((math.inf, math.inf, math.inf))
    maxs = Vector((-math.inf, -math.inf, -math.inf))
    for obj in objects:
        for corner in obj.bound_box:
            wc = obj.matrix_world @ Vector(corner)
            mins.x = min(mins.x, wc.x)
            mins.y = min(mins.y, wc.y)
            mins.z = min(mins.z, wc.z)
            maxs.x = max(maxs.x, wc.x)
            maxs.y = max(maxs.y, wc.y)
            maxs.z = max(maxs.z, wc.z)
    if mins.x == math.inf:
        return Vector((0.0, 0.0, 0.0)), Vector((1.0, 1.0, 1.0))
    return mins, maxs


def _ensure_iso_camera(elevation_deg: float = 35.264, *, ortho_scale: float = 12.0) -> bpy.types.Object:
    cam_data = bpy.data.cameras.new("TileOrthoCam")
    cam_data.type = "ORTHO"
    cam_data.ortho_scale = ortho_scale
    cam_obj = bpy.data.objects.new("TileOrthoCam", cam_data)
    bpy.context.scene.collection.objects.link(cam_obj)
    bpy.context.scene.camera = cam_obj
    elev = math.radians(elevation_deg)
    dist = 20.0
    cam_obj.location = (dist * math.cos(elev), -dist * math.sin(elev), dist * 0.7)
    cam_obj.rotation_euler = (math.radians(60.0), 0.0, math.radians(45.0))
    return cam_obj


def _frame_assembly_camera(elevation_deg: float = 35.264) -> bpy.types.Object:
    """Center ortho iso camera on ASSEMBLY bounds (corner view, not edge-on slab)."""
    meshes = _assembly_mesh_objects()
    mins, maxs = _world_bounds(meshes)
    center = (mins + maxs) * 0.5
    span = maxs - mins
    span_x = max(float(span.x), 1.0)
    span_y = max(float(span.y), 1.0)
    height = max(float(span.z), 1.0)
    # Wide footprints (e.g. bunker 6×3): approach from the short axis so two facades read.
    wide_x = span_x > span_y * 1.15
    footprint = max(span_x, span_y)
    ortho_scale = max(footprint * 1.35, height * 1.8, 6.0)

    cam = bpy.context.scene.camera
    if cam is None or cam.type != "CAMERA":
        cam = _ensure_iso_camera(elevation_deg, ortho_scale=ortho_scale)
    elif cam.data.type == "ORTHO":
        cam.data.ortho_scale = ortho_scale

    elev = math.radians(elevation_deg)
    dist = max(footprint, height) * 2.5
    if wide_x:
        # Camera on −Y side looking toward +Y corner (6×3 bunker readable).
        offset = Vector(
            (
                dist * 0.45 * math.cos(elev),
                -dist * math.sin(elev),
                dist * 0.55 + height * 0.35,
            )
        )
    else:
        offset = Vector(
            (
                dist * math.cos(elev),
                -dist * math.sin(elev),
                dist * 0.55 + height * 0.35,
            )
        )
    cam.location = center + offset
    direction = center - cam.location
    cam.rotation_euler = direction.to_track_quat("-Z", "Y").to_euler()
    return cam


def _variant_color(base: str, variant: dict) -> tuple[float, float, float, float]:
    r, g, b, a = BASE_COLORS.get(base, BASE_COLORS["concrete"])
    damage = float(variant.get("damage") or 0.0)
    lighting = str(variant.get("lighting") or "day")
    power = str(variant.get("power") or "off")
    dim = 1.0 - damage * 0.45
    r, g, b = r * dim, g * dim, b * dim
    if lighting == "night_on" and power in ("on", "partial"):
        r = min(1.0, r + 0.15)
        g = min(1.0, g + 0.12)
    elif lighting == "night_off":
        r, g, b = r * 0.55, g * 0.55, b * 0.6
    if str(variant.get("state")) == "ruined":
        r, g, b = r * 0.7, g * 0.65, b * 0.6
    return (r, g, b, a)


def _build_terrain_plane(base: str, variant: dict) -> None:
    bpy.ops.mesh.primitive_plane_add(size=8.0, location=(0.0, 0.0, 0.0))
    obj = bpy.context.active_object
    obj.name = "TERRAIN_TILE"
    mat = bpy.data.materials.new("TerrainMat")
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    if bsdf:
        col = _variant_color(base, variant)
        bsdf.inputs["Base Color"].default_value = col
        bsdf.inputs["Roughness"].default_value = 0.85
    obj.data.materials.append(mat)


def _apply_variant_to_assembly(variant: dict) -> None:
    damage = float(variant.get("damage") or 0.0)
    for mat in bpy.data.materials:
        if not mat.use_nodes:
            continue
        bsdf = mat.node_tree.nodes.get("Principled BSDF")
        if not bsdf:
            continue
        base = bsdf.inputs["Base Color"].default_value
        dim = 1.0 - damage * 0.35
        bsdf.inputs["Base Color"].default_value = (
            base[0] * dim,
            base[1] * dim,
            base[2] * dim,
            base[3],
        )


def bake(job: dict, *, repo_root: Path) -> Path:
    mode = str(job.get("mode") or "terrain")
    variant = dict(job.get("variant") or {})
    render = dict(job.get("render") or {})
    tile_px = int(render.get("tile_size_px") or 128)
    elev = float(render.get("camera_elevation_deg") or 35.264)

    out = job.get("output") or {}
    png_rel = out.get("png")
    if not png_rel:
        raise ValueError("tile_variant_bake output.png required")
    png_path = Path(png_rel)
    if not png_path.is_absolute():
        png_path = (repo_root / png_rel).resolve()
    png_path.parent.mkdir(parents=True, exist_ok=True)

    light_rel = job.get("light_blend") or ""
    light_path = Path(light_rel) if light_rel else Path()
    if light_rel and not light_path.is_absolute():
        light_path = (repo_root / light_rel).resolve()

    if mode == "assembly":
        blend_rel = job.get("assembly_blend") or ""
        blend_path = Path(blend_rel)
        if not blend_path.is_absolute():
            blend_path = (repo_root / blend_rel).resolve()
        if not blend_path.is_file():
            raise FileNotFoundError(f"Missing assembly blend: {blend_path}")
        bpy.ops.wm.open_mainfile(filepath=str(blend_path))
        _apply_variant_to_assembly(variant)
        _frame_assembly_camera(elev)
    else:
        _reset_scene()
        base = str(job.get("terrain_base") or "concrete")
        _build_terrain_plane(base, variant)

    if light_path.is_file():
        _append_light_rig(light_path)
    if mode == "assembly":
        _frame_assembly_camera(elev)
    elif bpy.context.scene.camera is None:
        _ensure_iso_camera(elev)

    _setup_render(tile_px)
    bpy.context.scene.render.filepath = str(png_path.with_suffix(""))
    bpy.ops.render.render(write_still=True)
    print(f"TILE_BAKE_OK {png_path}")
    return png_path
