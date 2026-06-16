"""RT-REG-001 — tile-atlas-register rowhouse production + lookup stamp witness."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

from .paths import repo_root
from .prod_index import promote_production_tile_index_ship_allowed
from .schemas import load_json_file
from .tile_index import load_tile_atlas_index

RT_REGISTRY_WITNESS = "debug_runs/rt_registry_001_live.json"
ROWHOUSE_PRODUCTION_BATCH = "tile_rowhouse_victorian_production_v1"
ROWHOUSE_PRODUCTION_ATLAS_ID = "rowhouse_victorian_production_v1"


def lookup_stamp_from_meta(meta: dict[str, Any]) -> dict[str, Any]:
    """Deterministic lookup-table stamp for engine registry parity."""
    schema_version = int(meta.get("schema_version") or 1)
    keys: list[str] = []
    if schema_version >= 2:
        for row in meta.get("lookups") or []:
            if not isinstance(row, dict):
                continue
            variant = str(row.get("variant_key") or row.get("key") or "")
            facing = int(row.get("facing") or 0)
            frame = int(row.get("frame") or 0)
            keys.append(f"{variant}:{facing}:{frame}")
    else:
        for row in meta.get("tiles") or []:
            if isinstance(row, dict) and row.get("variant_key"):
                keys.append(str(row["variant_key"]))
    keys_sorted = sorted(keys)
    digest = hashlib.sha256("\n".join(keys_sorted).encode("utf-8")).hexdigest()[:16]
    return {
        "schema_version": schema_version,
        "lookup_count": len(keys_sorted),
        "stamp": digest,
        "variant_keys_sample": keys_sorted[:8],
    }


def _find_index_entry(atlas_id: str) -> dict[str, Any] | None:
    for entry in load_tile_atlas_index():
        if str(entry.get("atlas_id") or "") == atlas_id:
            return entry
    return None


def rt_registry_register_rowhouse_production(*, batch_id: str | None = None) -> dict[str, Any]:
    """Register rowhouse production batch and return lookup stamp rollup."""
    bid = batch_id or ROWHOUSE_PRODUCTION_BATCH
    if bid != ROWHOUSE_PRODUCTION_BATCH:
        from .tile_index import register_tile_atlas_from_batch

        register_result = register_tile_atlas_from_batch(bid)
    else:
        register_result = promote_production_tile_index_ship_allowed()
    entry = register_result.get("entry") or {}
    atlas_id = str(entry.get("atlas_id") or ROWHOUSE_PRODUCTION_ATLAS_ID)
    meta_rel = str(entry.get("meta_json") or "")
    meta_path = Path(meta_rel) if Path(meta_rel).is_absolute() else repo_root() / meta_rel
    meta = load_json_file(meta_path) if meta_path.is_file() else {}
    stamp = lookup_stamp_from_meta(meta)
    atlas_png = Path(str(entry.get("atlas_png") or ""))
    if not atlas_png.is_absolute():
        atlas_png = repo_root() / atlas_png
    png_ok = atlas_png.is_file()
    ship_allowed = bool(entry.get("ship_allowed"))
    ok = (
        register_result.get("ok") is True
        and stamp["lookup_count"] > 0
        and bool(stamp["stamp"])
        and png_ok
        and ship_allowed
    )
    return {
        "schema": "rt_registry_v1",
        "ok": ok,
        "batch_id": bid,
        "atlas_id": atlas_id,
        "entry": entry,
        "lookup_stamp": stamp,
        "atlas_png_present": png_ok,
        "ship_allowed": ship_allowed,
        "index_entry_count": register_result.get("entry_count"),
        "register_written": register_result.get("written"),
    }


def refresh_rt_registry_001_witness(*, batch_id: str | None = None) -> bool:
    body = rt_registry_register_rowhouse_production(batch_id=batch_id)
    green = bool(body.get("ok"))
    payload = {
        "gate_id": "RT-REG-001",
        "green": green,
        "ok": green,
        "batch_id": body.get("batch_id"),
        "atlas_id": body.get("atlas_id"),
        "lookup_stamp": body.get("lookup_stamp"),
        "ship_allowed": body.get("ship_allowed"),
        "atlas_png_present": body.get("atlas_png_present"),
        "index_entry_count": body.get("index_entry_count"),
    }
    out = repo_root() / RT_REGISTRY_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return green


def rt_registry_001_green() -> bool:
    path = repo_root() / RT_REGISTRY_WITNESS
    if not path.is_file():
        return False
    data = json.loads(path.read_text(encoding="utf-8"))
    stamp = data.get("lookup_stamp") or {}
    return (
        data.get("green") is True
        and data.get("gate_id") == "RT-REG-001"
        and int(stamp.get("lookup_count") or 0) > 0
        and bool(stamp.get("stamp"))
        and data.get("ship_allowed") is True
        and data.get("atlas_png_present") is True
    )
