"""ARCH-BUILD-GRAMMAR-003 — Python mirror of building grammar evaluator (T3 compiler)."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

from .paths import repo_root, schemas_dir
from .schemas import load_json_file, validate_building_grammar

GRAMMARS_DIR = "assets/configs/buildings/grammars"
GRAMMAR_RULES_VERSION = "building_grammar_v1"


def _repo_grammars_dir() -> Path:
    return repo_root() / GRAMMARS_DIR


def _mix_seed(seed: int, salt: str) -> int:
    raw = f"{seed}:{salt}".encode()
    return int(hashlib.sha256(raw).hexdigest()[:16], 16)


def _pick_weighted(items: list[tuple[int, int]], seed: int) -> int:
    total = sum(w for w, _ in items)
    if total <= 0:
        return 0
    roll = seed % total
    acc = 0
    for weight, idx in items:
        acc += weight
        if roll < acc:
            return idx
    return items[-1][1]


def load_building_grammar_json(path: str | Path) -> dict[str, Any]:
    data = load_json_file(Path(path))
    validate_building_grammar(data)
    return data


def load_building_grammar_by_archetype(archetype_id: str) -> dict[str, Any]:
    root = _repo_grammars_dir()
    if not root.is_dir():
        raise FileNotFoundError(f"grammars dir missing: {root}")
    for path in sorted(root.glob("*.ron")):
        grammar = _load_grammar_ron(path)
        if str(grammar.get("archetype", {}).get("id")) == archetype_id:
            return grammar
    raise KeyError(f"no grammar for archetype: {archetype_id}")


def _load_grammar_ron(path: Path) -> dict[str, Any]:
    """Minimal RON → dict for pilot grammars (no full RON parser dependency)."""
    example = schemas_dir() / "examples" / "building_grammar_industrial_warehouse_v1.json"
    if path.stem == "industrial_warehouse_v1" and example.is_file():
        return load_building_grammar_json(example)
    raise NotImplementedError(f"RON grammar load not implemented for {path.name}; add JSON mirror")


def generate(archetype_id: str, district_style: str, seed: int) -> dict[str, Any]:
    """generate(archetype, district_style, seed) — parity with Rust building_grammar.rs."""
    grammar = load_building_grammar_by_archetype(archetype_id)
    district = None
    for row in grammar.get("district_styles") or []:
        if str(row.get("id")) == district_style:
            district = row
            break
    if district is None:
        raise KeyError(f"unknown district_style: {district_style}")

    strategies = grammar["massing"]["strategies"]
    massing_weights = [(int(s["weight"]), i) for i, s in enumerate(strategies)]
    mi = _pick_weighted(massing_weights, _mix_seed(seed, "massing"))
    strategy = strategies[mi]

    bounds = grammar["archetype"]["footprint_bounds"]
    width, depth, floors = _resolve_footprint(bounds, strategy, seed)

    age_bands = grammar["age"]["bands"]
    age_weights = [(int(b["weight"]), i) for i, b in enumerate(age_bands)]
    ai = _pick_weighted(age_weights, _mix_seed(seed, "age"))
    age_band = age_bands[ai]

    slot_overrides: dict[str, str] = {}
    roof_default = grammar["roof"]["default_slot"]
    for row in grammar["roof"].get("by_massing") or []:
        if str(row.get("massing_id")) == strategy["id"]:
            roof_default = str(row["slot"])
            break
    slot_overrides["roof_default"] = roof_default

    facade = grammar.get("facade") or {}
    if facade.get("wall_slot"):
        slot_overrides["wall_1u"] = str(facade["wall_slot"])
    if facade.get("door_slot"):
        slot_overrides["door_default"] = str(facade["door_slot"])
    if facade.get("window_slot"):
        slot_overrides["window_1u"] = str(facade["window_slot"])

    rule_chain = [
        {
            "layer": "archetype",
            "rule_id": grammar["archetype"]["id"],
            "detail": f"usage={grammar['archetype']['usage']}",
        },
        {
            "layer": "district_style",
            "rule_id": district_style,
            "detail": f"style_pack={district['style_pack_id']}",
        },
        {
            "layer": "massing",
            "rule_id": strategy["id"],
            "detail": f"{width}x{depth}x{floors} mode={strategy.get('footprint_mode', 'rect')}",
        },
        {
            "layer": "roof",
            "rule_id": roof_default,
            "detail": "slot override for R token",
        },
        {
            "layer": "facade",
            "rule_id": "facade_v1",
            "detail": f"tags={','.join(facade.get('placement_tags') or [])}",
        },
        {
            "layer": "detail",
            "rule_id": str((grammar.get("detail") or {}).get("prop_slot") or "none"),
            "detail": f"density={(grammar.get('detail') or {}).get('density', 0)}",
        },
        {
            "layer": "age",
            "rule_id": age_band["id"],
            "detail": f"variant_tags={','.join(age_band.get('variant_tags') or [])}",
        },
    ]

    footprint_mode = str(strategy.get("footprint_mode") or "rect")
    if footprint_mode == "l_shape":
        rule_chain.append(
            {
                "layer": "massing",
                "rule_id": "l_shape_v1",
                "detail": "asymmetric rect footprint (full L cutout in v2)",
            }
        )

    return {
        "grammar_id": grammar["grammar_id"],
        "archetype_id": archetype_id,
        "district_style": district_style,
        "seed": seed,
        "massing_strategy": strategy["id"],
        "footprint_mode": footprint_mode,
        "width": width,
        "depth": depth,
        "floors": floors,
        "style_pack_id": str(district["style_pack_id"]),
        "slot_overrides": slot_overrides,
        "placement_tags": list(facade.get("placement_tags") or []),
        "variant_tags": list(age_band.get("variant_tags") or ["clean"]),
        "detail_density": float((grammar.get("detail") or {}).get("density") or 0.0),
        "age_band": str(age_band["id"]),
        "rule_chain": rule_chain,
        "material_profiles": dict(district.get("material_profiles") or {}),
        "weathering": weathering_for_age_band(str(age_band["id"])),
    }


def _resolve_footprint(
    bounds: dict[str, Any], strategy: dict[str, Any], seed: int
) -> tuple[int, int, int]:
    s = _mix_seed(seed, f"footprint:{strategy['id']}")
    min_w = int(bounds["min_width"])
    max_w = int(bounds["max_width"])
    min_d = int(bounds["min_depth"])
    max_d = int(bounds["max_depth"])
    min_f = int(bounds["min_floors"])
    max_f = int(bounds["max_floors"])

    depth_span = max_d - min_d + 1
    depth = min_d + (s % depth_span)

    sid = str(strategy["id"])
    if sid in ("long_hall", "double_hall"):
        ratio = float(strategy.get("width_depth_ratio") or 1.5)
        width = max(2, min(max_w, round(depth * ratio)))
    elif sid == "l_shape":
        width = max(min_w, min(max_w, depth + 2))
    else:
        width_span = max_w - min_w + 1
        width = min_w + ((s >> 16) % width_span)

    floor_span = max_f - min_f + 1
    floors = min_f + ((s >> 32) % floor_span)
    return max(2, width), max(2, depth), max(1, floors)


def footprint_grid_from_grammar(result: dict[str, Any]) -> list[dict[str, Any]]:
    """Grammar-aware footprint cells (mirrors Rust FootprintGrid::from_grammar)."""
    from .assembly import footprint_grid

    cells = footprint_grid(
        int(result["width"]),
        int(result["depth"]),
        int(result["floors"]),
    )
    mode = str(result.get("footprint_mode") or "rect")
    width = int(result["width"])
    depth = int(result["depth"])
    floors = int(result["floors"])

    if mode == "yard_interior" and width >= 4 and depth >= 4:
        for floor in range(floors):
            for y in range(1, depth - 1):
                for x in range(1, width - 1):
                    if x == 0 or y == 0 or x + 1 == width or y + 1 == depth:
                        continue
                    cells.append({"x": x, "y": y, "floor": floor, "token": "Y"})

    if mode == "l_shape" and width >= 4 and depth >= 4:
        cut_x = (width * 2) // 3
        cut_y = (depth * 2) // 3
        for floor in range(floors):
            for y in range(cut_y, depth - 1):
                for x in range(cut_x, width - 1):
                    if x == 0 or y == 0 or x + 1 == width or y + 1 == depth:
                        continue
                    cells.append({"x": x, "y": y, "floor": floor, "token": "Y"})

    return cells


def grammar_reference_tags(result: dict[str, Any]) -> list[str]:
    tags = [
        f"grammar:{result['grammar_id']}",
        f"archetype:{result['archetype_id']}",
        f"district:{result['district_style']}",
        f"massing:{result['massing_strategy']}",
        f"age:{result['age_band']}",
        GRAMMAR_RULES_VERSION,
    ]
    for step in result.get("rule_chain") or []:
        layer = step.get("layer")
        rule_id = step.get("rule_id")
        if layer and rule_id:
            tags.append(f"chain:{layer}:{rule_id}")
    return tags


def weathering_for_age_band(age_band: str) -> str:
    return {"new": "light", "weathered": "medium", "abandoned": "heavy"}.get(age_band, "medium")


def default_material_for_slot(slot_key: str, style_pack_id: str) -> str | None:
    if style_pack_id == "style_industrial_west":
        return {
            "wall_1u": "steel_panel_01",
            "wall_2u": "steel_panel_01",
            "door_default": "steel_door_warehouse_01",
            "door_wide": "steel_door_warehouse_01",
            "corner_outer": "steel_corner_01",
            "corner_inner": "steel_corner_01",
            "roof_default": "roof_metal_01",
            "roof_industrial": "roof_metal_01",
            "roof_flat": "roof_metal_01",
            "window_industrial": "glass_panel_01",
            "window_1u": "glass_panel_01",
        }.get(slot_key)
    return {
        "wall_1u": "brick_red_01",
        "door_default": "wood_plank_01",
        "corner_outer": "brick_red_01",
        "roof_default": "roof_tile_01",
    }.get(slot_key)


def material_profile_for_slot(result: dict[str, Any], slot_key: str) -> str | None:
    profiles = dict(result.get("material_profiles") or {})
    if slot_key in profiles:
        return str(profiles[slot_key])
    return default_material_for_slot(slot_key, str(result.get("style_pack_id") or ""))


def list_archetype_ids() -> list[str]:
    root = _repo_grammars_dir()
    if not root.is_dir():
        return []
    out: list[str] = []
    for path in sorted(root.glob("*.ron")):
        try:
            grammar = _load_grammar_ron(path)
        except (NotImplementedError, OSError, KeyError, ValueError):
            continue
        aid = str((grammar.get("archetype") or {}).get("id") or "")
        if aid and aid not in out:
            out.append(aid)
    return out


def list_district_styles(archetype_id: str) -> list[str]:
    grammar = load_building_grammar_by_archetype(archetype_id)
    return [
        str(row.get("id"))
        for row in grammar.get("district_styles") or []
        if row.get("id")
    ]


def grammar_rule_chain_snapshot(result: dict[str, Any]) -> dict[str, str]:
    """Flatten rule_chain list → assembly_snapshot_v1.grammar_rule_chain object."""
    chain: dict[str, str] = {
        "footprint_mode": str(result.get("footprint_mode") or "rect"),
    }
    for step in result.get("rule_chain") or []:
        layer = str(step.get("layer") or "")
        rule_id = str(step.get("rule_id") or "")
        if not layer or not rule_id:
            continue
        if layer == "archetype" and "archetype" not in chain:
            chain["archetype"] = rule_id
        elif layer == "massing" and "massing" not in chain:
            chain["massing"] = rule_id
        elif layer == "roof" and "roof" not in chain:
            chain["roof"] = rule_id
        elif layer == "facade" and "facade" not in chain:
            chain["facade"] = rule_id
        elif layer == "detail" and "detail" not in chain:
            chain["detail"] = rule_id
        elif layer == "age" and "age" not in chain:
            chain["age"] = rule_id
    return chain


def generate_with_overrides(
    archetype_id: str,
    district_style: str,
    seed: int,
    *,
    massing_strategy: str | None = None,
    footprint: dict[str, Any] | None = None,
    footprint_mode: str | None = None,
    age_band_id: str | None = None,
    roof_slot: str | None = None,
    roof_rule_id: str | None = None,
    wall_slot: str | None = None,
    door_slot: str | None = None,
    window_slot: str | None = None,
    facade_rule_id: str | None = None,
) -> dict[str, Any]:
    """Deterministic grammar eval with explicit pins (GRAMMAR-ITER-001 massing path)."""
    grammar = load_building_grammar_by_archetype(archetype_id)
    district = None
    for row in grammar.get("district_styles") or []:
        if str(row.get("id")) == district_style:
            district = row
            break
    if district is None:
        raise KeyError(f"unknown district_style: {district_style}")

    strategies = grammar["massing"]["strategies"]
    if massing_strategy:
        strategy = next((s for s in strategies if str(s.get("id")) == massing_strategy), None)
        if strategy is None:
            raise KeyError(f"unknown massing_strategy: {massing_strategy}")
    else:
        massing_weights = [(int(s["weight"]), i) for i, s in enumerate(strategies)]
        mi = _pick_weighted(massing_weights, _mix_seed(seed, "massing"))
        strategy = strategies[mi]

    bounds = grammar["archetype"]["footprint_bounds"]
    if footprint:
        width = max(2, int(footprint["width"]))
        depth = max(2, int(footprint["depth"]))
        floors = max(1, int(footprint.get("floors") or bounds["min_floors"]))
    else:
        width, depth, floors = _resolve_footprint(bounds, strategy, seed)

    resolved_footprint_mode = str(footprint_mode or strategy.get("footprint_mode") or "rect")

    age_bands = grammar["age"]["bands"]
    if age_band_id:
        age_band = next((b for b in age_bands if str(b.get("id")) == age_band_id), None)
        if age_band is None:
            raise KeyError(f"unknown age_band_id: {age_band_id}")
    else:
        age_weights = [(int(b["weight"]), i) for i, b in enumerate(age_bands)]
        ai = _pick_weighted(age_weights, _mix_seed(seed, "age"))
        age_band = age_bands[ai]

    slot_overrides: dict[str, str] = {}
    roof_default = grammar["roof"]["default_slot"]
    for row in grammar["roof"].get("by_massing") or []:
        if str(row.get("massing_id")) == strategy["id"]:
            roof_default = str(row["slot"])
            break
    if roof_slot:
        roof_default = roof_slot
    elif roof_rule_id:
        roof_default = roof_rule_id
    slot_overrides["roof_default"] = roof_default

    facade = grammar.get("facade") or {}
    if facade.get("wall_slot"):
        slot_overrides["wall_1u"] = str(facade["wall_slot"])
    if facade.get("door_slot"):
        slot_overrides["door_default"] = str(facade["door_slot"])
    if facade.get("window_slot"):
        slot_overrides["window_1u"] = str(facade["window_slot"])
    if wall_slot:
        slot_overrides["wall_1u"] = wall_slot
    if door_slot:
        slot_overrides["door_default"] = door_slot
    if window_slot:
        slot_overrides["window_1u"] = window_slot

    facade_rule = facade_rule_id or "facade_v1"

    rule_chain = [
        {
            "layer": "archetype",
            "rule_id": grammar["archetype"]["id"],
            "detail": f"usage={grammar['archetype']['usage']}",
        },
        {
            "layer": "district_style",
            "rule_id": district_style,
            "detail": f"style_pack={district['style_pack_id']}",
        },
        {
            "layer": "massing",
            "rule_id": strategy["id"],
            "detail": f"{width}x{depth}x{floors} mode={resolved_footprint_mode}",
        },
        {
            "layer": "roof",
            "rule_id": roof_default,
            "detail": "slot override for R token",
        },
        {
            "layer": "facade",
            "rule_id": facade_rule,
            "detail": f"tags={','.join(facade.get('placement_tags') or [])}",
        },
        {
            "layer": "detail",
            "rule_id": str((grammar.get("detail") or {}).get("prop_slot") or "none"),
            "detail": f"density={(grammar.get('detail') or {}).get('density', 0)}",
        },
        {
            "layer": "age",
            "rule_id": age_band["id"],
            "detail": f"variant_tags={','.join(age_band.get('variant_tags') or [])}",
        },
    ]

    if resolved_footprint_mode == "l_shape":
        rule_chain.append(
            {
                "layer": "massing",
                "rule_id": "l_shape_v1",
                "detail": "asymmetric rect footprint (full L cutout in v2)",
            }
        )

    return {
        "grammar_id": grammar["grammar_id"],
        "archetype_id": archetype_id,
        "district_style": district_style,
        "seed": seed,
        "massing_strategy": strategy["id"],
        "footprint_mode": resolved_footprint_mode,
        "width": width,
        "depth": depth,
        "floors": floors,
        "style_pack_id": str(district["style_pack_id"]),
        "slot_overrides": slot_overrides,
        "placement_tags": list(facade.get("placement_tags") or []),
        "variant_tags": list(age_band.get("variant_tags") or ["clean"]),
        "detail_density": float((grammar.get("detail") or {}).get("density") or 0.0),
        "age_band": str(age_band["id"]),
        "rule_chain": rule_chain,
        "material_profiles": dict(district.get("material_profiles") or {}),
        "weathering": weathering_for_age_band(str(age_band["id"])),
    }
