"""Export active meshes to glTF binary with greybox or profile PBR materials."""

from __future__ import annotations

from pathlib import Path

import bmesh
import bpy

# sRGB-ish greybox tints by module category (R, G, B, A)
_GREYBOX_TINTS: dict[str, tuple[float, float, float, float]] = {
    "module_wall": (0.55, 0.52, 0.48, 1.0),
    "module_roof": (0.35, 0.38, 0.42, 1.0),
    "module_door": (0.42, 0.32, 0.22, 1.0),
    "module_window": (0.65, 0.78, 0.88, 0.85),
    "module_prop": (0.48, 0.50, 0.52, 1.0),
}
_DEFAULT_TINT = (0.5, 0.5, 0.5, 1.0)

_PROFILE_METALLIC: dict[str, float] = {
    "steel_door_01": 0.85,
    "steel_door_warehouse_01": 0.88,
    "steel_corner_01": 0.85,
    "steel_panel_01": 0.85,
    "roof_metal_01": 0.9,
    "roof_metal_shed_01": 0.9,
}


def _infer_category(obj_name: str) -> str:
    name = obj_name.lower()
    for key in _GREYBOX_TINTS:
        token = key.removeprefix("module_")
        if token in name or name.startswith(key):
            return key
    if "wall" in name:
        return "module_wall"
    if "roof" in name:
        return "module_roof"
    if "door" in name:
        return "module_door"
    if "window" in name:
        return "module_window"
    return "module_prop"


def _repo_from_glb_path(glb_path: Path) -> Path:
    candidate = glb_path.resolve()
    for _ in range(10):
        if (candidate / "Cargo.toml").is_file():
            return candidate
        if candidate.parent == candidate:
            break
        candidate = candidate.parent
    return glb_path.resolve().parents[4]


def _profile_texture_paths(repo: Path, profile_id: str) -> dict[str, Path] | None:
    base = repo / "assets" / "materials" / "textures" / profile_id
    albedo = base / "albedo.png"
    normal = base / "normal.png"
    rough = base / "roughness.png"
    if not albedo.is_file():
        return None
    out: dict[str, Path] = {"albedo": albedo}
    if normal.is_file():
        out["normal"] = normal
    if rough.is_file():
        out["roughness"] = rough
    return out


def _image_node(nodes, images: bpy.types.Image, *, label: str, non_color: bool = False):
    node = nodes.new("ShaderNodeTexImage")
    node.image = images
    node.label = label
    if non_color and hasattr(images, "colorspace_settings"):
        images.colorspace_settings.name = "Non-Color"
    return node


def _build_pbr_material(profile_id: str, tex: dict[str, Path]) -> bpy.types.Material:
    mat_name = f"pbr_{profile_id}"
    mat = bpy.data.materials.get(mat_name)
    if mat is not None:
        bpy.data.materials.remove(mat)

    mat = bpy.data.materials.new(name=mat_name)
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    links = mat.node_tree.links
    nodes.clear()

    out = nodes.new("ShaderNodeOutputMaterial")
    bsdf = nodes.new("ShaderNodeBsdfPrincipled")
    out.location = (400, 0)
    bsdf.location = (100, 0)
    links.new(bsdf.outputs["BSDF"], out.inputs["Surface"])

    albedo_img = bpy.data.images.load(str(tex["albedo"]), check_existing=True)
    albedo_node = _image_node(nodes, albedo_img, label="Albedo")
    albedo_node.location = (-600, 200)
    links.new(albedo_node.outputs["Color"], bsdf.inputs["Base Color"])

    if "roughness" in tex:
        rough_img = bpy.data.images.load(str(tex["roughness"]), check_existing=True)
        if hasattr(rough_img, "colorspace_settings"):
            rough_img.colorspace_settings.name = "Non-Color"
        rough_node = _image_node(nodes, rough_img, label="Roughness", non_color=True)
        rough_node.location = (-600, -50)
        links.new(rough_node.outputs["Color"], bsdf.inputs["Roughness"])

    if "normal" in tex:
        normal_img = bpy.data.images.load(str(tex["normal"]), check_existing=True)
        if hasattr(normal_img, "colorspace_settings"):
            normal_img.colorspace_settings.name = "Non-Color"
        normal_tex = _image_node(nodes, normal_img, label="Normal", non_color=True)
        normal_tex.location = (-600, -250)
        normal_map = nodes.new("ShaderNodeNormalMap")
        normal_map.location = (-300, -250)
        links.new(normal_tex.outputs["Color"], normal_map.inputs["Color"])
        links.new(normal_map.outputs["Normal"], bsdf.inputs["Normal"])

    metallic = _PROFILE_METALLIC.get(profile_id, 0.0)
    bsdf.inputs["Metallic"].default_value = metallic
    return mat


def _ensure_greybox_material(obj: bpy.types.Object, category: str) -> None:
    tint = _GREYBOX_TINTS.get(category, _DEFAULT_TINT)
    mat_name = f"greybox_{category}"
    mat = bpy.data.materials.get(mat_name)
    if mat is None:
        mat = bpy.data.materials.new(name=mat_name)
        mat.use_nodes = True
        nodes = mat.node_tree.nodes
        bsdf = nodes.get("Principled BSDF")
        if bsdf is not None:
            bsdf.inputs["Base Color"].default_value = tint
            bsdf.inputs["Roughness"].default_value = 0.65
            if category == "module_window":
                bsdf.inputs["Alpha"].default_value = tint[3]
                mat.blend_method = "BLEND"
    if obj.data.materials:
        obj.data.materials[0] = mat
    else:
        obj.data.materials.append(mat)


def apply_material_profile_to_meshes(
    mesh_objects: list,
    *,
    material_profile: str | None = None,
    repo_root: Path | None = None,
) -> str:
    """Apply PBR profile (or greybox) to specific mesh objects only."""
    repo = repo_root
    tex = None
    if material_profile and repo is not None:
        tex = _profile_texture_paths(repo, material_profile)

    mode = "pbr_profile" if tex else "greybox"
    mat = None
    if mode == "pbr_profile" and tex is not None and material_profile is not None:
        mat = _build_pbr_material(material_profile, tex)

    for obj in mesh_objects:
        if getattr(obj, "type", None) != "MESH":
            continue
        if mat is not None:
            if obj.data.materials:
                obj.data.materials[0] = mat
            else:
                obj.data.materials.append(mat)
        else:
            _ensure_greybox_material(obj, _infer_category(obj.name))
    return mode


def apply_materials(*, material_profile: str | None = None, repo_root: Path | None = None) -> str:
    """Apply PBR profile textures when available; else greybox fallback. Returns mode used."""
    repo = repo_root
    tex = None
    if material_profile and repo is not None:
        tex = _profile_texture_paths(repo, material_profile)

    mode = "pbr_profile" if tex else "greybox"
    mat = None
    if mode == "pbr_profile" and tex is not None and material_profile is not None:
        mat = _build_pbr_material(material_profile, tex)

    meshes = [o for o in bpy.context.scene.objects if o.type == "MESH"]
    return apply_material_profile_to_meshes(
        meshes, material_profile=material_profile, repo_root=repo
    )


def apply_greybox_materials() -> None:
    apply_materials()


def _assign_box_uv_to_mesh(mesh: bpy.types.Mesh) -> None:
    """Procedural modules (bmesh) ship without UV — Bevy mikktspace needs TEXCOORD_0."""
    if len(mesh.polygons) == 0:
        return
    bm = bmesh.new()
    bm.from_mesh(mesh)
    uv_layer = bm.loops.layers.uv.verify()
    for face in bm.faces:
        n = face.normal
        ax, ay, az = abs(n.x), abs(n.y), abs(n.z)
        for loop in face.loops:
            v = loop.vert.co
            if ay >= ax and ay >= az:
                u, vcoord = v.x, v.z
            elif ax >= az:
                u, vcoord = v.z, v.y
            else:
                u, vcoord = v.x, v.y
            loop[uv_layer].uv = (u, vcoord)
    bm.to_mesh(mesh)
    bm.free()
    mesh.update()


def ensure_mesh_uv_layers(objects: list[bpy.types.Object]) -> None:
    for obj in objects:
        if getattr(obj, "type", None) != "MESH":
            continue
        mesh = obj.data
        if not mesh.uv_layers:
            mesh.uv_layers.new(name="UVMap")
        _assign_box_uv_to_mesh(mesh)


def export_glb(filepath: str, *, material_profile: str | None = None, repo_root: Path | None = None) -> None:
    glb_path = Path(filepath)
    repo = repo_root or _repo_from_glb_path(glb_path)
    mode = apply_materials(material_profile=material_profile, repo_root=repo)
    meshes = [o for o in bpy.context.scene.objects if o.type == "MESH"]
    ensure_mesh_uv_layers(meshes)
    bpy.ops.object.select_all(action="DESELECT")
    for obj in bpy.context.scene.objects:
        if obj.type == "MESH":
            obj.select_set(True)
    bpy.ops.export_scene.gltf(
        filepath=filepath,
        export_format="GLB",
        use_selection=True,
        export_apply=True,
        export_yup=True,
        export_materials="EXPORT",
        export_image_format="AUTO",
    )
    print(f"EXPORT_MODE {mode} profile={material_profile or 'none'}")
