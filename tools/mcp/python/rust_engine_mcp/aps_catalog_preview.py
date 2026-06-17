"""APS-PREVIEW-CATALOG-001 — module list thumbnails for Catalog tab."""

from __future__ import annotations

import io
import json
from pathlib import Path

from PIL import Image

from .assembly_preview import try_render_glb_thumbnail_bytes
from .paths import repo_root

APS_PREVIEW_CATALOG_WITNESS = "debug_runs/aps_preview_catalog_live.json"
LIST_THUMB_PX = 48
CACHE_DIR = repo_root() / "debug_runs" / "catalog_thumb_cache"


def _cache_path(module_id: str, *, size: int = LIST_THUMB_PX) -> Path:
    safe = module_id.replace("/", "_").replace("\\", "_")
    return CACHE_DIR / f"{safe}_{size}.png"


def render_module_list_thumb(
    glb_path: Path | str,
    *,
    module_id: str | None = None,
    size: int = LIST_THUMB_PX,
) -> Image.Image | None:
    """Isolated GLB ortho thumb for catalog list rows (trimesh optional)."""
    path = Path(glb_path)
    if not path.is_absolute():
        path = (repo_root() / path).resolve()
    mid = module_id or path.parent.name
    cache = _cache_path(mid, size=size)
    if cache.is_file():
        try:
            return Image.open(cache).convert("RGB")
        except OSError:
            pass
    if not path.is_file():
        return _placeholder(mid, size=size)
    png = try_render_glb_thumbnail_bytes(path, resolution=(max(size, 64), max(size, 64)))
    if png:
        img = Image.open(io.BytesIO(png)).convert("RGB")
    else:
        img = _placeholder(mid, size=size)
    img.thumbnail((size, size), Image.Resampling.LANCZOS)
    cache.parent.mkdir(parents=True, exist_ok=True)
    img.save(cache)
    return img


def _placeholder(module_id: str, *, size: int) -> Image.Image:
    img = Image.new("RGB", (size, size), (96, 104, 116))
    try:
        from PIL import ImageDraw

        draw = ImageDraw.Draw(img)
        draw.text((4, size // 2 - 6), module_id[:12], fill=(235, 235, 240))
    except Exception:
        pass
    return img


def write_aps_preview_catalog_witness(
    *,
    sample_module_id: str,
    thumb_ok: bool,
) -> Path:
    body = {
        "program_id": "APS-PREVIEW-CATALOG-001",
        "green": thumb_ok,
        "sample_module_id": sample_module_id,
        "list_thumb_px": LIST_THUMB_PX,
        "cache_dir": CACHE_DIR.relative_to(repo_root()).as_posix(),
        "sidecar_truth_line": (
            "Sidecar tags are hints only — assembly snapshot semantic_tags win at ship."
        ),
        "panel": "art_pipeline_suite.catalog.CatalogPanel",
    }
    out = repo_root() / APS_PREVIEW_CATALOG_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return out
