"""Prop / corner module — greybox boxes; prop_kind selects L-corner vs vent/ac."""

from __future__ import annotations

import bpy


def _add_box(name: str, w: float, h: float, d: float, loc: tuple[float, float, float]) -> bpy.types.Object:
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=loc)
    obj = bpy.context.active_object
    obj.name = name
    obj.scale = (w, h, d)
    bpy.ops.object.transform_apply(scale=True)
    return obj


def _build_l_corner(w: float, h: float, d: float, leg: float) -> bpy.types.Object:
    leg_w = w * leg
    leg_d = d * leg
    a = _add_box("corner_leg_a", leg_w, h, d, (leg_w * 0.5 - w * 0.5, h * 0.5, 0.0))
    b = _add_box("corner_leg_b", w, h, leg_d, (0.0, h * 0.5, leg_d * 0.5 - d * 0.5))
    bpy.ops.object.select_all(action="DESELECT")
    a.select_set(True)
    b.select_set(True)
    bpy.context.view_layer.objects.active = a
    bpy.ops.object.join()
    return bpy.context.active_object


def build(params: dict) -> bpy.types.Object:
    w = float(params.get("width_m", 2.0))
    h = float(params.get("height_m", 2.0))
    d = float(params.get("depth_m", 2.0))
    kind = str(params.get("prop_kind", "box"))

    if kind in ("l_corner", "corner", "corner_l"):
        obj = _build_l_corner(w, h, d, leg=float(params.get("leg_ratio", 0.45)))
    elif kind in ("chimney", "prop_chimney"):
        base = _add_box("chimney_base", w * 0.85, h * 0.72, d * 0.85, (0.0, h * 0.36, 0.0))
        stack = _add_box("chimney_stack", w * 0.55, h * 0.55, d * 0.55, (0.0, h * 0.78, 0.0))
        cap = _add_box("chimney_cap", w * 0.95, max(h * 0.12, 0.15), d * 0.95, (0.0, h * 0.96, 0.0))
        bpy.ops.object.select_all(action="DESELECT")
        base.select_set(True)
        stack.select_set(True)
        cap.select_set(True)
        bpy.context.view_layer.objects.active = base
        bpy.ops.object.join()
        obj = bpy.context.active_object
    elif kind in ("vent", "ac"):
        base_h = h * 0.65
        cap_h = h - base_h
        base = _add_box("prop_base", w, base_h, d, (0.0, base_h * 0.5, 0.0))
        cap = _add_box(
            "prop_cap",
            w * 0.7,
            max(cap_h, 0.05),
            d * 0.7,
            (0.0, base_h + cap_h * 0.5, 0.0),
        )
        bpy.ops.object.select_all(action="DESELECT")
        base.select_set(True)
        cap.select_set(True)
        bpy.context.view_layer.objects.active = base
        bpy.ops.object.join()
        obj = bpy.context.active_object
    else:
        obj = _add_box("module_prop", w, h, d, (0.0, h * 0.5, 0.0))

    obj.name = params.get("name", "module_prop")
    return obj
