"""CITY-C8 — bake-time merge of module-kit GLBs into one variant GLB (deterministic)."""

from __future__ import annotations

from pathlib import Path

import bpy


def _repo_from_params(params: dict) -> Path:
    raw = params.get("_repo_root")
    if raw:
        return Path(str(raw))
    return Path.cwd()


def _reset_scene() -> None:
    bpy.ops.wm.read_factory_settings(use_empty=True)


def _import_glb(glb_path: Path) -> list[bpy.types.Object]:
    bpy.ops.import_scene.gltf(filepath=str(glb_path))
    return list(bpy.context.selected_objects)


def build(params: dict) -> None:
    repo = _repo_from_params(params)
    parts = params.get("parts") or []
    if not parts:
        raise ValueError("module_variant_merge requires params.parts[]")

    out_raw = params.get("_output_glb") or params.get("output_glb")
    if not out_raw:
        raise ValueError("module_variant_merge requires _output_glb")
    out_path = Path(str(out_raw))
    if not out_path.is_absolute():
        out_path = (repo / out_path).resolve()
    out_path.parent.mkdir(parents=True, exist_ok=True)

    _reset_scene()
    root_empty = bpy.data.objects.new("VARIANT_ROOT", None)
    bpy.context.scene.collection.objects.link(root_empty)

    mesh_objects: list[bpy.types.Object] = []
    for idx, part in enumerate(parts):
        glb_rel = part.get("glb_path") or part.get("glb")
        if not glb_rel:
            raise ValueError(f"parts[{idx}] missing glb_path")
        glb_path = Path(str(glb_rel))
        if not glb_path.is_absolute():
            glb_path = (repo / glb_path).resolve()
        if not glb_path.is_file():
            raise FileNotFoundError(glb_path)

        imported = _import_glb(glb_path)
        anchor = bpy.data.objects.new(f"part_{idx}", None)
        bpy.context.scene.collection.objects.link(anchor)
        anchor.parent = root_empty
        pos = part.get("position") or [0.0, 0.0, 0.0]
        rot = part.get("rotation_euler") or [0.0, 0.0, 0.0]
        anchor.location = (float(pos[0]), float(pos[1]), float(pos[2]))
        anchor.rotation_euler = (float(rot[0]), float(rot[1]), float(rot[2]))
        for obj in imported:
            obj.parent = anchor
            if obj.type == "MESH":
                mesh_objects.append(obj)

    if not mesh_objects:
        raise RuntimeError("module_variant_merge: no mesh objects imported")

    bpy.ops.object.select_all(action="DESELECT")
    for obj in mesh_objects:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = mesh_objects[0]
    bpy.ops.object.join()

    merged = bpy.context.view_layer.objects.active
    merged.name = str(params.get("name") or "building_variant_merged")

    from ops.export_glb import export_glb

    export_glb(str(out_path), material_profile=params.get("material_profile"), repo_root=repo)
    print(f"VARIANT_MERGE_OK {out_path} parts={len(parts)} verts={len(merged.data.vertices)}")
