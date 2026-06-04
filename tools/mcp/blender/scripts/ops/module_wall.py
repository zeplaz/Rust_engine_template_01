"""Parametric wall module — panel / recess / brick profiles (not a single scaled cube)."""

from __future__ import annotations

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


def _build_flat_panel(w: float, h: float, d: float) -> bpy.types.Object:
    """Flat panel + base course — readable massing at tactical zoom."""
    sill_h = max(h * 0.06, 0.12)
    body_h = h - sill_h
    sill = _add_box("wall_sill", w, sill_h, d * 1.05, (0.0, sill_h * 0.5, 0.0))
    body = _add_box("wall_body", w * 0.98, body_h, d, (0.0, sill_h + body_h * 0.5, 0.0))
    return _join([sill, body])


def _build_recess(w: float, h: float, d: float) -> bpy.types.Object:
    frame = max(min(0.12, w * 0.04), 0.06)
    recess_d = d * 0.55
    outer = _add_box("wall_outer", w, h, d, (0.0, h * 0.5, 0.0))
    inner_w = max(w - 2 * frame, w * 0.7)
    inner_h = max(h - 2 * frame, h * 0.7)
    recess = _add_box(
        "wall_recess",
        inner_w,
        inner_h,
        recess_d,
        (0.0, h * 0.5, (d - recess_d) * 0.5),
    )
    return _join([outer, recess])


def _build_brick(w: float, h: float, d: float, courses: int) -> bpy.types.Object:
    courses = max(4, int(courses))
    course_h = h / courses
    parts: list[bpy.types.Object] = []
    for i in range(courses):
        inset = 0.02 if i % 2 == 0 else 0.0
        cw = w * (1.0 - inset)
        cy = course_h * 0.5 + i * course_h
        parts.append(_add_box(f"brick_course_{i}", cw, course_h * 0.92, d, (0.0, cy, 0.0)))
    pilaster_w = max(w * 0.08, 0.15)
    parts.append(
        _add_box("brick_pilaster_l", pilaster_w, h, d * 1.02, (-w * 0.5 + pilaster_w * 0.5, h * 0.5, 0.0))
    )
    parts.append(
        _add_box("brick_pilaster_r", pilaster_w, h, d * 1.02, (w * 0.5 - pilaster_w * 0.5, h * 0.5, 0.0))
    )
    return _join(parts)


def build(params: dict) -> bpy.types.Object:
    w = float(params.get("width_m", 4.0))
    h = float(params.get("height_m", 3.0))
    d = float(params.get("depth_m", 0.3))
    profile = str(params.get("profile", params.get("panel", "recess"))).lower()

    if profile in ("brick", "brick_red", "masonry"):
        obj = _build_brick(w, h, d, courses=int(params.get("brick_courses", max(4, int(h // 0.45)))))
    elif profile in ("recess", "panel_recess", "inset"):
        obj = _build_recess(w, h, d)
    elif profile in ("flat", "panel", "panel_flat", ""):
        obj = _build_flat_panel(w, h, d)
    else:
        obj = _build_recess(w, h, d)

    obj.name = params.get("name", "module_wall")
    return obj
