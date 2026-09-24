"""Window module — full-bay wall with punched opening + glass (readable at distance).

Axis: height on Blender +Z → glTF +Y after export_yup.
When height_m is a strip (<2.0), expands to FLOOR_HEIGHT (3m) wall bay with
sill/head openings so W→window cell swaps still close the envelope.
"""

from __future__ import annotations

import bpy

from ops.module_coords import add_box, join_objects

FLOOR_HEIGHT_M = 3.0


def _cut_opening(
    wall: bpy.types.Object,
    *,
    opening_w: float,
    opening_h: float,
    sill_m: float,
    depth_m: float,
) -> bpy.types.Object:
    cutter_d = max(depth_m * 2.2, 0.4)
    cutter = add_box(
        "window_opening_cutter",
        opening_w,
        opening_h,
        cutter_d,
        (0.0, 0.0, sill_m + opening_h * 0.5),
    )
    mod = wall.modifiers.new(name="WinCut", type="BOOLEAN")
    mod.operation = "DIFFERENCE"
    mod.solver = "EXACT"
    mod.object = cutter
    bpy.context.view_layer.objects.active = wall
    bpy.ops.object.modifier_apply(modifier=mod.name)
    bpy.data.objects.remove(cutter, do_unlink=True)
    return wall


def build(params: dict) -> bpy.types.Object:
    w = float(params.get("width_m", 4.0))
    strip_h = float(params.get("height_m", 1.5))
    d = float(params.get("depth_m", 0.15))
    frame = float(params.get("frame_m", min(0.1, w * 0.05, strip_h * 0.08)))
    profile = str(params.get("profile", "flat")).lower()
    mullion_w = float(params.get("mullion_width_m", max(0.04, w * 0.02)))

    # Full-bay envelope so window cells don't leave a floating strip.
    bay_h = float(params.get("bay_height_m", FLOOR_HEIGHT_M if strip_h < 2.0 else strip_h))
    # Soft openings at iso: ≥65% bay width, tall glass band (readable void at city-sim distance).
    sill = float(params.get("opening_sill_m", 0.55 if strip_h < 2.0 else 0.0))
    oh = float(
        params.get(
            "opening_height_m",
            max(1.8, bay_h * 0.62) if strip_h < 2.0 else max(strip_h - 0.35, 1.4),
        )
    )
    ow = float(params.get("opening_width_m", max(w - 2 * frame * 2, w * 0.68)))

    wall = add_box("window_bay_wall", w, bay_h, d, (0.0, 0.0, bay_h * 0.5))
    wall = _cut_opening(wall, opening_w=ow, opening_h=oh, sill_m=sill, depth_m=d)

    parts: list[bpy.types.Object] = [wall]
    glass_d = max(d * 0.35, 0.02)
    glass_z = sill + oh * 0.5
    inner_w = max(ow - 2 * frame, ow * 0.7)
    inner_h = max(oh - 2 * frame, oh * 0.7)

    if profile in ("mullion", "curtain", "strip", "arched", "frame_mullion"):
        lite_w = max((inner_w - mullion_w) * 0.5, inner_w * 0.35)
        offset = lite_w * 0.5 + mullion_w * 0.5
        parts.append(add_box("glass_l", lite_w, inner_h, glass_d, (-offset, 0.0, glass_z)))
        parts.append(add_box("glass_r", lite_w, inner_h, glass_d, (offset, 0.0, glass_z)))
        parts.append(add_box("mullion_v", mullion_w, inner_h, d * 0.9, (0.0, 0.0, glass_z)))
    else:
        parts.append(add_box("window_glass", inner_w, inner_h, glass_d, (0.0, 0.0, glass_z)))

    joined = join_objects(parts)
    joined.name = params.get("name", "module_window")
    return joined
