"""DES-CITY-PALETTE-VARIATION-001 — city palette variation charter sign-off witness."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

WITNESS_REL = "debug_runs/city_palette_variation_charter_live.json"
GATE_ID = "DES-CITY-PALETTE-VARIATION-001"
CHARTER_DOC = "src/dev/design_city_palette_variation_v1.md"
G1_C3_WITNESS_REL = "debug_runs/city_g1_c3_001_live.json"
G1_C3_QUEUE_CLOSED = "CITY-G1-C3-001"

PALETTE_RON: tuple[tuple[str, str], ...] = (
    ("palette_industrial_west_v1", "assets/configs/buildings/palettes/industrial_west_v1.ron"),
    ("palette_colonial_res_v1", "assets/configs/buildings/palettes/colonial_res_v1.ron"),
    ("palette_rowhouse_urban_v1", "assets/configs/buildings/palettes/rowhouse_urban_v1.ron"),
)

PALETTE_JSON: tuple[tuple[str, str], ...] = (
    ("palette_industrial_west_v1", "tools/mcp/schemas/examples/palette_industrial_west_v1.example.json"),
    ("palette_colonial_res_v1", "tools/mcp/schemas/examples/palette_colonial_res_v1.example.json"),
    ("palette_rowhouse_urban_v1", "tools/mcp/schemas/examples/palette_rowhouse_urban_v1.example.json"),
)

V1_STYLE_PACKS = frozenset({"style_industrial_west", "style_colonial", "style_victorian"})
SLOT_KEYS = ("wall_primary", "trim", "roof")


def _load_json(path: Path) -> dict[str, Any] | None:
    if not path.is_file():
        return None
    return json.loads(path.read_text(encoding="utf-8"))


def _palette_checks(root: Path, palette_id: str, rel: str) -> dict[str, bool]:
    path = root / rel
    body = _load_json(path) if rel.endswith(".json") else None
    if body is None and path.is_file():
        text = path.read_text(encoding="utf-8")
        teaches_ok = "teaches:" in text and "palette_variation" in text
        style_ok = any(sp in text for sp in V1_STYLE_PACKS)
        variations_ok = text.count("variation_id:") >= 2 or text.count('"variation_id"') >= 2
        slots_ok = all(key in text for key in SLOT_KEYS)
        return {
            f"{palette_id}_file": True,
            f"{palette_id}_schema_v1": "palette_catalog_v1" in text,
            f"{palette_id}_teaches": teaches_ok,
            f"{palette_id}_style_pack": style_ok,
            f"{palette_id}_variations_min2": variations_ok,
            f"{palette_id}_material_slots": slots_ok,
        }
    if body is None:
        return {f"{palette_id}_file": False}
    variations = body.get("variations") or []
    teaches = (body.get("_meta") or {}).get("teaches") or []
    style_pack = str(body.get("style_pack") or "")
    slot_ok = all(
        any(key in (v.get("material_slots") or {}) for key in SLOT_KEYS)
        for v in variations
        if v.get("material_slots")
    )
    return {
        f"{palette_id}_file": True,
        f"{palette_id}_schema_v1": body.get("schema") == "palette_catalog_v1",
        f"{palette_id}_id_match": body.get("palette_id") == palette_id,
        f"{palette_id}_teaches_min2": len(teaches) >= 2,
        f"{palette_id}_variations_min2": len(variations) >= 2,
        f"{palette_id}_style_pack": style_pack in V1_STYLE_PACKS,
        f"{palette_id}_material_slots": slot_ok,
    }


def _g1_c3_dependency_met(root: Path) -> bool:
    wit = root / G1_C3_WITNESS_REL
    if wit.is_file():
        data = json.loads(wit.read_text(encoding="utf-8"))
        return data.get("green") is True
    queue = root / "tools/orchestrator/queues/city_grammar_queue.json"
    if not queue.is_file():
        return False
    body = json.loads(queue.read_text(encoding="utf-8"))
    for row in body.get("drain") or []:
        if row.get("id") == G1_C3_QUEUE_CLOSED:
            return row.get("status") == "done"
    return False


def run_city_palette_variation_charter_audit(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    charter = (root / CHARTER_DOC).read_text(encoding="utf-8") if (root / CHARTER_DOC).is_file() else ""
    checks: dict[str, bool] = {
        "charter_doc": (root / CHARTER_DOC).is_file(),
        "g1_c3_dependency": _g1_c3_dependency_met(root),
        "vocabulary_documented": "palette_catalog_v1" in charter and "palette_family" in charter,
        "seed_resolution_documented": "lot_seed" in charter and "variation_id" in charter,
        "three_palette_charters": all(pid in charter for pid, _ in PALETTE_RON),
        "module_index_extension": "palette_family" in charter,
        "tile_atlas_extension": "__pal_" in charter or "palette_column" in charter,
        "presentation_only": "Presentation only" in charter or "presentation only" in charter.lower(),
        "block_recipe_handoff": "district_style" in charter,
    }
    for palette_id, rel in PALETTE_RON:
        checks.update(_palette_checks(root, palette_id, rel))
    for palette_id, rel in PALETTE_JSON:
        checks.update(_palette_checks(root, palette_id, rel))

    green = all(checks.values())
    return {
        "gate": GATE_ID,
        "issue": "CITY-C5",
        "program": "PLAN-CITY-GRAMMAR-v1",
        "charter_doc": CHARTER_DOC,
        "palette_ron_paths": [rel for _, rel in PALETTE_RON],
        "palette_json_examples": [rel for _, rel in PALETTE_JSON],
        "checks": checks,
        "acceptance": {
            "C1_vocabulary": checks.get("vocabulary_documented"),
            "C2_three_charters": checks.get("three_palette_charters"),
            "C3_ron_on_disk": all(checks.get(f"{pid}_file") for pid, _ in PALETTE_RON),
            "C4_seed_resolution": checks.get("seed_resolution_documented"),
            "C5_style_packs": all(checks.get(f"{pid}_style_pack") for pid, _ in PALETTE_RON),
            "C6_presentation_only": checks.get("presentation_only"),
            "C8_critique_no_rust_colormap": "Data not code" in charter or "data not code" in charter.lower(),
        },
        "green": green,
        "verdict": "PASS" if green else "FAIL",
        "handoff": {
            "coder_mcp": "palette_catalog_v1.schema.json",
            "coder": "CITY-G2-C5-001 palette resolver in module_index + tile_atlas_index",
            "unblocks": ["CITY-G2-C5-001"],
        },
    }


def refresh_city_palette_variation_charter_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    body = run_city_palette_variation_charter_audit(repo=root)
    body["_agent_meta"] = {
        "schema": "city_palette_variation_charter_live_v1",
        "written_at_epoch_secs": int(time.time()),
        "profile": "DES_CITY_PALETTE_VARIATION",
        "source_system": "dmcp_city_palette_variation",
        "relative_path": WITNESS_REL,
        "ritual": f"BLANG:WIT-HON→Q✓ {GATE_ID}" if body.get("green") else "BLANG:WIT-HON FAIL",
        "agent": "designer-mcp",
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = WITNESS_REL
    return body
