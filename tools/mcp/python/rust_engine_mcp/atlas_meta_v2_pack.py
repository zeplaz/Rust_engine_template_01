"""TILE-FIX-008/010 — pack facing-aware stills → atlas.png + atlas_meta v2 lookups."""

from __future__ import annotations

import json
import math
import re
from pathlib import Path
from typing import Any

from rust_engine_mcp.building_definition import BakeCell
from rust_engine_mcp.paths import repo_root

try:
    from PIL import Image
except ImportError as exc:  # pragma: no cover
    raise ImportError("Install Pillow: pip install Pillow") from exc

_CELL_PNG_RE = re.compile(
    r"^(?P<variant>[a-z0-9_]+)_f(?P<facing>\d+)(?:_frame(?P<frame>\d+))?\.png$"
)


def cell_png_basename(cell: BakeCell) -> str:
    if cell.frame > 0:
        return f"{cell.variant_key}_f{cell.facing}_frame{cell.frame}.png"
    return f"{cell.variant_key}_f{cell.facing}.png"


def parse_cell_png_name(name: str) -> tuple[str, int, int] | None:
    m = _CELL_PNG_RE.match(name)
    if not m:
        return None
    return (
        m.group("variant"),
        int(m.group("facing")),
        int(m.group("frame") or 0),
    )


def pack_cells_to_atlas(
    cells: list[BakeCell],
    staging_dir: Path,
    *,
    atlas_png: Path,
    tile_px: int = 128,
    columns: int | None = None,
) -> dict[str, Any]:
    """Pack ordered cells into one atlas; return meta body fields (lookups, cols, rows)."""
    if not cells:
        raise ValueError("pack_cells_to_atlas requires at least one cell")
    cols = columns or min(8, max(4, int(math.ceil(math.sqrt(len(cells))))))
    rows = max(1, (len(cells) + cols - 1) // cols)
    atlas_w = cols * tile_px
    atlas_h = rows * tile_px
    atlas = Image.new("RGBA", (atlas_w, atlas_h), (0, 0, 0, 0))
    lookups: list[dict[str, Any]] = []
    missing: list[str] = []

    for i, cell in enumerate(cells):
        name = cell_png_basename(cell)
        src = staging_dir / name
        col = i % cols
        row = i // cols
        if not src.is_file():
            missing.append(name)
            continue
        tile = Image.open(src).convert("RGBA")
        if tile.size != (tile_px, tile_px):
            tile = tile.resize((tile_px, tile_px), Image.Resampling.NEAREST)
        atlas.paste(tile, (col * tile_px, row * tile_px))
        lookups.append(
            {
                "variant": cell.variant_key,
                "facing": cell.facing,
                "frame": cell.frame,
                "grid": [col, row],
                "uv": [
                    col / cols,
                    row / rows,
                    1.0 / cols,
                    1.0 / rows,
                ],
                "png": name,
            }
        )

    if missing:
        raise FileNotFoundError(f"Missing {len(missing)} cell PNG(s), e.g. {missing[:3]}")

    atlas_png.parent.mkdir(parents=True, exist_ok=True)
    atlas.save(atlas_png)
    return {
        "columns": cols,
        "rows": rows,
        "tile_px": tile_px,
        "lookups": lookups,
        "cell_count": len(cells),
    }


def write_atlas_meta_v2(
    *,
    batch: dict[str, Any],
    pack_info: dict[str, Any],
    atlas_png_rel: str,
    visual_config_rel: str,
    minimum_g4_ship: bool = False,
) -> Path:
    atlas = batch.get("atlas") or {}
    meta_rel = str(atlas.get("meta_json") or "")
    if not meta_rel:
        batch_id = str(batch.get("batch_id") or "tile_batch")
        meta_rel = f"assets/staging/tiles/{batch_id}/atlas_meta.json"
    meta_path = Path(meta_rel)
    if not meta_path.is_absolute():
        meta_path = repo_root() / meta_rel
    meta_path.parent.mkdir(parents=True, exist_ok=True)

    render_contract = dict(batch.get("render_contract") or {})
    if not render_contract.get("facings"):
        render_contract["facings"] = int(pack_info.get("facings") or 8)
    if not render_contract.get("tile_px"):
        render_contract["tile_px"] = int(pack_info.get("tile_px") or 128)

    body: dict[str, Any] = {
        "schema_version": 2,
        "atlas_id": str(atlas.get("atlas_id") or batch.get("atlas_id") or ""),
        "batch_id": str(batch.get("batch_id") or ""),
        "tile_id": str(batch.get("tile_id") or ""),
        "atlas_png": atlas_png_rel.replace("\\", "/"),
        "visual_config": visual_config_rel.replace("\\", "/"),
        "columns": int(pack_info["columns"]),
        "rows": int(pack_info["rows"]),
        "tile_px": int(pack_info["tile_px"]),
        "render_contract": render_contract,
        "lookups": pack_info["lookups"],
    }
    if minimum_g4_ship:
        body["minimum_g4_ship"] = True
        body["lookup_mode"] = "minimum_g4"
    meta_path.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return meta_path
