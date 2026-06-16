"""APS-MAT-003 — disk-cached material thumbnails for large catalogs."""

from __future__ import annotations

from pathlib import Path

from PIL import Image

from .material_profiles import MaterialProfileEntry, ensure_profile_textures
from .paths import repo_root

CACHE_DIR = repo_root() / "debug_runs" / "material_thumb_cache"
LIST_THUMB = 48
PREVIEW_THUMB = 168


def cache_path(profile_id: str, *, size: int = LIST_THUMB) -> Path:
    safe = profile_id.replace("/", "_")
    return CACHE_DIR / f"{safe}_{size}.png"


def get_cached_thumb(
    entry: MaterialProfileEntry,
    *,
    size: int = LIST_THUMB,
    force: bool = False,
) -> Image.Image | None:
    """Return cached RGB thumb; generate from albedo if missing."""
    path = cache_path(entry.profile_id, size=size)
    if not force and path.is_file():
        try:
            return Image.open(path).convert("RGB")
        except OSError:
            pass
    try:
        fresh = ensure_profile_textures(entry.profile_id, size=max(size, 256))
        albedo = fresh.albedo_path
    except Exception:
        albedo = entry.albedo_path
    if albedo is None or not albedo.is_file():
        return _placeholder(entry.profile_id, size=size)
    img = Image.open(albedo).convert("RGB")
    img.thumbnail((size, size), Image.Resampling.LANCZOS)
    path.parent.mkdir(parents=True, exist_ok=True)
    img.save(path)
    return img


def warm_thumbnail_cache(
    entries: list[MaterialProfileEntry],
    *,
    size: int = LIST_THUMB,
    limit: int | None = None,
) -> dict[str, int]:
    """Pre-warm thumbs for visible/filtered catalog (bounded for UI responsiveness)."""
    warmed = 0
    for entry in entries[: limit or len(entries)]:
        if get_cached_thumb(entry, size=size) is not None:
            warmed += 1
    return {"warmed": warmed, "total": len(entries)}


def _placeholder(profile_id: str, *, size: int) -> Image.Image:
    img = Image.new("RGB", (size, size), (120, 128, 140))
    try:
        from PIL import ImageDraw

        draw = ImageDraw.Draw(img)
        draw.text((4, size // 2 - 6), profile_id[:10], fill=(240, 240, 245))
    except Exception:
        pass
    return img
