"""Roof module — flat | pitched/gable_cap | shed | sawtooth profiles.

Axis: rise on Blender +Z → glTF +Y after export_yup. Seat plane at Blender Z=0
(underside on wall tops). Pitched builds a **single-span gable cap** (continuous
ridge along depth), not a tiled slope-kit collage.
"""

from __future__ import annotations

import math

import bmesh
import bpy

from ops.module_coords import add_box, join_objects


def _snap_roof_seat_plane_to_z_zero(obj: bpy.types.Object) -> None:
    """Underside seat at Blender Z=0 → glTF Y=0 after export_yup (wall-top plane)."""
    bpy.context.view_layer.update()
    if not obj.data.vertices:
        return
    world_zs = [(obj.matrix_world @ v.co).z for v in obj.data.vertices]
    min_z = min(world_zs)
    if abs(min_z) > 1e-6:
        obj.location.z -= min_z
        bpy.ops.object.transform_apply(location=True)


def _mesh_from_bmesh(bm: bmesh.types.BMesh, name: str) -> bpy.types.Object:
    mesh = bpy.data.meshes.new(name)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    return obj


def _build_flat(w: float, d: float, t: float) -> bpy.types.Object:
    # Thickness along Z; underside at Z=0.
    return add_box("roof_flat", w, t, d, (0.0, 0.0, t * 0.5))


def _build_gable_cap(w: float, d: float, t: float, pitch_h: float) -> bpy.types.Object:
    """Single-span gable prism — ridge along depth, seat eaves at Z=0."""
    hw, hd = w * 0.5, d * 0.5
    # Slight eave thickness so the cap reads as a solid roof, not two floating planks.
    shell = max(t, 0.08)
    bm = bmesh.new()
    # Outer gable prism (closed volume under the slopes down to a thin floor at z=0 would
    # hide wall tops; instead build a hollow-ish shell: two slopes + ridge + gable ends.
    # Bottom of slopes sit at z=0; ridge at z=pitch_h.
    # Outer verts (front y=-hd, back y=+hd)
    # Front eaves L/R, ridge; back eaves L/R, ridge
    # Offset outward by shell along normals approximated by duplicating inner surface.
    ph = max(pitch_h, shell * 2.0)
    # Outer surface
    fl = bm.verts.new((-hw, -hd, 0.0))
    fr = bm.verts.new((hw, -hd, 0.0))
    fre = bm.verts.new((0.0, -hd, ph))
    bl = bm.verts.new((-hw, hd, 0.0))
    br = bm.verts.new((hw, hd, 0.0))
    bre = bm.verts.new((0.0, hd, ph))
    # Inner surface (inset toward ridge / up by shell along Z on eaves)
    inset_x = min(shell * 0.5, hw * 0.15)
    inset_z = shell
    fli = bm.verts.new((-hw + inset_x, -hd, inset_z))
    fri = bm.verts.new((hw - inset_x, -hd, inset_z))
    frei = bm.verts.new((0.0, -hd, ph - inset_z))
    bli = bm.verts.new((-hw + inset_x, hd, inset_z))
    bri = bm.verts.new((hw - inset_x, hd, inset_z))
    brei = bm.verts.new((0.0, hd, ph - inset_z))

    def face(*vs: bmesh.types.BMVert) -> None:
        bm.faces.new(vs)

    # Left slope outer
    face(fl, bl, bre, fre)
    # Right slope outer
    face(fr, fre, bre, br)
    # Front gable
    face(fl, fre, fr)
    # Back gable
    face(bl, br, bre)
    # Left slope inner (reversed)
    face(fli, frei, brei, bli)
    # Right slope inner
    face(fri, bri, brei, frei)
    # Front inner gable
    face(fli, fri, frei)
    # Back inner gable
    face(bli, brei, bri)
    # Eave strips + ridge strip
    face(fl, fr, fri, fli)  # front eave underside
    face(bl, bli, bri, br)  # back eave underside
    face(fl, fli, bli, bl)  # left eave end
    face(fr, br, bri, fri)  # right eave end
    face(fre, bre, brei, frei)  # ridge underside

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    return _mesh_from_bmesh(bm, "roof_gable_cap")


def _build_pitched(w: float, d: float, t: float, pitch_h: float) -> bpy.types.Object:
    """Prefer solid single-span cap; fall back to joined slopes if bmesh fails."""
    try:
        return _build_gable_cap(w, d, t, pitch_h)
    except (ValueError, RuntimeError):
        half_w = w * 0.5
        slope_len = (half_w**2 + pitch_h**2) ** 0.5
        # Box: X=along-slope, Y=depth, Z=thickness; rotate around Y (depth).
        left = add_box("roof_slope_l", slope_len, t, d, (-half_w * 0.5, 0.0, pitch_h * 0.5))
        left.rotation_euler[1] = math.atan2(pitch_h, half_w)
        bpy.ops.object.transform_apply(rotation=True)
        right = add_box("roof_slope_r", slope_len, t, d, (half_w * 0.5, 0.0, pitch_h * 0.5))
        right.rotation_euler[1] = -math.atan2(pitch_h, half_w)
        bpy.ops.object.transform_apply(rotation=True)
        return join_objects([left, right])


def _build_shed(w: float, d: float, t: float, rise: float) -> bpy.types.Object:
    slope_len = (w**2 + rise**2) ** 0.5
    slab = add_box("roof_shed", slope_len, t, d, (0.0, 0.0, rise * 0.5))
    slab.rotation_euler[1] = -math.atan2(rise, w)
    bpy.ops.object.transform_apply(rotation=True)
    fascia = add_box(
        "roof_shed_fascia",
        w,
        max(rise * 0.2, t),
        t,
        (0.0, 0.0, max(rise * 0.1, t * 0.5)),
    )
    return join_objects([slab, fascia])


def _build_sawtooth(w: float, d: float, t: float, bays: int, rise: float) -> bpy.types.Object:
    bays = max(2, int(bays))
    bay_w = w / bays
    parts: list[bpy.types.Object] = []
    for i in range(bays):
        cx = -w * 0.5 + bay_w * (i + 0.5)
        slope_len = math.sqrt((bay_w * 0.5) ** 2 + rise**2)
        left = add_box(f"saw_l_{i}", slope_len, t, d, (cx - bay_w * 0.25, 0.0, rise * 0.5))
        left.rotation_euler[1] = math.atan2(rise, bay_w * 0.5)
        bpy.ops.object.transform_apply(rotation=True)
        right = add_box(f"saw_r_{i}", slope_len, t, d, (cx + bay_w * 0.25, 0.0, rise * 0.5))
        right.rotation_euler[1] = -math.atan2(rise, bay_w * 0.5)
        bpy.ops.object.transform_apply(rotation=True)
        parts.extend([left, right])
    return join_objects(parts)


def build(params: dict) -> bpy.types.Object:
    w = float(params.get("width_m", 4.0))
    d = float(params.get("depth_m", 4.0))
    profile = str(params.get("profile", "flat")).lower()
    # Do not treat height_m as slab thickness for pitched roofs (legacy job bug).
    t = float(params.get("thickness_m", 0.2))
    if "pitch_height_m" in params:
        pitch_h = float(params["pitch_height_m"])
    elif profile in ("pitched", "pitched_gable", "gable", "gable_cap", "cap") and "height_m" in params:
        pitch_h = float(params["height_m"])
    else:
        pitch_h = float(params.get("pitch_height_m", max(t * 3.0, 0.8)))
    bays = int(params.get("sawtooth_bays", max(2, int(w // 2))))

    if profile in ("pitched", "pitched_gable", "gable", "gable_cap", "cap"):
        obj = _build_pitched(w, d, t, pitch_h)
    elif profile == "shed":
        obj = _build_shed(w, d, t, pitch_h)
    elif profile == "sawtooth":
        obj = _build_sawtooth(w, d, t, bays, pitch_h)
    else:
        if "thickness_m" not in params and "height_m" in params:
            t = float(params["height_m"])
        obj = _build_flat(w, d, t)

    obj.name = params.get("name", "module_roof")
    _snap_roof_seat_plane_to_z_zero(obj)
    return obj
