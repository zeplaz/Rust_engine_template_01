"""CITY-G2-C5-001 — palette_family index patch + atlas __pal_* keys + MCP witness."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

TASK_ID = "CITY-G2-C5-001"
WITNESS_REL = "debug_runs/city_g2_c5_mcp_live.json"
SCHEMA_REL = "tools/mcp/schemas/palette_catalog_v1.schema.json"
PALETTE_INDEX_REL = "assets/configs/buildings/_palette_catalog_index.ron"
ROWHOUSE_META_REL = (
    "assets/staging/tiles/tile_rowhouse_victorian_production_v1/atlas_meta.json"
)

STYLE_PACK_PALETTE: dict[str, tuple[str, int, str]] = {
    "style_industrial_west": ("palette_industrial_west", 3, "iw_rust_clean"),
    "style_colonial": ("palette_colonial_res", 3, "cr_brick_red"),
    "style_victorian": ("palette_rowhouse_urban", 4, "rh_brownstone"),
}

PILOT_MODULE_IDS: frozenset[str] = frozenset(
    {
        "wall_concrete_2u",
        "roof_shed",
        "door_warehouse",
        "win_industrial_3u",
        "corner_L",
        "wall_brick_1u",
        "door_residential",
        "roof_pitched_gable",
        "win_single_1u",
        "win_double_1u",
        "door_shop",
        "stack_chimney_1u",
    }
)

ROWHOUSE_PAL_VARIATIONS: tuple[str, ...] = (
    "rh_brownstone",
    "rh_painted_row",
    "rh_brick_party",
    "rh_rehab_mixed",
)


def palette_fields_for_entry(entry: dict[str, Any]) -> dict[str, Any]:
    """Return optional palette_family fields for a module index row."""
    style_pack = str(entry.get("style_pack") or "")
    module_id = str(entry.get("module_id") or "")
    tier = str(entry.get("development_tier") or "")
    if tier not in ("lod0", "production"):
        return {}
    if module_id not in PILOT_MODULE_IDS:
        return {}
    row = STYLE_PACK_PALETTE.get(style_pack)
    if row is None:
        return {}
    family, count, default_var = row
    return {
        "palette_family": family,
        "palette_variation_count": count,
        "default_variation_id": default_var,
    }


def _load_json(path: Path) -> dict[str, Any] | None:
    if not path.is_file():
        return None
    return json.loads(path.read_text(encoding="utf-8"))


def wire_rowhouse_atlas_palette_keys(*, repo: Path | None = None) -> dict[str, Any]:
    """Add clean_day__pal_{variation_id} keys mirroring clean_day UV (v1 pilot)."""
    root = repo or repo_root()
    path = root / ROWHOUSE_META_REL
    if not path.is_file():
        return {"ok": False, "error": f"missing {ROWHOUSE_META_REL}"}
    body = json.loads(path.read_text(encoding="utf-8"))
    tiles: list[dict[str, Any]] = list(body.get("tiles") or [])
    by_key = {str(t.get("variant_key")): t for t in tiles}
    clean = by_key.get("clean_day")
    if clean is None:
        return {"ok": False, "error": "clean_day missing in atlas meta"}
    added: list[str] = []
    for var_id in ROWHOUSE_PAL_VARIATIONS:
        key = f"clean_day__pal_{var_id}"
        if key in by_key:
            continue
        clone = {
            "variant_key": key,
            "grid": list(clean.get("grid") or [0, 0]),
            "uv": list(clean.get("uv") or [0.0, 0.0, 0.25, 0.25]),
        }
        if clean.get("png"):
            clone["png"] = clean["png"]
        tiles.append(clone)
        added.append(key)
    if added:
        body["tiles"] = tiles
        body["variant_count"] = len(tiles)
        path.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return {"ok": True, "added": added, "path": ROWHOUSE_META_REL}


def audit_city_g2_c5_mcp(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    checks: dict[str, bool] = {
        "schema_file": (root / SCHEMA_REL).is_file(),
        "palette_index": (root / PALETTE_INDEX_REL).is_file(),
    }
    for palette_id, rel in (
        ("palette_industrial_west_v1", "assets/configs/buildings/palettes/industrial_west_v1.ron"),
        ("palette_colonial_res_v1", "assets/configs/buildings/palettes/colonial_res_v1.ron"),
        ("palette_rowhouse_urban_v1", "assets/configs/buildings/palettes/rowhouse_urban_v1.ron"),
    ):
        checks[f"{palette_id}_ron"] = (root / rel).is_file()

    index_text = (root / "assets/configs/buildings/_module_index.ron").read_text(encoding="utf-8")
    pilot_hits = sum(1 for mid in PILOT_MODULE_IDS if f'module_id: "{mid}"' in index_text or f"module_id: '{mid}'" in index_text)
    palette_field_rows = index_text.count("palette_family:")
    checks["module_index_palette_rows"] = palette_field_rows >= 6
    checks["module_index_pilot_modules_present"] = pilot_hits >= 6

    meta = _load_json(root / ROWHOUSE_META_REL)
    pal_keys = 0
    if meta:
        for tile in meta.get("tiles") or []:
            key = str(tile.get("variant_key") or "")
            if "__pal_" in key:
                pal_keys += 1
    checks["rowhouse_atlas_pal_keys"] = pal_keys >= 4

    rust_wit = _load_json(root / "debug_runs/city_g2_c5_001_live.json")
    checks["rust_witness_file"] = rust_wit is not None
    checks["rust_witness_green"] = bool(rust_wit and rust_wit.get("green") is True)

    green = all(checks.values())
    return {
        "task_id": TASK_ID,
        "green": green,
        "checks": checks,
        "pilot_module_count": len(PILOT_MODULE_IDS),
        "module_index_palette_rows": palette_field_rows,
        "rowhouse_pal_key_count": pal_keys,
    }


def write_city_g2_c5_mcp_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    body = audit_city_g2_c5_mcp(repo=root)
    body["_agent_meta"] = {
        "schema": "city_g2_c5_mcp_live_v1",
        "written_at_epoch_secs": int(time.time()),
        "profile": "CITY_G2_C5_MCP",
        "source_system": "city_palette_g2",
        "relative_path": WITNESS_REL,
        "ritual": f"BLANG:WIT-HON {TASK_ID}-MCP" if body.get("green") else None,
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = WITNESS_REL
    return body


def apply_city_g2_c5_index_and_atlas(*, repo: Path | None = None) -> dict[str, Any]:
    """Regenerate module index (palette fields) + wire rowhouse atlas palette keys."""
    from rust_engine_mcp.library import write_module_index

    index = write_module_index()
    atlas = wire_rowhouse_atlas_palette_keys(repo=repo)
    return {"index_entries": index.get("entry_count"), "atlas": atlas}
