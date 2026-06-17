"""APS-PREVIEW-001 — slot/module/material/combined preview renders for Assembly tab."""

from __future__ import annotations

import io
import json
from pathlib import Path
from typing import Any

from PIL import Image, ImageDraw

from .assembly_preview import try_render_glb_thumbnail_bytes
from .material_thumb_cache import get_cached_thumb
from .material_profiles import ensure_profile_textures
from .paths import repo_root

APS_PREVIEW_001_WITNESS = "debug_runs/aps_preview_001_live.json"
PREVIEW_SIZE = 128


def render_module_isolated(glb_rel: str, *, size: int = PREVIEW_SIZE) -> Image.Image | None:
    if not glb_rel or glb_rel.endswith("nonexistent.glb"):
        return _placeholder("module", size=size, color=(120, 128, 140))
    path = (repo_root() / glb_rel.replace("\\", "/")).resolve()
    if not path.is_file():
        return _placeholder(path.parent.name, size=size, color=(120, 128, 140))
    png = try_render_glb_thumbnail_bytes(path, resolution=(size, size))
    if png:
        img = Image.open(io.BytesIO(png)).convert("RGB")
        img.thumbnail((size, size), Image.Resampling.LANCZOS)
        return img
    return _placeholder(path.parent.name, size=size)


def render_material_preview(profile_id: str, *, size: int = PREVIEW_SIZE) -> Image.Image | None:
    if not profile_id or profile_id == "—":
        return None
    try:
        entry = ensure_profile_textures(profile_id, size=max(size, 256))
    except Exception:
        return _placeholder(profile_id[:10], size=size, color=(140, 130, 120))
    thumb = get_cached_thumb(entry, size=size)
    return thumb


def render_combined_preview(
    module_img: Image.Image | None,
    material_img: Image.Image | None,
    *,
    size: int = PREVIEW_SIZE,
) -> Image.Image:
    out = Image.new("RGB", (size, size), (200, 200, 205))
    if module_img is not None:
        mod = module_img.copy()
        mod.thumbnail((size // 2, size), Image.Resampling.LANCZOS)
        out.paste(mod, (0, 0))
    if material_img is not None:
        mat = material_img.copy()
        mat.thumbnail((size // 2, size), Image.Resampling.LANCZOS)
        out.paste(mat, (size // 2, 0))
    return out


def render_placement_context_strip(
    snapshot: dict[str, Any] | None,
    *,
    selected: dict[str, Any] | None = None,
    assembly_thumb: Image.Image | None = None,
    size: int = PREVIEW_SIZE,
) -> Image.Image:
    if assembly_thumb is not None:
        img = assembly_thumb.copy()
        img.thumbnail((size, size), Image.Resampling.LANCZOS)
        return img
    img = Image.new("RGB", (size, size), (232, 234, 238))
    draw = ImageDraw.Draw(img)
    placements = list((snapshot or {}).get("module_placements") or [])
    if not placements:
        draw.text((8, size // 2 - 6), "no placements", fill=(80, 80, 90))
        return img
    xs = [int(p.get("grid_x") or 0) for p in placements if isinstance(p, dict)]
    ys = [int(p.get("grid_y") or 0) for p in placements if isinstance(p, dict)]
    if not xs:
        return img
    min_x, max_x = min(xs), max(xs)
    min_y, max_y = min(ys), max(ys)
    span_x = max(1, max_x - min_x + 1)
    span_y = max(1, max_y - min_y + 1)
    cell = max(4, min(size // max(span_x, span_y), 24))
    for row in placements:
        if not isinstance(row, dict):
            continue
        gx = int(row.get("grid_x") or 0)
        gy = int(row.get("grid_y") or 0)
        px = 4 + (gx - min_x) * cell
        py = 4 + (gy - min_y) * cell
        fill = (90, 140, 200)
        if selected and row is selected:
            fill = (220, 120, 40)
        elif selected and row.get("grid_x") == selected.get("grid_x") and row.get("grid_y") == selected.get("grid_y"):
            fill = (220, 120, 40)
        draw.rectangle([px, py, px + cell - 1, py + cell - 1], fill=fill, outline=(40, 40, 50))
    return img


def write_aps_preview_001_witness(
    *,
    module_ok: bool,
    material_ok: bool,
    combined_ok: bool,
) -> Path:
    body = {
        "gate_id": "APS-PREVIEW-001",
        "ok": module_ok and material_ok,
        "green": module_ok and material_ok and combined_ok,
        "module_thumb_ok": module_ok,
        "material_thumb_ok": material_ok,
        "combined_thumb_ok": combined_ok,
        "preview_size_px": PREVIEW_SIZE,
        "panel": "art_pipeline_suite.slot_preview_panel.SlotPreviewPanel",
    }
    out = repo_root() / APS_PREVIEW_001_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return out


def _placeholder(label: str, *, size: int, color: tuple[int, int, int]) -> Image.Image:
    img = Image.new("RGB", (size, size), color)
    try:
        draw = ImageDraw.Draw(img)
        draw.text((4, size // 2 - 6), label[:14], fill=(240, 240, 245))
    except Exception:
        pass
    return img
