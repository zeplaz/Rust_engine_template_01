"""MCP-PROD-INDEX — production atlas row in _tile_atlas_index.ron with ship_allowed true."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.prod_tile_val import PRODUCTION_BATCH
from rust_engine_mcp.tile_index import (
    load_tile_atlas_index,
    register_tile_atlas_from_batch,
    write_tile_atlas_index,
)

MCP_PROD_INDEX_WITNESS = "debug_runs/mcp_prod_index_live.json"
ATLAS_ID = "rowhouse_victorian_production_v1"
BATCH_ID = "tile_rowhouse_victorian_production_v1"


def promote_production_tile_index_ship_allowed() -> dict[str, Any]:
    """Register staging atlas + set production row ship_allowed true."""
    register_tile_atlas_from_batch(BATCH_ID, tile_batch_path=PRODUCTION_BATCH)
    entries = load_tile_atlas_index()
    row: dict[str, Any] | None = None
    for entry in entries:
        if str(entry.get("atlas_id") or "") == ATLAS_ID:
            entry["ship_allowed"] = True
            entry["development_tier"] = "production"
            row = entry
            break
    if row is None:
        raise ValueError(f"atlas_id not in index: {ATLAS_ID}")
    written = write_tile_atlas_index(entries)
    return {"ok": True, "entry": row, **written}


def mcp_prod_index_checks() -> dict[str, Any]:
    entries = load_tile_atlas_index()
    row = next((e for e in entries if str(e.get("atlas_id") or "") == ATLAS_ID), None)
    if row is None:
        return {
            "gate_id": "MCP-PROD-INDEX",
            "ok": False,
            "green": False,
            "atlas_id": ATLAS_ID,
            "index_row_present": False,
        }
    atlas_png = repo_root() / str(row.get("atlas_png") or "")
    meta_json = repo_root() / str(row.get("meta_json") or "")
    green = (
        row.get("development_tier") == "production"
        and bool(row.get("ship_allowed"))
        and atlas_png.is_file()
        and meta_json.is_file()
    )
    return {
        "gate_id": "MCP-PROD-INDEX",
        "ok": green,
        "green": green,
        "atlas_id": ATLAS_ID,
        "batch_id": row.get("batch_id"),
        "index_row_present": True,
        "development_tier": row.get("development_tier"),
        "ship_allowed": bool(row.get("ship_allowed")),
        "atlas_png": str(row.get("atlas_png") or ""),
        "meta_json": str(row.get("meta_json") or ""),
        "atlas_png_on_disk": atlas_png.is_file(),
        "meta_json_on_disk": meta_json.is_file(),
    }


def refresh_mcp_prod_index_witness() -> bool:
    promote_production_tile_index_ship_allowed()
    payload = mcp_prod_index_checks()
    out: Path = repo_root() / MCP_PROD_INDEX_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return bool(payload.get("green"))
