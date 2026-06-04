"""Door module — frame profile with recessed leaf opening."""

from __future__ import annotations

import bmesh
import bpy


def _mesh_from_bmesh(bm: bmesh.types.BMesh, name: str) -> bpy.types.Object:
    mesh = bpy.data.meshes.new(name)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    return obj


def _build_frame(params: dict) -> bpy.types.Object:
    w = float(params.get("width_m", 1.0))
    h = float(params.get("height_m", 2.1))
    d = float(params.get("depth_m", 0.15))
    frame = float(params.get("frame_m", min(0.08, w * 0.12)))
    name = params.get("name", "module_door")

    hw, hh, hd = w * 0.5, h * 0.5, d * 0.5
    bm = bmesh.new()
    # Outer box faces only (hollow frame silhouette)
    outer = [
        bm.verts.new((-hw, 0.0, -hd)),
        bm.verts.new((hw, 0.0, -hd)),
        bm.verts.new((hw, h, -hd)),
        bm.verts.new((-hw, h, -hd)),
        bm.verts.new((-hw, 0.0, hd)),
        bm.verts.new((hw, 0.0, hd)),
        bm.verts.new((hw, h, hd)),
        bm.verts.new((-hw, h, hd)),
    ]
    iw, ih = hw - frame, h - frame
    inner = [
        bm.verts.new((-iw, frame, -hd * 0.6)),
        bm.verts.new((iw, frame, -hd * 0.6)),
        bm.verts.new((iw, ih, -hd * 0.6)),
        bm.verts.new((-iw, ih, -hd * 0.6)),
        bm.verts.new((-iw, frame, hd * 0.6)),
        bm.verts.new((iw, frame, hd * 0.6)),
        bm.verts.new((iw, ih, hd * 0.6)),
        bm.verts.new((-iw, ih, hd * 0.6)),
    ]
    # Left jamb
    bm.faces.new([outer[0], outer[3], inner[3], inner[0]])
    bm.faces.new([outer[4], inner[4], inner[7], outer[7]])
    bm.faces.new([outer[0], inner[0], inner[4], outer[4]])
    # Right jamb
    bm.faces.new([outer[1], inner[1], inner[2], outer[2]])
    bm.faces.new([outer[5], outer[6], inner[6], inner[5]])
    bm.faces.new([outer[1], outer[5], inner[5], inner[1]])
    # Header
    bm.faces.new([outer[2], outer[3], inner[3], inner[2]])
    bm.faces.new([outer[6], inner[6], inner[7], outer[7]])
    bm.faces.new([outer[2], inner[2], inner[6], outer[6]])
    # Threshold
    bm.faces.new([outer[0], outer[1], inner[1], inner[0]])
    bm.faces.new([outer[4], inner[4], inner[5], outer[5]])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    return _mesh_from_bmesh(bm, name)


def build(params: dict) -> bpy.types.Object:
    profile = str(params.get("profile", "frame")).lower()
    if profile in {"frame", "residential", "shop", "lod0"}:
        return _build_frame(params)
    w = float(params.get("width_m", 1.2))
    h = float(params.get("height_m", 2.4))
    d = float(params.get("depth_m", 0.15))
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=(0.0, h * 0.5, 0.0))
    obj = bpy.context.active_object
    obj.name = params.get("name", "module_door")
    obj.scale = (w, h, d)
    bpy.ops.object.transform_apply(scale=True)
    return obj
