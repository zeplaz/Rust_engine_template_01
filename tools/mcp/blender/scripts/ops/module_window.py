"""Window module — frame + glass + optional mullions."""

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


def build(params: dict) -> bpy.types.Object:
    w = float(params.get("width_m", 4.0))
    h = float(params.get("height_m", 1.5))
    d = float(params.get("depth_m", 0.12))
    frame = float(params.get("frame_m", min(0.08, w * 0.05, h * 0.05)))
    profile = str(params.get("profile", "flat")).lower()
    mullion_w = float(params.get("mullion_width_m", max(0.04, w * 0.02)))
    sill_y = h * 0.5

    parts: list[bpy.types.Object] = []
    parts.append(_add_box("window_frame", w, h, d, (0.0, sill_y, 0.0)))

    inner_w = max(w - 2 * frame, w * 0.5)
    inner_h = max(h - 2 * frame, h * 0.5)
    glass_d = max(d * 0.35, 0.02)
    glass_y = sill_y

    if profile in ("mullion", "curtain", "strip", "arched"):
        # Two lite bays + center mullion
        lite_w = max((inner_w - mullion_w) * 0.5, inner_w * 0.35)
        offset = lite_w * 0.5 + mullion_w * 0.5
        parts.append(_add_box("glass_l", lite_w, inner_h, glass_d, (-offset, glass_y, 0.0)))
        parts.append(_add_box("glass_r", lite_w, inner_h, glass_d, (offset, glass_y, 0.0)))
        parts.append(_add_box("mullion_v", mullion_w, inner_h, d * 0.9, (0.0, glass_y, 0.0)))
        if profile == "arched":
            head_h = max(frame, h * 0.08)
            parts.append(_add_box("arch_head", inner_w, head_h, d * 0.85, (0.0, sill_y + inner_h * 0.5 - head_h * 0.5, 0.0)))
    else:
        parts.append(_add_box("window_glass", inner_w, inner_h, glass_d, (0.0, glass_y, 0.0)))

    joined = _join(parts)
    joined.name = params.get("name", "module_window")
    return joined
