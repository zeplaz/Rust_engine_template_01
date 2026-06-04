"""Iso tile camera + light rig — no meshes, no reference vehicles.

Authoritative on-disk rig: ``utils/Tile_iso_rig_v1.blend`` (collection ``TILE_ISO_RIG``).
Legacy ``Light_keysshotsetup.blend`` is input-only for one-time extraction; never append whole-file.
"""

from __future__ import annotations

import math
from pathlib import Path

import bpy

TILE_ISO_RIG_COLLECTION = "TILE_ISO_RIG"
DEFAULT_ISO_RIG_REL = "utils/Tile_iso_rig_v1.blend"

_JUNK_NAME_PARTS = (
    "truck",
    "civ",
    "vehicle",
    "car",
    "chassis",
    "wheel",
    "body",
    "mesh",
    "cargo",
    "trailer",
    "lorry",
    "bus",
    "van",
)


def _is_junk_name(name: str) -> bool:
    n = name.lower()
    return any(p in n for p in _JUNK_NAME_PARTS)


def _rig_object_types() -> set[str]:
    return {"CAMERA", "LIGHT", "EMPTY"}


def _ensure_rig_collection() -> bpy.types.Collection:
    coll = bpy.data.collections.get(TILE_ISO_RIG_COLLECTION)
    if coll is None:
        coll = bpy.data.collections.new(TILE_ISO_RIG_COLLECTION)
        bpy.context.scene.collection.children.link(coll)
    return coll


def append_iso_rig(light_blend: Path, *, link: bool = False) -> None:
    """Append only camera/light/empty rig objects — never mesh geometry."""
    if not light_blend.is_file():
        return
    rig_coll = _ensure_rig_collection()
    with bpy.data.libraries.load(str(light_blend), link=link) as (data_from, data_to):
        pick: list[str] = []
        for name in data_from.objects:
            if _is_junk_name(name):
                continue
            nl = name.lower()
            if any(
                tok in nl
                for tok in (
                    "cam",
                    "camera",
                    "light",
                    "sun",
                    "key",
                    "fill",
                    "rim",
                    "iso",
                    "spot",
                    "area",
                )
            ):
                pick.append(name)
        if not pick:
            for name in data_from.objects:
                if _is_junk_name(name):
                    continue
                pick.append(name)
        data_to.objects = pick[:24]

    for obj in data_to.objects:
        if obj.type not in _rig_object_types():
            bpy.data.objects.remove(obj, do_unlink=True)
            continue
        if obj.name not in rig_coll.objects:
            rig_coll.objects.link(obj)
        for uc in list(obj.users_collection):
            if uc != rig_coll:
                uc.objects.unlink(obj)
        if obj.type == "CAMERA" and bpy.context.scene.camera is None:
            bpy.context.scene.camera = obj


def _remove_non_rig_objects() -> None:
    for obj in list(bpy.data.objects):
        if obj.type not in _rig_object_types():
            bpy.data.objects.remove(obj, do_unlink=True)
        elif _is_junk_name(obj.name):
            bpy.data.objects.remove(obj, do_unlink=True)
    for block_type in (
        "meshes",
        "curves",
        "surfaces",
        "metaballs",
        "fonts",
        "armatures",
        "materials",
        "textures",
    ):
        datablock = getattr(bpy.data, block_type, None)
        if datablock is None:
            continue
        for block in list(datablock):
            datablock.remove(block, do_unlink=True, do_id_user=True, do_ui_user=True)
    for image in list(bpy.data.images):
        bpy.data.images.remove(image, do_unlink=True)
    for action in list(bpy.data.actions):
        used = False
        for obj in bpy.data.objects:
            ad = obj.animation_data
            if ad and ad.action == action:
                used = True
                break
        if not used:
            bpy.data.actions.remove(action, do_unlink=True)
    try:
        bpy.ops.outliner.orphans_purge(
            do_local_ids=True,
            do_linked_ids=True,
            do_recursive=True,
        )
    except Exception:
        pass


def _organize_rig_collection() -> None:
    rig_coll = _ensure_rig_collection()
    scene_root = bpy.context.scene.collection
    for obj in list(bpy.data.objects):
        if obj.type not in _rig_object_types():
            continue
        if obj.name not in rig_coll.objects:
            rig_coll.objects.link(obj)
        for uc in list(obj.users_collection):
            if uc != rig_coll and uc != scene_root:
                uc.objects.unlink(obj)
        if obj.type == "CAMERA" and bpy.context.scene.camera is None:
            bpy.context.scene.camera = obj


def build_procedural_iso_rig() -> None:
    """Minimal deterministic iso rig when legacy file is unavailable."""
    bpy.ops.wm.read_factory_settings(use_empty=True)
    rig_coll = _ensure_rig_collection()

    bpy.ops.object.light_add(type="SUN", location=(8.0, -6.0, 12.0))
    sun = bpy.context.active_object
    sun.name = "IsoSun"
    sun.data.energy = 2.5
    rig_coll.objects.link(sun)
    bpy.context.scene.collection.objects.unlink(sun)

    bpy.ops.object.light_add(type="AREA", location=(-5.0, 4.0, 8.0))
    fill = bpy.context.active_object
    fill.name = "IsoFill"
    fill.data.energy = 180.0
    fill.data.size = 4.0
    rig_coll.objects.link(fill)
    bpy.context.scene.collection.objects.unlink(fill)

    bpy.ops.object.camera_add(location=(14.0, -14.0, 10.0))
    cam = bpy.context.active_object
    cam.name = "IsoCamera"
    cam.data.type = "PERSP"
    cam.data.lens = 50.0
    elev = math.radians(35.264)
    cam.rotation_euler = (math.radians(60.0), 0.0, math.radians(45.0))
    bpy.context.scene.camera = cam
    rig_coll.objects.link(cam)
    bpy.context.scene.collection.objects.unlink(cam)

    bpy.ops.object.empty_add(type="PLAIN_AXES", location=(0.0, 0.0, 0.0))
    target = bpy.context.active_object
    target.name = "KeyframeTarget"
    rig_coll.objects.link(target)
    bpy.context.scene.collection.objects.unlink(target)

    bpy.context.scene.render.engine = "BLENDER_EEVEE"
    bpy.context.scene.render.film_transparent = True


def extract_iso_rig_from_legacy(legacy_blend: Path) -> None:
    """Open legacy main file, strip meshes, keep camera/light keyframe logic."""
    if not legacy_blend.is_file():
        raise FileNotFoundError(f"Legacy rig not found: {legacy_blend}")
    bpy.ops.wm.open_mainfile(filepath=str(legacy_blend))
    _remove_non_rig_objects()
    _organize_rig_collection()
    bpy.context.scene.render.film_transparent = True


def save_iso_rig_blend(dest: Path) -> Path:
    dest = dest.resolve()
    dest.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.wm.save_as_mainfile(filepath=str(dest))
    print(f"ISO_RIG_OK {dest}")
    return dest


# Back-compat alias used by tile_ortho_bake / tile_keyframe_bake imports.
def append_light_rig(light_blend: Path) -> None:
    append_iso_rig(light_blend, link=False)
