"""Landscape iso atlas index — separate from buildings `_tile_atlas_index.ron`."""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.schemas import load_json_file
from rust_engine_mcp.tile_index import (
    _rel_repo_path,
    _ron_str,
    entry_from_meta,
    format_tile_atlas_index_ron,
)

LANDSCAPE_ATLAS_INDEX_RON = "assets/configs/landscape/_landscape_atlas_index.ron"
LANDSCAPE_ATLAS_INDEX_JSON = "assets/configs/landscape/_landscape_atlas_index.json"


def landscape_atlas_index_ron_path() -> Path:
    return repo_root() / LANDSCAPE_ATLAS_INDEX_RON


def landscape_atlas_index_json_path() -> Path:
    return repo_root() / LANDSCAPE_ATLAS_INDEX_JSON


def _parse_index_ron(text: str) -> list[dict[str, Any]]:
    entries: list[dict[str, Any]] = []
    for m in re.finditer(
        r"atlas_id:\s*\"([^\"]+)\".*?batch_id:\s*\"([^\"]+)\".*?assembly_id:\s*\"([^\"]*?)\".*?"
        r"tile_id:\s*\"([^\"]+)\".*?atlas_png:\s*\"([^\"]+)\".*?meta_json:\s*\"([^\"]+)\".*?"
        r"development_tier:\s*\"([^\"]+)\".*?style_pack_id:\s*\"([^\"]*)\"(?:.*?ship_allowed:\s*(true|false))?",
        text,
        re.DOTALL,
    ):
        ship_raw = m.group(9)
        ship_allowed = ship_raw != "false" if ship_raw else str(m.group(7)) == "production"
        entries.append(
            {
                "atlas_id": m.group(1),
                "batch_id": m.group(2),
                "assembly_id": m.group(3),
                "tile_id": m.group(4),
                "atlas_png": m.group(5),
                "meta_json": m.group(6),
                "development_tier": m.group(7),
                "style_pack_id": m.group(8),
                "ship_allowed": ship_allowed,
            }
        )
    return entries


def load_landscape_atlas_index() -> list[dict[str, Any]]:
    json_path = landscape_atlas_index_json_path()
    if json_path.is_file():
        data = json.loads(json_path.read_text(encoding="utf-8"))
        return list(data.get("entries") or [])
    ron_path = landscape_atlas_index_ron_path()
    if ron_path.is_file():
        return _parse_index_ron(ron_path.read_text(encoding="utf-8"))
    return []


def write_landscape_atlas_index(entries: list[dict[str, Any]]) -> dict[str, Any]:
    entries = sorted(entries, key=lambda e: str(e.get("atlas_id") or ""))
    ron_path = landscape_atlas_index_ron_path()
    json_path = landscape_atlas_index_json_path()
    ron_path.parent.mkdir(parents=True, exist_ok=True)
    ron_path.write_text(format_tile_atlas_index_ron(entries), encoding="utf-8")
    json_path.write_text(
        json.dumps({"schema_version": 1, "entries": entries}, indent=2) + "\n",
        encoding="utf-8",
    )
    return {
        "written": str(ron_path),
        "json_mirror": str(json_path),
        "entry_count": len(entries),
        "atlas_ids": [e["atlas_id"] for e in entries],
    }


def register_landscape_atlas_from_meta(
    meta_json_path: str | Path,
    *,
    batch: dict[str, Any] | None = None,
) -> dict[str, Any]:
    p = Path(meta_json_path)
    meta_path = (repo_root() / p).resolve() if not p.is_absolute() else p.resolve()
    if not meta_path.is_file():
        raise FileNotFoundError(f"atlas_meta not found: {meta_path}")
    meta = load_json_file(meta_path)
    entry = entry_from_meta(meta, batch)
    entry["meta_json"] = _rel_repo_path(meta_path)
    entry["assembly_id"] = ""
    entry["style_pack_id"] = "landscape_lg5"
    if not entry["atlas_id"]:
        raise ValueError("atlas_meta missing atlas_id")
    entries = load_landscape_atlas_index()
    entries = [e for e in entries if e.get("atlas_id") != entry["atlas_id"]]
    entries.append(entry)
    written = write_landscape_atlas_index(entries)
    return {"ok": True, "entry": entry, **written}


def landscape_lg5_registry_stamped() -> bool:
    return any(e.get("atlas_id") == "landscape_lg5_pilot_v1" for e in load_landscape_atlas_index())


def landscape_atlas_registered(atlas_id: str) -> bool:
    return any(str(e.get("atlas_id") or "") == atlas_id for e in load_landscape_atlas_index())


def landscape_expanded_atlas_registered() -> bool:
    return landscape_atlas_registered("landscape_lg5_expanded_v1")
