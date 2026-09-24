"""Prop / corner module — greybox boxes; prop_kind selects L-corner vs vent/ac.

Axis: height on Blender +Z → glTF +Y after export_yup (same as walls).
"""

from __future__ import annotations

import bpy

from ops.module_coords import add_box, join_objects


def _build_l_corner(w: float, h: float, d: float, leg: float) -> bpy.types.Object:
    leg_w = w * leg
    leg_d = d * leg
    a = add_box("corner_leg_a", leg_w, h, d, (leg_w * 0.5 - w * 0.5, 0.0, h * 0.5))
    b = add_box("corner_leg_b", w, h, leg_d, (0.0, leg_d * 0.5 - d * 0.5, h * 0.5))
    return join_objects([a, b])


def build(params: dict) -> bpy.types.Object:
    w = float(params.get("width_m", 2.0))
    h = float(params.get("height_m", 2.0))
    d = float(params.get("depth_m", 2.0))
    kind = str(params.get("prop_kind", "box"))

    if kind in ("l_corner", "corner", "corner_l"):
        obj = _build_l_corner(w, h, d, leg=float(params.get("leg_ratio", 0.45)))
    elif kind in ("chimney", "prop_chimney"):
        base = add_box("chimney_base", w * 0.85, h * 0.72, d * 0.85, (0.0, 0.0, h * 0.36))
        stack = add_box("chimney_stack", w * 0.55, h * 0.55, d * 0.55, (0.0, 0.0, h * 0.78))
        cap = add_box("chimney_cap", w * 0.95, max(h * 0.12, 0.15), d * 0.95, (0.0, 0.0, h * 0.96))
        obj = join_objects([base, stack, cap])
    elif kind in ("vent", "ac"):
        base_h = h * 0.65
        cap_h = h - base_h
        base = add_box("prop_base", w, base_h, d, (0.0, 0.0, base_h * 0.5))
        cap = add_box(
            "prop_cap",
            w * 0.7,
            max(cap_h, 0.05),
            d * 0.7,
            (0.0, 0.0, base_h + cap_h * 0.5),
        )
        obj = join_objects([base, cap])
    elif kind in ("transformer", "prop_transformer"):
        radius = min(d, h) * 0.35
        length = w * 0.55
        bpy.ops.mesh.primitive_cylinder_add(
            radius=radius,
            depth=length,
            location=(0.0, 0.0, radius + 0.05),
        )
        tank = bpy.context.active_object
        tank.name = "transformer_tank"
        tank.rotation_euler[2] = 1.5707963
        bpy.ops.object.transform_apply(rotation=True)
        bushing_r = max(radius * 0.22, 0.08)
        bushing_h = max(h * 0.28, 0.12)
        offsets = (-length * 0.28, 0.0, length * 0.28)
        parts: list[bpy.types.Object] = [tank]
        for idx, ox in enumerate(offsets):
            bpy.ops.mesh.primitive_cylinder_add(
                radius=bushing_r,
                depth=bushing_h,
                location=(ox, 0.0, radius * 2.0 + bushing_h * 0.5),
            )
            bushing = bpy.context.active_object
            bushing.name = f"transformer_bushing_{idx}"
            parts.append(bushing)
        pad = add_box("transformer_pad", w, max(h * 0.08, 0.08), d, (0.0, 0.0, max(h * 0.04, 0.04)))
        parts.append(pad)
        obj = join_objects(parts)
    elif kind in ("fence", "fence_chainlink"):
        panel = add_box("fence_panel", w, h, max(d, 0.08), (0.0, 0.0, h * 0.5))
        post_l = add_box(
            "fence_post_l",
            max(w * 0.06, 0.08),
            h * 1.02,
            max(d * 0.5, 0.08),
            (-w * 0.45, 0.0, h * 0.51),
        )
        post_r = add_box(
            "fence_post_r",
            max(w * 0.06, 0.08),
            h * 1.02,
            max(d * 0.5, 0.08),
            (w * 0.45, 0.0, h * 0.51),
        )
        obj = join_objects([panel, post_l, post_r])
    else:
        # Generic upright box — covers bus_bay, breaker, shack, gravel, signs, yards, etc.
        obj = add_box("module_prop", w, h, d, (0.0, 0.0, h * 0.5))

    obj.name = params.get("name", "module_prop")
    return obj
