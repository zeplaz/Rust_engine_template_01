"""Tile atlas index — register baked atlases in _tile_atlas_index.ron (TILE-REAL-001 R1)."""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any

from rust_engine_mcp import assembly
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.schemas import load_json_file

TILE_ATLAS_INDEX_RON = "assets/configs/buildings/_tile_atlas_index.ron"
TILE_ATLAS_INDEX_JSON = "assets/configs/buildings/_tile_atlas_index.json"


def tile_atlas_index_ron_path() -> Path:
    return repo_root() / TILE_ATLAS_INDEX_RON


def tile_atlas_index_json_path() -> Path:
    return repo_root() / TILE_ATLAS_INDEX_JSON


def _ron_str(value: str) -> str:
    return json.dumps(value, ensure_ascii=False)


def _rel_repo_path(path: str | Path) -> str:
    p = Path(path)
    root = repo_root().resolve()
    if not p.is_absolute():
        p = (root / p).resolve()
    else:
        p = p.resolve()
    try:
        return str(p.relative_to(root)).replace("\\", "/")
    except ValueError:
        return str(p).replace("\\", "/")


def format_tile_atlas_index_ron(entries: list[dict[str, Any]]) -> str:
    lines = [
        "// Registered by rust_engine_mcp.tile_index — variant UVs in linked meta_json.",
        "(",
        "    schema_version: 1,",
        "    entries: [",
    ]
    for e in entries:
        lines.extend(
            [
                "        (",
                f"            atlas_id: {_ron_str(e['atlas_id'])},",
                f"            batch_id: {_ron_str(e['batch_id'])},",
                f"            assembly_id: {_ron_str(e.get('assembly_id') or '')},",
                f"            tile_id: {_ron_str(e['tile_id'])},",
                f"            atlas_png: {_ron_str(e['atlas_png'])},",
                f"            meta_json: {_ron_str(e['meta_json'])},",
                f"            development_tier: {_ron_str(e.get('development_tier') or 'lod0')},",
                f"            style_pack_id: {_ron_str(e.get('style_pack_id') or '')},",
                f"            ship_allowed: {'true' if e.get('ship_allowed', True) else 'false'},",
                "        ),",
            ]
        )
    lines.extend(["    ],", ")", ""])
    return "\n".join(lines)


def _parse_index_ron(text: str) -> list[dict[str, Any]]:
    """Minimal RON entry parser for upsert (fields we write)."""
    entries: list[dict[str, Any]] = []
    for block in re.findall(r"\(\s*atlas_id:", text):
        pass
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


def load_tile_atlas_index() -> list[dict[str, Any]]:
    json_path = tile_atlas_index_json_path()
    if json_path.is_file():
        data = json.loads(json_path.read_text(encoding="utf-8"))
        return list(data.get("entries") or [])
    ron_path = tile_atlas_index_ron_path()
    if ron_path.is_file():
        return _parse_index_ron(ron_path.read_text(encoding="utf-8"))
    return []


def write_tile_atlas_index(entries: list[dict[str, Any]]) -> dict[str, Any]:
    entries = sorted(entries, key=lambda e: str(e.get("atlas_id") or ""))
    ron_path = tile_atlas_index_ron_path()
    json_path = tile_atlas_index_json_path()
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


def _assembly_id_from_batch(batch: dict[str, Any] | None) -> tuple[str, str]:
    if not batch:
        return "", ""
    style_pack_id = ""
    assembly_id = ""
    ref = batch.get("assembly_ref") or {}
    style_pack_id = str(ref.get("style_pack_id") or batch.get("style_pack_id") or "")
    snap_path = ref.get("assembly_snapshot")
    if snap_path:
        p = Path(str(snap_path))
        if not p.is_absolute():
            p = repo_root() / p
        if p.is_file():
            assembly_id = str(assembly.load_assembly_snapshot(p).get("assembly_id") or "")
    if not assembly_id:
        vs_ref = batch.get("variant_set_ref")
        if vs_ref:
            vs = load_json_file(Path(str(vs_ref)) if Path(str(vs_ref)).is_absolute() else repo_root() / vs_ref)
            assembly_id = str(vs.get("assembly_id") or "")
            style_pack_id = style_pack_id or str(vs.get("style_pack_id") or "")
    return assembly_id, style_pack_id


def entry_from_meta(meta: dict[str, Any], batch: dict[str, Any] | None = None) -> dict[str, Any]:
    atlas_id = str(meta.get("atlas_id") or (batch or {}).get("atlas", {}).get("atlas_id") or "")
    batch_id = str(meta.get("batch_id") or (batch or {}).get("batch_id") or "")
    tile_id = str(meta.get("tile_id") or (batch or {}).get("tile_id") or "")
    atlas_png = meta.get("atlas_png") or (batch or {}).get("atlas", {}).get("output_png") or ""
    assembly_id, style_pack_id = _assembly_id_from_batch(batch)
    tier = str(
        (batch or {}).get("development_tier")
        or (batch or {}).get("source_tier")
        or "lod0"
    )
    ship = (batch or {}).get("ship")
    ship_allowed = bool(ship) if ship is not None else tier == "production"
    return {
        "atlas_id": atlas_id,
        "batch_id": batch_id,
        "assembly_id": assembly_id,
        "tile_id": tile_id,
        "atlas_png": _rel_repo_path(str(atlas_png)),
        "meta_json": "",
        "development_tier": tier,
        "style_pack_id": style_pack_id,
        "ship_allowed": ship_allowed,
    }


def register_tile_atlas_from_meta(
    meta_json_path: str | Path,
    *,
    batch: dict[str, Any] | None = None,
) -> dict[str, Any]:
    """Upsert one atlas row from atlas_meta.json (+ optional tile_batch body)."""
    p = Path(meta_json_path)
    if not p.is_absolute():
        meta_path = (repo_root() / p).resolve()
    else:
        meta_path = p.resolve()
    if not meta_path.is_file():
        raise FileNotFoundError(f"atlas_meta not found: {meta_path}")
    meta = load_json_file(meta_path)
    entry = entry_from_meta(meta, batch)
    entry["meta_json"] = _rel_repo_path(meta_path)
    if not entry["atlas_id"]:
        raise ValueError("atlas_meta missing atlas_id")
    entries = load_tile_atlas_index()
    entries = [e for e in entries if e.get("atlas_id") != entry["atlas_id"]]
    entries.append(entry)
    written = write_tile_atlas_index(entries)
    return {"ok": True, "entry": entry, **written}


def register_tile_atlas_from_batch(
    batch_id: str,
    *,
    tile_batch_path: str | Path | None = None,
) -> dict[str, Any]:
    """Register from staging batch_status + atlas_meta (+ optional batch JSON)."""
    root = repo_root()
    staging = root / "assets" / "staging" / "tiles" / batch_id
    status_path = staging / "batch_status.json"
    if not status_path.is_file():
        raise FileNotFoundError(f"batch_status missing: {status_path}")
    status = json.loads(status_path.read_text(encoding="utf-8"))
    meta_rel = str(status.get("meta_json") or staging / "atlas_meta.json")
    meta_path = Path(meta_rel) if Path(meta_rel).is_absolute() else root / meta_rel

    batch: dict[str, Any] | None = None
    if tile_batch_path:
        batch = load_json_file(Path(tile_batch_path))
    else:
        examples = root / "tools" / "mcp" / "schemas" / "examples"
        for candidate in examples.glob("tile_batch_*.json"):
            data = load_json_file(candidate)
            if str(data.get("batch_id")) == batch_id:
                batch = data
                break

    result = register_tile_atlas_from_meta(meta_path, batch=batch)
    result["batch_id"] = batch_id
    result["dry_run"] = bool(status.get("dry_run"))
    return result
