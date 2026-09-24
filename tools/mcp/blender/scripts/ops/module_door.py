"""Door module — bay wall with human-scale opening, or narrow frame leaf.

Axis: height on Blender +Z → glTF +Y after export_yup.
Bay-sized doors (≥2.5 m wide) punch a cutout into a wall panel so the opening
reads at city-sim distance; narrow doors keep a framed leaf.
"""

from __future__ import annotations

import bmesh
import bpy

from ops.module_coords import add_box, join_objects


def _mesh_from_bmesh(bm: bmesh.types.BMesh, name: str) -> bpy.types.Object:
    mesh = bpy.data.meshes.new(name)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    return obj


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
        "door_opening_cutter",
        opening_w,
        opening_h,
        cutter_d,
        (0.0, 0.0, sill_m + opening_h * 0.5),
    )
    mod = wall.modifiers.new(name="DoorCut", type="BOOLEAN")
    mod.operation = "DIFFERENCE"
    mod.solver = "EXACT"
    mod.object = cutter
    bpy.context.view_layer.objects.active = wall
    bpy.ops.object.modifier_apply(modifier=mod.name)
    bpy.data.objects.remove(cutter, do_unlink=True)
    return wall


def _build_bay_with_opening(params: dict) -> bpy.types.Object:
    """Full-bay wall panel + punched door opening + recessed leaf."""
    w = float(params.get("width_m", 4.0))
    h = float(params.get("height_m", 3.0))
    d = float(params.get("depth_m", 0.2))
    name = params.get("name", "module_door")
    ow = float(params.get("opening_width_m", min(1.2, w * 0.35)))
    oh = float(params.get("opening_height_m", min(2.2, h * 0.75)))
    sill = float(params.get("opening_sill_m", 0.0))

    wall = add_box("door_bay_wall", w, h, d, (0.0, 0.0, h * 0.5))
    wall = _cut_opening(wall, opening_w=ow, opening_h=oh, sill_m=sill, depth_m=d)
    leaf = add_box(
        "door_leaf",
        max(ow * 0.85, 0.4),
        max(oh * 0.9, 0.8),
        max(d * 0.4, 0.05),
        (0.0, -d * 0.1, sill + oh * 0.45),
    )
    joined = join_objects([wall, leaf])
    joined.name = name
    return joined


def _build_frame(params: dict) -> bpy.types.Object:
    w = float(params.get("width_m", 1.0))
    h = float(params.get("height_m", 2.1))
    d = float(params.get("depth_m", 0.15))
    frame = float(params.get("frame_m", min(0.08, w * 0.12)))
    name = params.get("name", "module_door")

    hw, hd = w * 0.5, d * 0.5
    bm = bmesh.new()
    outer = [
        bm.verts.new((-hw, -hd, 0.0)),
        bm.verts.new((hw, -hd, 0.0)),
        bm.verts.new((hw, -hd, h)),
        bm.verts.new((-hw, -hd, h)),
        bm.verts.new((-hw, hd, 0.0)),
        bm.verts.new((hw, hd, 0.0)),
        bm.verts.new((hw, hd, h)),
        bm.verts.new((-hw, hd, h)),
    ]
    iw = hw - frame
    iz0 = frame
    iz1 = h - frame
    inner = [
        bm.verts.new((-iw, -hd * 0.6, iz0)),
        bm.verts.new((iw, -hd * 0.6, iz0)),
        bm.verts.new((iw, -hd * 0.6, iz1)),
        bm.verts.new((-iw, -hd * 0.6, iz1)),
        bm.verts.new((-iw, hd * 0.6, iz0)),
        bm.verts.new((iw, hd * 0.6, iz0)),
        bm.verts.new((iw, hd * 0.6, iz1)),
        bm.verts.new((-iw, hd * 0.6, iz1)),
    ]
    bm.faces.new([outer[0], outer[3], inner[3], inner[0]])
    bm.faces.new([outer[4], inner[4], inner[7], outer[7]])
    bm.faces.new([outer[0], inner[0], inner[4], outer[4]])
    bm.faces.new([outer[1], inner[1], inner[2], outer[2]])
    bm.faces.new([outer[5], outer[6], inner[6], inner[5]])
    bm.faces.new([outer[1], outer[5], inner[5], inner[1]])
    bm.faces.new([outer[2], outer[3], inner[3], inner[2]])
    bm.faces.new([outer[6], inner[6], inner[7], outer[7]])
    bm.faces.new([outer[2], inner[2], inner[6], outer[6]])
    bm.faces.new([outer[0], outer[1], inner[1], inner[0]])
    bm.faces.new([outer[4], inner[4], inner[5], outer[5]])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    frame_obj = _mesh_from_bmesh(bm, name)

    leaf_w = max(w - 2 * frame * 1.2, w * 0.55)
    leaf_h = max(h - 2 * frame * 1.2, h * 0.55)
    leaf = add_box(
        "door_leaf",
        leaf_w,
        leaf_h,
        max(d * 0.35, 0.04),
        (0.0, -d * 0.15, frame + leaf_h * 0.5),
    )
    joined = join_objects([frame_obj, leaf])
    joined.name = name
    return joined


def build(params: dict) -> bpy.types.Object:
    w = float(params.get("width_m", 1.2))
    profile = str(params.get("profile", "frame")).lower()
    # Bay-sized doors: wall panel with punched opening (readable at distance).
    if w >= 2.5 or profile in {"bay", "wall_opening", "warehouse", "garage", "industrial"}:
        # Warehouse/garage often want taller/wider openings inside a bay panel.
        p = dict(params)
        if "height_m" not in p:
            p["height_m"] = 3.0
        if profile in {"warehouse", "garage", "industrial"} and "opening_width_m" not in p:
            p["opening_width_m"] = min(w * 0.78, 3.4)
            p["opening_height_m"] = float(p.get("height_m", 3.0)) * 0.88
        elif "opening_width_m" not in p:
            # Residential bay: soft door void — not a thin leaf lost at iso distance.
            p["opening_width_m"] = min(w * 0.55, 2.4)
            p["opening_height_m"] = min(2.55, float(p.get("height_m", 3.0)) * 0.85)
        return _build_bay_with_opening(p)
    if profile in {"frame", "residential", "shop", "lod0"}:
        return _build_frame(params)
    h = float(params.get("height_m", 2.4))
    d = float(params.get("depth_m", 0.15))
    return add_box(params.get("name", "module_door"), w, h, d, (0.0, 0.0, h * 0.5))
