"""Expand designer variant_matrix YAML → tile_batch variants + variant_set rows (PT-3)."""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

# Canonical bake params per variant_key (deterministic Blender tile_variant_bake job).
VARIANT_BAKE: dict[str, dict[str, Any]] = {
    "clean_day": {
        "state": "clean",
        "damage": 0.0,
        "power": "off",
        "fill": "empty",
        "lighting": "day",
    },
    "clean_night_off": {
        "state": "clean",
        "damage": 0.0,
        "power": "off",
        "fill": "empty",
        "lighting": "night_off",
    },
    "clean_night_on": {
        "state": "clean",
        "damage": 0.0,
        "power": "on",
        "fill": "empty",
        "lighting": "night_on",
    },
    "damaged_day": {
        "state": "damaged",
        "damage": 0.45,
        "power": "off",
        "fill": "half",
        "lighting": "day",
    },
    "damaged_night_on": {
        "state": "damaged",
        "damage": 0.45,
        "power": "on",
        "fill": "half",
        "lighting": "night_on",
    },
    "under_construction_01": {
        "state": "dirty",
        "damage": 0.0,
        "power": "off",
        "fill": "empty",
        "lighting": "day",
    },
    "under_construction_02": {
        "state": "dirty",
        "damage": 0.1,
        "power": "partial",
        "fill": "quarter",
        "lighting": "day",
    },
    "under_construction_03": {
        "state": "dirty",
        "damage": 0.0,
        "power": "on",
        "fill": "half",
        "lighting": "day",
    },
    "abandoned": {
        "state": "ruined",
        "damage": 0.8,
        "power": "off",
        "fill": "empty",
        "lighting": "day",
    },
    "ruined": {
        "state": "ruined",
        "damage": 1.0,
        "power": "off",
        "fill": "empty",
        "lighting": "day",
    },
}

for _i in range(8):
    VARIANT_BAKE[f"burning_{_i:02d}"] = {
        "state": "damaged",
        "damage": 0.55 + _i * 0.02,
        "power": "on",
        "fill": "half",
        "lighting": "night_on",
        "fire_frame": _i,
    }

SIM_TAGS: dict[str, list[str]] = {
    "clean_day": ["sim_operational", "sim_day", "sim_power_any"],
    "clean_night_off": ["sim_operational", "sim_night", "sim_power_off"],
    "clean_night_on": ["sim_operational", "sim_night", "sim_power_on"],
    "damaged_day": ["sim_damage_mid", "sim_day"],
    "damaged_night_on": ["sim_damage_mid", "sim_night", "sim_power_on"],
    "under_construction_01": ["sim_construction_early"],
    "under_construction_02": ["sim_construction_mid"],
    "under_construction_03": ["sim_construction_late"],
    "abandoned": ["sim_abandoned"],
    "ruined": ["sim_ruined"],
}
for _i in range(8):
    SIM_TAGS[f"burning_{_i:02d}"] = ["sim_fire", f"sim_fire_frame_{_i}"]


def _parse_inline_bool(block: str, key: str) -> bool | None:
    m = re.search(rf"{key}:\s*(true|false)", block, re.I)
    if not m:
        return None
    return m.group(1).lower() == "true"


def load_variant_matrix(path: str | Path) -> dict[str, Any]:
    """Minimal YAML reader for variant_matrix_* v1 files (no PyYAML dep)."""
    p = Path(path)
    if not p.is_absolute():
        p = repo_root() / p
    text = p.read_text(encoding="utf-8")
    out: dict[str, Any] = {
        "path": str(p),
        "archetype": "",
        "primary_style_pack": "",
        "pilot": {},
        "variant_keys": {},
    }
    m = re.search(r"^archetype:\s*(\S+)", text, re.M)
    if m:
        out["archetype"] = m.group(1)
    m = re.search(r"^primary_style_pack:\s*(\S+)", text, re.M)
    if m:
        out["primary_style_pack"] = m.group(1)

    pilot_block = re.search(r"^pilot:\n((?:  .+\n)+)", text, re.M)
    if pilot_block:
        pb = pilot_block.group(1)
        for key in (
            "lod0_batch_id",
            "production_batch_id",
            "tile_id",
            "assembly_id_lod0",
        ):
            km = re.search(rf"^\s{{2}}{key}:\s*(\S+)", pb, re.M)
            if km:
                out["pilot"][key] = km.group(1)
        fp = re.search(
            r"footprint:\s*\{\s*width:\s*(\d+),\s*depth:\s*(\d+),\s*floors:\s*(\d+)\s*\}",
            pb,
        )
        if fp:
            out["pilot"]["footprint"] = {
                "width": int(fp.group(1)),
                "depth": int(fp.group(2)),
                "floors": int(fp.group(3)),
            }

    vk = re.search(r"^variant_keys:\n((?:(?:  .+\n)|(?:    .+\n))+)", text, re.M)
    if vk:
        block = vk.group(1)
        for km in re.finditer(r"^  ([a-z0-9_]+):\s*(?:\n|(\{[^\n]+\}))", block, re.M):
            key = km.group(1)
            inline = km.group(2) or ""
            start = km.end()
            next_m = re.search(r"^  [a-z0-9_]+:", block[start:], re.M)
            section = block[km.start() :]
            if next_m:
                section = block[km.start() : km.start() + next_m.start()]
            required = _parse_inline_bool(inline or section, "required")
            out["variant_keys"][key] = {"required": bool(required) if required is not None else False}

    return out


def load_ship_minimum_keys() -> list[str]:
    ron = repo_root() / "assets/configs/buildings/_variant_catalog.ron"
    if not ron.is_file():
        return [
            "clean_day",
            "clean_night_on",
            "damaged_night_on",
            "under_construction_02",
            "abandoned",
            "burning_00",
        ]
    text = ron.read_text(encoding="utf-8")
    keys = re.findall(r'"([a-z0-9_]+)"', text.split("ship_minimum_keys:")[-1].split("),")[0])
    return keys or [
        "clean_day",
        "clean_night_on",
        "damaged_night_on",
        "under_construction_02",
        "abandoned",
        "burning_00",
    ]


def expanded_variant_keys(
    matrix: dict[str, Any],
    *,
    include_fire_row: bool = True,
    minimum_only: bool = False,
) -> list[str]:
    if minimum_only:
        keys = load_ship_minimum_keys()
    else:
        keys = [k for k, v in matrix.get("variant_keys", {}).items() if v.get("required")]
        for min_key in load_ship_minimum_keys():
            if min_key in matrix.get("variant_keys", {}) and min_key not in keys:
                keys.append(min_key)
        if include_fire_row:
            for i in range(8):
                fk = f"burning_{i:02d}"
                if fk in matrix.get("variant_keys", {}) and fk not in keys:
                    keys.append(fk)
    # Stable catalog order
    catalog_order = load_ship_minimum_keys() + [f"burning_{i:02d}" for i in range(1, 8)]
    all_known = list(VARIANT_BAKE.keys())
    ordered = [k for k in all_known if k in keys]
    for k in keys:
        if k not in ordered:
            ordered.append(k)
    for k in catalog_order:
        if k in keys and k not in ordered:
            ordered.append(k)
    return ordered


def variant_row_for_key(key: str) -> dict[str, Any]:
    base = dict(VARIANT_BAKE.get(key) or VARIANT_BAKE["clean_day"])
    row = {"variant_key": key, **base}
    return row


def variant_set_rows(keys: list[str]) -> list[dict[str, Any]]:
    rows = []
    for key in keys:
        bake = VARIANT_BAKE.get(key, VARIANT_BAKE["clean_day"])
        tags = list(SIM_TAGS.get(key, []))
        rows.append(
            {
                "variant_key": key,
                "sim_tags": tags,
                "tags": tags,
                "layers": {
                    "lighting": {
                        "lighting": bake.get("lighting", "day"),
                        "power": bake.get("power", "off"),
                        "night_lights": bake.get("lighting") == "night_on",
                    },
                    "damage": {
                        "state": bake.get("state", "clean"),
                        "damage": bake.get("damage", 0.0),
                    },
                    "fill": {"fill": bake.get("fill", "empty")},
                },
            }
        )
    return rows


def atlas_layout(variant_count: int) -> tuple[int, int]:
    if variant_count <= 2:
        return 2, 1
    if variant_count <= 6:
        return 3, 2
    if variant_count <= 8:
        return 4, 2
    if variant_count <= 12:
        return 4, 3
    return 4, max(4, (variant_count + 3) // 4)


def expand_matrix_to_tile_batch(
    matrix_path: str | Path,
    *,
    pilot_slug: str,
    style_pack_id: str,
    assembly_snapshot_rel: str,
    base: str,
    seed: int,
    minimum_only: bool = False,
    include_fire_row: bool = True,
) -> dict[str, Any]:
    matrix = load_variant_matrix(matrix_path)
    pilot = matrix.get("pilot") or {}
    batch_id = str(pilot.get("production_batch_id") or f"tile_{pilot_slug}_production_v1")
    atlas_id = batch_id.replace("tile_", "").replace("_production_v1", "_production_v1")
    if not atlas_id.endswith("_production_v1"):
        atlas_id = f"{pilot_slug}_production_v1"
    tile_id = str(pilot.get("tile_id") or pilot_slug)
    footprint = pilot.get("footprint") or {"width": 4, "depth": 3, "floors": 2}
    keys = expanded_variant_keys(
        matrix, include_fire_row=include_fire_row, minimum_only=minimum_only
    )
    cols, rows = atlas_layout(len(keys))
    variant_set_id = f"{pilot_slug}_production_v1"
    variant_set_rel = f"tools/mcp/schemas/examples/variant_set_{pilot_slug}_production_v1.json"
    return {
        "schema_version": 1,
        "batch_id": batch_id,
        "tile_id": tile_id,
        "base": base,
        "status": "production",
        "ship": True,
        "bake_source": "keyframe_pack",
        "source_tier": "production",
        "development_tier": "production",
        "keyframe_rename_pk": True,
        "matrix_ref": str(Path(matrix_path).as_posix()),
        "rules_applied": [
            "no_ai_generated_images",
            "deterministic_output",
            "batch_processing",
            "grid_alignment",
        ],
        "render": {
            "method": "blender_keyframe_light_rig",
            "isometric": True,
            "seed": seed,
            "tile_size_px": 128,
            "light_blend": "utils/Tile_iso_rig_v1.blend",
        },
        "assembly_ref": {
            "style_pack_id": style_pack_id,
            "assembly_snapshot": assembly_snapshot_rel,
            "footprint": footprint,
        },
        "variant_set_ref": variant_set_rel,
        "variants": [variant_row_for_key(k) for k in keys],
        "atlas": {
            "atlas_id": atlas_id,
            "columns": cols,
            "rows": rows,
            "tile_px": 128,
            "padding_px": 2,
            "output_png": f"assets/archive/greybox_tile_production_v1_frozen_2026-06/atlases/{atlas_id}_atlas.png",
            "meta_json": f"assets/staging/tiles/{batch_id}/atlas_meta.json",
        },
        "expected_outputs": ["{variant_key}.png", "atlas_meta.json", f"{atlas_id}_atlas.png"],
        "note": f"PT-2 production bake — {style_pack_id} {footprint['width']}x{footprint['depth']}x{footprint['floors']}",
    }


def variant_matrix_expand(
    matrix_path: str | Path,
    *,
    minimum_only: bool = False,
    include_fire_row: bool = True,
    write_batch: bool = True,
) -> dict[str, Any]:
    """Expand matrix → variant keys, optional production tile_batch + variant_set JSON."""
    matrix = load_variant_matrix(matrix_path)
    keys = expanded_variant_keys(
        matrix, include_fire_row=include_fire_row, minimum_only=minimum_only
    )
    result: dict[str, Any] = {
        "ok": True,
        "matrix_path": str(Path(matrix_path).resolve()),
        "archetype": matrix.get("archetype"),
        "variant_keys": keys,
        "variant_count": len(keys),
        "sim_tags_by_key": {k: list(SIM_TAGS.get(k, [])) for k in keys},
    }
    if not write_batch:
        return result
    pilot = matrix.get("pilot") or {}
    slug = str(pilot.get("tile_id") or matrix.get("archetype") or "pilot")
    style = str(matrix.get("primary_style_pack") or "style_victorian")
    fp = pilot.get("footprint") or {"width": 4, "depth": 3, "floors": 2}
    batch = expand_matrix_to_tile_batch(
        matrix_path,
        pilot_slug=slug,
        style_pack_id=style,
        assembly_snapshot_rel=f"tools/mcp/schemas/examples/assembly_snapshot_{slug}_v1.json",
        base="stone",
        seed=42,
        minimum_only=minimum_only,
        include_fire_row=include_fire_row,
    )
    result["tile_batch"] = batch
    result["variant_set_rows"] = variant_set_rows(keys)
    return result
