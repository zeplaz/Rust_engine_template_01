"""APS-MAT-STUDIO-PHASE-A — material preview modes (sphere / wall / building section)."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from PIL import Image, ImageDraw

from .assembly_preview import collect_preview_placements
from .material_profiles import MaterialProfileEntry, ensure_profile_textures, load_material_profile_catalog
from .material_thumb_cache import warm_thumbnail_cache
from .paths import repo_root

APS_MATERIAL_STUDIO_WITNESS = "debug_runs/aps_material_studio_live.json"
PREVIEW_ROOT = repo_root() / "debug_runs" / "material_studio_previews"
DEFAULT_SNAPSHOT = (
    "tools/mcp/schemas/examples/assembly_snapshot_warehouse_industrial_west_production_v1.json"
)


def _profile_dir(profile_id: str) -> Path:
    safe = profile_id.replace("/", "_")
    return PREVIEW_ROOT / safe


def render_sphere_preview(entry: MaterialProfileEntry, *, size: int = 256) -> Image.Image:
    albedo = entry.albedo_path
    if albedo is None or not albedo.is_file():
        return _flat_placeholder(entry.profile_id, size=size)
    tex = Image.open(albedo).convert("RGB")
    tex = tex.resize((size, size), Image.Resampling.LANCZOS)
    mask = Image.new("L", (size, size), 0)
    draw = ImageDraw.Draw(mask)
    draw.ellipse([4, 4, size - 4, size - 4], fill=255)
    bg = Image.new("RGB", (size, size), (210, 215, 220))
    bg.paste(tex, mask=mask)
    return bg


def render_wall_strip_preview(entry: MaterialProfileEntry, *, size: int = 256) -> Image.Image:
    albedo = entry.albedo_path
    if albedo is None or not albedo.is_file():
        return _flat_placeholder(entry.profile_id, size=size)
    tile = Image.open(albedo).convert("RGB")
    tile = tile.resize((64, 64), Image.Resampling.LANCZOS)
    out = Image.new("RGB", (size, size), (180, 180, 185))
    for y in range(0, size, 64):
        for x in range(0, size, 64):
            out.paste(tile, (x, y))
    return out


def render_building_section_preview(
    entry: MaterialProfileEntry,
    *,
    snapshot_path: Path | str | None = None,
    size: int = 256,
) -> tuple[Image.Image, dict[str, Any]]:
    snap_path = Path(snapshot_path or repo_root() / DEFAULT_SNAPSHOT)
    meta: dict[str, Any] = {
        "mode": "building_section",
        "degraded": True,
        "placement_count": 0,
        "missing_glbs": [],
        "source": snap_path.relative_to(repo_root()).as_posix() if snap_path.is_file() else str(snap_path),
    }
    if snap_path.is_file():
        data = json.loads(snap_path.read_text(encoding="utf-8"))
        placements, missing = collect_preview_placements(data)
        meta["placement_count"] = len(placements)
        meta["missing_glbs"] = missing
        meta["degraded"] = len(missing) > 0
    wall = render_wall_strip_preview(entry, size=size)
    meta["fallback"] = "wall_strip"
    return wall, meta


def preview_modes_for_profile(profile_id: str) -> dict[str, Any]:
    entry = ensure_profile_textures(profile_id, size=512)
    out_dir = _profile_dir(profile_id)
    out_dir.mkdir(parents=True, exist_ok=True)
    modes: dict[str, Any] = {
        "profile_id": profile_id,
        "category": entry.category,
        "texture_status": entry.texture_status(),
        "ok": True,
    }
    for key, renderer in (
        ("sphere", lambda: render_sphere_preview(entry)),
        ("wall_strip", lambda: render_wall_strip_preview(entry)),
    ):
        img = renderer()
        rel = out_dir.relative_to(repo_root()) / f"{key}.png"
        path = repo_root() / rel
        img.save(path)
        modes[key] = {"ok": True, "path": rel.as_posix()}
    bimg, bmeta = render_building_section_preview(entry)
    brel = out_dir.relative_to(repo_root()) / "building_section.png"
    bimg.save(repo_root() / brel)
    modes["building_section"] = {"ok": True, "path": brel.as_posix(), **bmeta}
    return modes


def write_material_studio_witness(*, profile_id: str = "concrete_grey_01") -> dict[str, Any]:
    catalog = load_material_profile_catalog()
    warmed = warm_thumbnail_cache(catalog)
    modes = preview_modes_for_profile(profile_id)
    categories = sorted({e.category for e in catalog})
    body = {
        "gate_id": "APS-MAT-STUDIO-PHASE-A",
        "ok": modes.get("ok", False),
        "profile_id": profile_id,
        "catalog_count": len(catalog),
        "categories": categories,
        "preview_modes": modes,
        "ui": {
            "materials_tab": "tools/mcp/art_pipeline_suite/app.py",
            "shared_widget": "material_library_widget.py",
            "layout": "studio_tree",
            "list_rows": "scrollable_thumb_rows",
            "thumb_cache_dir": "debug_runs/material_thumb_cache",
            "list_thumb_px": 48,
            "assembly_assign": "assembly_panel.py MaterialBrowserPanel",
            "context_thumb": "assembly_preview_panel → slot_preview_panel",
        },
        "aps_mat_003": warmed,
        "ship_policy": "material authority on snapshot only — not Blender viewport",
    }
    out = repo_root() / APS_MATERIAL_STUDIO_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body


def _flat_placeholder(profile_id: str, *, size: int) -> Image.Image:
    img = Image.new("RGB", (size, size), (130, 135, 145))
    draw = ImageDraw.Draw(img)
    draw.text((8, size // 2 - 8), profile_id[:16], fill=(240, 240, 245))
    return img
