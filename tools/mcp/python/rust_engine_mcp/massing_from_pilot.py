"""Grammar / pilot → massing_preview.json (presentation-only; DR-VOXEL-OCCUPATION).

IO-09 conditioning: never writes FootprintMatrix or sim occupation.
"""

from __future__ import annotations

import math
import re
from pathlib import Path
from typing import Any

from . import building_grammar
from .grammar_build_set import (
    MOCK_SHAPES_RON,
    _read_asset,
    _ron_field,
    _split_ron_entries,
    load_pilot_catalog,
)
from .schemas import validate_massing_preview


def load_mock_shapes() -> dict[str, dict[str, Any]]:
    """Parse `_mock_shapes.ron` → id → {width, depth, cells}."""
    text = _read_asset(MOCK_SHAPES_RON)
    out: dict[str, dict[str, Any]] = {}
    for block in _split_ron_entries(text):
        sid = _ron_field(block, "id")
        if not sid:
            continue
        wm = re.search(r"width:\s*(\d+)", block)
        dm = re.search(r"depth:\s*(\d+)", block)
        cm = re.search(r"cells:\s*\[([^\]]*)\]", block, re.DOTALL)
        if not wm or not dm or not cm:
            continue
        cells_raw = cm.group(1)
        cells = [int(x) for x in re.findall(r"\d+", cells_raw)]
        out[sid] = {
            "id": sid,
            "width": int(wm.group(1)),
            "depth": int(dm.group(1)),
            "cells": cells,
        }
    return out


def _occupied_tiles(shape: dict[str, Any]) -> list[tuple[int, int]]:
    width = int(shape["width"])
    depth = int(shape["depth"])
    cells = list(shape["cells"])
    tiles: list[tuple[int, int]] = []
    for i, v in enumerate(cells):
        if int(v) <= 0:
            continue
        x = i % width
        y = i // width
        if y >= depth:
            break
        tiles.append((x, y))
    return tiles


def _derive_floors(occupied_count: int, *, family_bonus: int = 0, grammar_floors: int | None = None) -> int:
    """Match presentation heuristic: log2 band + optional bonus; clamp 1..4."""
    if grammar_floors is not None:
        return min(4, max(1, int(grammar_floors)))
    n = max(1, int(occupied_count))
    band = 1 + int(math.floor(math.log2(n)))
    return min(4, max(1, band + max(0, int(family_bonus))))


def massing_columns_from_tiles(
    tiles: list[tuple[int, int]],
    *,
    floors: int,
    color_kind: str = "Valid",
    weight: float = 1.0,
    origin: tuple[int, int] = (0, 0),
) -> list[dict[str, Any]]:
    floors_i = min(4, max(1, int(floors)))
    ox, oy = origin
    return [
        {
            "tile": [ox + x, oy + y],
            "floors": floors_i,
            "color_kind": color_kind,
            "weight": float(weight),
        }
        for x, y in tiles
    ]


def build_massing_preview_from_pilot(
    pilot_id: str,
    *,
    seed: int = 1,
    origin: tuple[int, int] = (0, 0),
) -> dict[str, Any]:
    """Export presentation massing from catalog pilot + mock shape (+ grammar lineage when present)."""
    seed_i = max(0, int(seed))
    pilots = {str(p["id"]): p for p in load_pilot_catalog()}
    if pilot_id not in pilots:
        raise KeyError(f"unknown pilot_id: {pilot_id}")
    pilot = pilots[pilot_id]
    mock_id = pilot.get("mock_shape_id")
    if not mock_id:
        raise ValueError(f"pilot {pilot_id} has no mock_shape_id")
    shapes = load_mock_shapes()
    if mock_id not in shapes:
        raise KeyError(f"unknown mock_shape_id: {mock_id}")
    shape = shapes[mock_id]
    tiles = _occupied_tiles(shape)
    if not tiles:
        raise ValueError(f"mock shape {mock_id} has no occupied cells")

    grammar_result: dict[str, Any] | None = None
    chain: dict[str, str] = {}
    material_strategy: str | None = None
    grammar_floors: int | None = None
    arch = pilot.get("grammar_archetype_id")
    district = pilot.get("district_style")
    if arch and district:
        grammar_result = building_grammar.generate(str(arch), str(district), seed_i)
        chain = building_grammar.grammar_rule_chain_snapshot(grammar_result)
        grammar_floors = int(grammar_result.get("floors") or 1)
        material_strategy = str(
            grammar_result.get("massing_strategy")
            or (chain.get("massing") if chain else "")
            or ""
        ) or None

    floors = _derive_floors(len(tiles), grammar_floors=grammar_floors)
    columns = massing_columns_from_tiles(tiles, floors=floors, origin=origin)

    payload: dict[str, Any] = {
        "schema_version": 1,
        "title": f"massing_preview:{pilot_id}",
        "presentation_only": True,
        "sim_occupation": False,
        "seed": seed_i,
        "pilot_id": pilot_id,
        "massing_columns": columns,
        "note": "VOXEL-MASSING-PRES-001 — never commit to occupation",
    }
    if pilot.get("arch_dna_preset"):
        payload["arch_build_grammar_preset_id"] = str(pilot["arch_dna_preset"])
    if chain:
        payload["grammar_rule_chain"] = chain
    if material_strategy:
        payload["material_strategy"] = material_strategy
    validate_massing_preview(payload)
    return payload


def export_massing_preview_from_pilot(
    path: Path,
    pilot_id: str,
    *,
    seed: int = 1,
    origin: tuple[int, int] = (0, 0),
) -> dict[str, Any]:
    """Build + write massing_preview JSON for a catalog pilot."""
    import json

    payload = build_massing_preview_from_pilot(pilot_id, seed=seed, origin=origin)
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    return payload
