"""Grammar MCP lane verification — APS-TAGS-002, inspector, footprint UI."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path

import pytest

from rust_engine_mcp import aps_tags, assembly, building_grammar
from rust_engine_mcp.paths import repo_root


def test_aps_mcp_modules_on_disk() -> None:
    root = repo_root()
    assert (root / "tools/mcp/python/rust_engine_mcp/aps_tags.py").is_file()
    assert (root / "tools/mcp/art_pipeline_suite/grammar_inspector.py").is_file()
    assert (root / "tools/mcp/art_pipeline_suite/footprint_canvas.py").is_file()


def test_grammar_mcp_witness_payload() -> None:
    snap = assembly.generate_assembly_snapshot(
        archetype_id="IndustrialWarehouse",
        district_style="industrial_west",
        seed=43,
        source_tier="lod0",
        write=False,
    )
    chain = snap.get("grammar_rule_chain") or {}
    placement = (snap.get("module_placements") or [None])[0] or {}
    body = {
        "gate_id": "GRAMMAR-MCP-LANE",
        "green": bool(chain.get("massing") and placement.get("semantic_tags")),
        "slices": {
            "MCP-APS-TAGS-002": "aps_tags.py + assembly_panel semantic_tags",
            "MCP-APS-GRAMMAR-INSPECTOR-001": "grammar_inspector.py",
            "MCP-APS-UI-003b": "footprint_canvas.py + archetype generate",
        },
        "archetype_id": snap.get("archetype_id"),
        "grammar_rule_chain": chain,
        "sample_semantic_tags": placement.get("semantic_tags"),
    }
    out = repo_root() / "debug_runs" / "grammar_aps_mcp_live.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    assert body["green"]


def test_building_grammar_rule_chain_snapshot_shape() -> None:
    result = building_grammar.generate("IndustrialWarehouse", "industrial_west", 1)
    chain = building_grammar.grammar_rule_chain_snapshot(result)
    assert chain.get("massing")
    assert chain.get("footprint_mode")
