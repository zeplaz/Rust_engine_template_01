"""MCP-PROD-TILE-VAL — production tile_batch validate + variant_matrix_expand witness."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators import run_validator
from rust_engine_mcp.variant_matrix_expand import variant_matrix_expand

MCP_PROD_TILE_VAL_WITNESS = "debug_runs/mcp_prod_tile_val_live.json"
PRODUCTION_BATCH = (
    repo_root() / "tools/mcp/schemas/examples/tile_batch_rowhouse_victorian_production_v1.json"
)
G4_WITNESS = repo_root() / "debug_runs/art_pipeline/rowhouse_production_atlas_g0_g4_live.json"


def _load_batch() -> dict[str, Any]:
    return json.loads(PRODUCTION_BATCH.read_text(encoding="utf-8"))


def mcp_prod_tile_val_checks() -> dict[str, Any]:
    """Run tile_batch_validate + variant_matrix_expand checks for rowhouse production."""
    batch = _load_batch()
    report = run_validator("tile_batch", str(PRODUCTION_BATCH))
    tile_batch_ok = report.status == "passed"
    keyframe_pack = str(batch.get("bake_source") or "") == "keyframe_pack"

    matrix_ref = str(batch.get("matrix_ref") or "")
    expand = variant_matrix_expand(matrix_ref, write_batch=False)
    expand_ok = bool(expand.get("ok")) and int(expand.get("variant_count") or 0) >= 6

    batch_keys = {str(v.get("variant_key") or "") for v in batch.get("variants") or []}
    batch_keys.discard("")
    expand_keys = {str(k) for k in expand.get("variant_keys") or []}
    matrix_keys_cover_batch = batch_keys <= expand_keys

    g4_green = False
    proceed_ship = False
    if G4_WITNESS.is_file():
        g4 = json.loads(G4_WITNESS.read_text(encoding="utf-8"))
        g4_green = bool(g4.get("green"))
        proceed_ship = bool(g4.get("proceed_ship"))

    atlas = batch.get("atlas") or {}
    atlas_png = repo_root() / str(atlas.get("output_png") or "")
    meta_json = repo_root() / str(atlas.get("meta_json") or "")
    assets_on_disk = atlas_png.is_file() and meta_json.is_file()

    green = (
        tile_batch_ok
        and keyframe_pack
        and expand_ok
        and matrix_keys_cover_batch
        and g4_green
        and proceed_ship
        and assets_on_disk
    )
    return {
        "gate_id": "MCP-PROD-TILE-VAL",
        "ok": green,
        "green": green,
        "batch_id": batch.get("batch_id"),
        "tile_batch_validate_passed": tile_batch_ok,
        "bake_source_keyframe_pack": keyframe_pack,
        "variant_matrix_expand_ok": expand_ok,
        "matrix_keys_cover_batch": matrix_keys_cover_batch,
        "batch_variant_count": len(batch_keys),
        "matrix_variant_count": len(expand_keys),
        "g4_witness_green": g4_green,
        "proceed_ship": proceed_ship,
        "atlas_png_on_disk": atlas_png.is_file(),
        "atlas_meta_on_disk": meta_json.is_file(),
    }


def refresh_mcp_prod_tile_val_witness() -> bool:
    payload = mcp_prod_tile_val_checks()
    out: Path = repo_root() / MCP_PROD_TILE_VAL_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return bool(payload.get("green"))
