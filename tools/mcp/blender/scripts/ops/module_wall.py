"""Parametric wall module — panel / recess / brick profiles (not a single scaled cube).

Axis: height on Blender +Z → glTF +Y after export_yup (Bevy upright).
Optional opening cutouts for human-scale door/window reads at distance.
"""

from __future__ import annotations

import bpy

from ops.module_coords import add_box, join_objects


def _build_flat_panel(w: float, h: float, d: float) -> bpy.types.Object:
    """Flat panel at exact param dims — bottom flush Z=0, no sill depth lip."""
    return add_box("wall_panel", w, h, d, (0.0, 0.0, h * 0.5))


def _build_recess(w: float, h: float, d: float) -> bpy.types.Object:
    frame = max(min(0.12, w * 0.04), 0.06)
    recess_d = d * 0.55
    outer = add_box("wall_outer", w, h, d, (0.0, 0.0, h * 0.5))
    inner_w = max(w - 2 * frame, w * 0.7)
    inner_h = max(h - 2 * frame, h * 0.7)
    recess = add_box(
        "wall_recess",
        inner_w,
        inner_h,
        recess_d,
        (0.0, (d - recess_d) * 0.5, h * 0.5),
    )
    return join_objects([outer, recess])


def _build_brick(w: float, h: float, d: float, courses: int) -> bpy.types.Object:
    courses = max(4, int(courses))
    course_h = h / courses
    parts: list[bpy.types.Object] = []
    for i in range(courses):
        inset = 0.02 if i % 2 == 0 else 0.0
        cw = w * (1.0 - inset)
        cz = course_h * 0.5 + i * course_h
        parts.append(add_box(f"brick_course_{i}", cw, course_h * 0.92, d, (0.0, 0.0, cz)))
    pilaster_w = max(w * 0.08, 0.15)
    parts.append(
        add_box(
            "brick_pilaster_l",
            pilaster_w,
            h,
            d * 1.02,
            (-w * 0.5 + pilaster_w * 0.5, 0.0, h * 0.5),
        )
    )
    parts.append(
        add_box(
            "brick_pilaster_r",
            pilaster_w,
            h,
            d * 1.02,
            (w * 0.5 - pilaster_w * 0.5, 0.0, h * 0.5),
        )
    )
    return join_objects(parts)


def _cut_opening(
    wall: bpy.types.Object,
    *,
    opening_w: float,
    opening_h: float,
    sill_m: float,
    depth_m: float,
) -> bpy.types.Object:
    """Boolean-subtract a rectangular opening (door/window cutout readable at distance)."""
    cutter_d = max(depth_m * 2.2, 0.4)
    cutter = add_box(
        "wall_opening_cutter",
        opening_w,
        opening_h,
        cutter_d,
        (0.0, 0.0, sill_m + opening_h * 0.5),
    )
    mod = wall.modifiers.new(name="OpeningCut", type="BOOLEAN")
    mod.operation = "DIFFERENCE"
    mod.solver = "EXACT"
    mod.object = cutter
    bpy.context.view_layer.objects.active = wall
    bpy.ops.object.modifier_apply(modifier=mod.name)
    bpy.data.objects.remove(cutter, do_unlink=True)
    return wall


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

    # Optional human-scale cutouts (door / window) — readable void at city-sim distance.
    opening = str(params.get("opening", "") or "").lower()
    if opening in ("door", "doorway"):
        ow = float(params.get("opening_width_m", min(1.2, w * 0.35)))
        oh = float(params.get("opening_height_m", min(2.1, h * 0.72)))
        sill = float(params.get("opening_sill_m", 0.0))
        obj = _cut_opening(obj, opening_w=ow, opening_h=oh, sill_m=sill, depth_m=d)
    elif opening in ("window", "win"):
        ow = float(params.get("opening_width_m", min(1.6, w * 0.45)))
        oh = float(params.get("opening_height_m", min(1.2, h * 0.4)))
        sill = float(params.get("opening_sill_m", 0.9))
        obj = _cut_opening(obj, opening_w=ow, opening_h=oh, sill_m=sill, depth_m=d)
    elif params.get("opening_width_m") is not None:
        ow = float(params["opening_width_m"])
        oh = float(params.get("opening_height_m", 1.2))
        sill = float(params.get("opening_sill_m", 0.9))
        obj = _cut_opening(obj, opening_w=ow, opening_h=oh, sill_m=sill, depth_m=d)

    obj.name = params.get("name", "module_wall")
    return obj
