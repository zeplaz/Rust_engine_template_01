"""DES-CITY-BLOCK-RECIPE-001 — city block recipe charter sign-off witness."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

WITNESS_REL = "debug_runs/city_block_recipe_charter_live.json"
GATE_ID = "DES-CITY-BLOCK-RECIPE-001"
CHARTER_DOC = "src/dev/design_city_block_recipe_v1.md"
G0_WITNESS_REL = "debug_runs/city_g0_wit_001_live.json"
G0_QUEUE_CLOSED = "CITY-G0-WIT-001"

RECIPE_RON: tuple[tuple[str, str], ...] = (
    ("block_recipe_industrial_yard_v1", "assets/configs/settlement/block_recipes/industrial_yard_v1.ron"),
    ("block_recipe_low_density_res_v1", "assets/configs/settlement/block_recipes/low_density_res_v1.ron"),
    ("block_recipe_medium_density_res_v1", "assets/configs/settlement/block_recipes/medium_density_res_v1.ron"),
)

RECIPE_JSON: tuple[tuple[str, str], ...] = (
    ("block_recipe_industrial_yard_v1", "tools/mcp/schemas/examples/block_recipe_industrial_yard_v1.example.json"),
    ("block_recipe_low_density_res_v1", "tools/mcp/schemas/examples/block_recipe_low_density_res_v1.example.json"),
    ("block_recipe_medium_density_res_v1", "tools/mcp/schemas/examples/block_recipe_medium_density_res_v1.example.json"),
)

GRAMMAR_ARCHETYPES = (
    "assets/configs/buildings/grammars/civic_block_v1.ron",
    "assets/configs/buildings/grammars/factory_cluster_v1.ron",
)

PRIMITIVE_KINDS = ("lot_row", "edge", "scatter", "park_fill", "plaza")
V1_BUILDING_ARCHETYPES = frozenset({"IndustrialWarehouse", "CivicBlock"})


def _load_json(path: Path) -> dict[str, Any] | None:
    if not path.is_file():
        return None
    return json.loads(path.read_text(encoding="utf-8"))


def _recipe_checks(root: Path, recipe_id: str, rel: str) -> dict[str, bool]:
    path = root / rel
    body = _load_json(path) if rel.endswith(".json") else None
    if body is None and path.is_file():
        text = path.read_text(encoding="utf-8")
        body = {"recipe_id": recipe_id, "schema": "block_recipe_v1", "_raw_ron": text}
        teaches_ok = "teaches:" in text and "lot_row" in text
        archetype_ok = any(a in text for a in V1_BUILDING_ARCHETYPES)
        return {
            f"{recipe_id}_file": True,
            f"{recipe_id}_schema_v1": "block_recipe_v1" in text,
            f"{recipe_id}_teaches": teaches_ok,
            f"{recipe_id}_lot_row": "lot_row" in text,
            f"{recipe_id}_catalog_archetype": archetype_ok,
        }
    if body is None:
        return {f"{recipe_id}_file": False}
    steps = body.get("steps") or []
    teaches = (body.get("_meta") or {}).get("teaches") or []
    archetypes = {
        str(step.get("building_archetype"))
        for step in steps
        if step.get("kind") == "lot_row" and step.get("building_archetype")
    }
    sim_flags = [
        step.get("sim_authority")
        for step in steps
        if step.get("kind") == "edge"
    ]
    return {
        f"{recipe_id}_file": True,
        f"{recipe_id}_schema_v1": body.get("schema") == "block_recipe_v1",
        f"{recipe_id}_id_match": body.get("recipe_id") == recipe_id,
        f"{recipe_id}_teaches_min2": len(teaches) >= 2,
        f"{recipe_id}_steps_nonempty": len(steps) >= 2,
        f"{recipe_id}_catalog_archetype": archetypes <= V1_BUILDING_ARCHETYPES and bool(archetypes),
        f"{recipe_id}_edge_sim_authority_false": all(flag is False for flag in sim_flags) if sim_flags else True,
    }


def _g0_dependency_met(root: Path) -> bool:
    wit = root / G0_WITNESS_REL
    if wit.is_file():
        data = json.loads(wit.read_text(encoding="utf-8"))
        return data.get("green") is True
    queue = root / "tools/orchestrator/queues/city_grammar_queue.json"
    if not queue.is_file():
        return False
    body = json.loads(queue.read_text(encoding="utf-8"))
    for row in body.get("drain") or []:
        if row.get("id") == G0_QUEUE_CLOSED:
            return row.get("status") == "done"
    return False


def run_city_block_recipe_charter_audit(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    charter = (root / CHARTER_DOC).read_text(encoding="utf-8") if (root / CHARTER_DOC).is_file() else ""
    checks: dict[str, bool] = {
        "charter_doc": (root / CHARTER_DOC).is_file(),
        "g0_dependency": _g0_dependency_met(root),
        "vocabulary_five_primitives": all(f"`{kind}`" in charter for kind in PRIMITIVE_KINDS),
        "seed_chain_documented": "block_seed" in charter and "lot_seed" in charter,
        "three_recipe_charters": all(rid in charter for rid, _ in RECIPE_RON),
        "civic_block_grammar": (root / GRAMMAR_ARCHETYPES[0]).is_file(),
        "factory_cluster_grammar": (root / GRAMMAR_ARCHETYPES[1]).is_file(),
        "no_rust_layout_shortcut": "no hand-authored" in charter.lower() or "Data not code" in charter,
    }
    for recipe_id, rel in RECIPE_RON:
        checks.update(_recipe_checks(root, recipe_id, rel))
    for recipe_id, rel in RECIPE_JSON:
        checks.update(_recipe_checks(root, recipe_id, rel))

    green = all(checks.values())
    return {
        "gate": GATE_ID,
        "issue": "CITY-C3",
        "program": "PLAN-CITY-GRAMMAR-v1",
        "charter_doc": CHARTER_DOC,
        "recipe_ron_paths": [rel for _, rel in RECIPE_RON],
        "recipe_json_examples": [rel for _, rel in RECIPE_JSON],
        "checks": checks,
        "acceptance": {
            "B1_vocabulary": checks.get("vocabulary_five_primitives"),
            "B2_three_charters": checks.get("three_recipe_charters"),
            "B3_ron_on_disk": all(checks.get(f"{rid}_file") for rid, _ in RECIPE_RON),
            "B4_seed_chain": checks.get("seed_chain_documented"),
            "B5_catalog_ids": all(
                checks.get(f"{rid}_catalog_archetype") for rid, _ in RECIPE_RON
            ),
            "B8_critique_no_rust_layout": checks.get("no_rust_layout_shortcut"),
        },
        "green": green,
        "verdict": "PASS" if green else "FAIL",
        "handoff": {
            "coder_mcp": "block_recipe_v1.schema.json",
            "coder": "CITY-G1-C3-001 block_recipe_evaluator",
            "blocked_on": ["CITY-G1-C2-001 BlockFrame"],
        },
    }


def refresh_city_block_recipe_charter_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    body = run_city_block_recipe_charter_audit(repo=root)
    body["_agent_meta"] = {
        "schema": "city_block_recipe_charter_live_v1",
        "written_at_epoch_secs": int(time.time()),
        "profile": "DES_CITY_BLOCK_RECIPE",
        "source_system": "dmcp_city_block_recipe",
        "relative_path": WITNESS_REL,
        "ritual": f"BLANG:WIT-HON→Q✓ {GATE_ID}" if body.get("green") else "BLANG:WIT-HON FAIL",
        "agent": "designer-mcp",
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = WITNESS_REL
    return body
