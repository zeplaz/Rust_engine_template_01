"""Roof module — flat | pitched | shed | sawtooth profiles."""

from __future__ import annotations

import math

import bpy


def _add_box(name: str, w: float, h: float, d: float, loc: tuple[float, float, float]) -> bpy.types.Object:
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=loc)
    obj = bpy.context.active_object
    obj.name = name
    obj.scale = (w, h, d)
    bpy.ops.object.transform_apply(scale=True)
    return obj


def _join(objects: list[bpy.types.Object]) -> bpy.types.Object:
    bpy.ops.object.select_all(action="DESELECT")
    for obj in objects:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = objects[0]
    bpy.ops.object.join()
    return bpy.context.active_object


def _snap_roof_seat_plane_to_y_zero(obj: bpy.types.Object) -> None:
    """Underside seat at Blender Y=0 (wall-top plane); post-yup export glTF min Z >= 0."""
    bpy.context.view_layer.update()
    if not obj.data.vertices:
        return
    world_ys = [(obj.matrix_world @ v.co).y for v in obj.data.vertices]
    max_y = max(world_ys)
    if abs(max_y) > 1e-6:
        obj.location.y -= max_y
        bpy.ops.object.transform_apply(location=True)


def _build_flat(w: float, d: float, t: float) -> bpy.types.Object:
    return _add_box("roof_flat", w, t, d, (0.0, -t * 0.5, 0.0))


def _build_pitched(w: float, d: float, t: float, pitch_h: float) -> bpy.types.Object:
    half_w = w * 0.5
    slope_len = (half_w**2 + pitch_h**2) ** 0.5
    left = _add_box("roof_slope_l", slope_len, t, d, (-half_w * 0.5, pitch_h * 0.5, 0.0))
    left.rotation_euler[2] = 0.0
    left.rotation_euler[0] = -math.atan2(pitch_h, half_w)
    bpy.ops.object.transform_apply(rotation=True)
    right = _add_box("roof_slope_r", slope_len, t, d, (half_w * 0.5, pitch_h * 0.5, 0.0))
    right.rotation_euler[0] = math.atan2(pitch_h, half_w)
    bpy.ops.object.transform_apply(rotation=True)
    return _join([left, right])


def _build_shed(w: float, d: float, t: float, rise: float) -> bpy.types.Object:
    slope_len = (w**2 + rise**2) ** 0.5
    slab = _add_box("roof_shed", slope_len, t, d, (0.0, rise * 0.5, 0.0))
    slab.rotation_euler[0] = -math.atan2(rise, w)
    bpy.ops.object.transform_apply(rotation=True)
    fascia = _add_box("roof_shed_fascia", w, max(rise * 0.2, t), t, (-w * 0.5 + w * 0.5, max(rise * 0.1, t * 0.5), 0.0))
    return _join([slab, fascia])


def _build_sawtooth(w: float, d: float, t: float, bays: int, rise: float) -> bpy.types.Object:
    bays = max(2, int(bays))
    bay_w = w / bays
    parts: list[bpy.types.Object] = []
    for i in range(bays):
        cx = -w * 0.5 + bay_w * (i + 0.5)
        slope_len = (bay_w * 0.5) ** 2 + rise**2
        slope_len = math.sqrt(slope_len)
        left = _add_box(f"saw_l_{i}", slope_len, t, d, (cx - bay_w * 0.25, rise * 0.5, 0.0))
        left.rotation_euler[0] = -math.atan2(rise, bay_w * 0.5)
        bpy.ops.object.transform_apply(rotation=True)
        right = _add_box(f"saw_r_{i}", slope_len, t, d, (cx + bay_w * 0.25, rise * 0.5, 0.0))
        right.rotation_euler[0] = math.atan2(rise, bay_w * 0.5)
        bpy.ops.object.transform_apply(rotation=True)
        parts.extend([left, right])
    return _join(parts)


def build(params: dict) -> bpy.types.Object:
    w = float(params.get("width_m", 4.0))
    d = float(params.get("depth_m", 4.0))
    t = float(params.get("thickness_m", params.get("height_m", 0.2)))
    profile = str(params.get("profile", "flat")).lower()
    pitch_h = float(params.get("pitch_height_m", max(t * 3.0, 0.8)))
    bays = int(params.get("sawtooth_bays", max(2, int(w // 2))))

    if profile in ("pitched", "pitched_gable", "gable"):
        obj = _build_pitched(w, d, t, pitch_h)
    elif profile == "shed":
        obj = _build_shed(w, d, t, pitch_h)
    elif profile == "sawtooth":
        obj = _build_sawtooth(w, d, t, bays, pitch_h)
    else:
        obj = _build_flat(w, d, t)

    obj.name = params.get("name", "module_roof")
    _snap_roof_seat_plane_to_y_zero(obj)
    return obj
