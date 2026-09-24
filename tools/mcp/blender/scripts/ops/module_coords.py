"""Shared module axis convention for Bevy Y-up glTF.

Author in Blender with **height on +Z** (Blender up). ``export_glb`` uses
``export_yup=True``, which maps Blender Z → glTF Y so wall height lands on
Bevy +Y. Semantic dims: width→X, depth→Blender Y, height→Blender Z.

Legacy bug: height was authored on Blender Y, so after yup export walls lay
flat (height on glTF −Z, thickness on glTF Y). See BUILDING-LOOK-V2-ART.
"""

from __future__ import annotations

import bpy


def scale_whd(width_m: float, height_m: float, depth_m: float) -> tuple[float, float, float]:
    """Blender object.scale XYZ for semantic width/height/depth."""
    return (float(width_m), float(depth_m), float(height_m))


def add_box(
    name: str,
    width_m: float,
    height_m: float,
    depth_m: float,
    loc: tuple[float, float, float],
) -> bpy.types.Object:
    """Unit cube scaled to (width, height, depth) with height on Blender Z."""
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=loc)
    obj = bpy.context.active_object
    obj.name = name
    obj.scale = scale_whd(width_m, height_m, depth_m)
    bpy.ops.object.transform_apply(scale=True)
    return obj


def join_objects(objects: list[bpy.types.Object]) -> bpy.types.Object:
    bpy.ops.object.select_all(action="DESELECT")
    for obj in objects:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = objects[0]
    bpy.ops.object.join()
    return bpy.context.active_object
