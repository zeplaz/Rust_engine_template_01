"""GRAMMAR-ITER-001 — iterate_grammar determinism + witness refresh."""

from __future__ import annotations

import json
from pathlib import Path

from rust_engine_mcp import building_grammar
from rust_engine_mcp.grammar_iterate import (
    compute_cell_diff_map,
    compute_snapshot_diff,
    iterate_grammar,
    refresh_grammar_iter_aps1_witness,
    refresh_grammar_iter_e2e_witness,
    refresh_grammar_iter_massing_witness,
    refresh_grammar_002_roof_facade_witness,
)
from rust_engine_mcp.paths import repo_root


def _massing_request() -> dict:
    return {
        "schema": "grammar_iterate_request_v1",
        "mode": "massing",
        "seed": 43,
        "archetype_id": "IndustrialWarehouse",
        "district_style": "industrial_west",
        "base_snapshot_path": (
            "tools/mcp/schemas/examples/"
            "assembly_snapshot_warehouse_industrial_west_production_v1.json"
        ),
        "overrides": {
            "massing_strategy": "double_hall",
            "footprint": {"width": 10, "depth": 6, "floors": 2},
        },
        "preserve_layers": ["district_style", "age"],
        "parent_lineage_id": "industrial_west_8x9_s43_f75a",
    }


def test_generate_with_overrides_pins_massing():
    g = building_grammar.generate_with_overrides(
        "IndustrialWarehouse",
        "industrial_west",
        43,
        massing_strategy="double_hall",
        footprint={"width": 10, "depth": 6, "floors": 2},
    )
    assert g["massing_strategy"] == "double_hall"
    assert g["width"] == 10
    assert g["depth"] == 6
    assert g["floors"] == 2


def test_iterate_grammar_massing_deterministic():
    req = _massing_request()
    r1 = iterate_grammar(req)
    r2 = iterate_grammar(req)
    assert r1["ok"] is True, r1.get("errors")
    assert r2["ok"] is True
    assert r1["snapshot"]["assembly_id"] == r2["snapshot"]["assembly_id"]
    assert r1["snapshot"]["grammar_lineage"]["iteration_seq"] == 1
    assert "massing" in r1["diff"]["layers_touched"]


def test_iterate_grammar_roof_mode():
    req = _massing_request()
    req["mode"] = "roof"
    req["overrides"] = {"roof_rule_id": "roof_flat"}
    result = iterate_grammar(req)
    assert result["ok"] is True, result.get("errors")
    assert "roof" in result["diff"]["layers_touched"]
    chain = result["snapshot"].get("grammar_rule_chain") or {}
    assert chain.get("roof") == "roof_flat"


def test_iterate_grammar_facade_mode():
    req = _massing_request()
    req["mode"] = "facade"
    req["overrides"] = {"door_slot": "door_wide", "facade_rule_id": "loading_bay"}
    result = iterate_grammar(req)
    assert result["ok"] is True, result.get("errors")
    assert "facade" in result["diff"]["layers_touched"]


def test_iterate_grammar_deferred_detail():
    req = _massing_request()
    req["mode"] = "detail"
    result = iterate_grammar(req)
    assert result["ok"] is False
    assert result["errors"][0]["code"] == "GRAMMAR_ITER_DEFERRED"


def test_compute_cell_diff_map():
    example = repo_root() / "tools/mcp/schemas/examples/assembly_snapshot_grammar_lineage_example.json"
    snap = json.loads(example.read_text(encoding="utf-8"))
    assert compute_cell_diff_map(snap, snap) == {}


def test_compute_snapshot_diff_counts():
    before = {"module_placements": [{"grid_x": 0, "grid_y": 0, "floor": 0, "module_id": "a", "token": "W"}]}
    after = {
        "module_placements": [
            {"grid_x": 0, "grid_y": 0, "floor": 0, "module_id": "b", "token": "W"},
            {"grid_x": 1, "grid_y": 0, "floor": 0, "module_id": "c", "token": "W"},
        ]
    }
    diff = compute_snapshot_diff(before, after)
    assert diff["cells_added"] == 1
    assert diff["cells_changed"] == 1


def test_refresh_massing_witness():
    assert refresh_grammar_iter_massing_witness()
    path = repo_root() / "debug_runs/grammar_iter_001_massing_live.json"
    body = json.loads(path.read_text(encoding="utf-8"))
    assert body["green"] is True
    assert body["mode"] == "massing"
    assert body["determinism"] == "pass"


def test_refresh_aps1_witness():
    assert refresh_grammar_iter_aps1_witness()
    path = repo_root() / "debug_runs/grammar_iter_001_aps1_live.json"
    body = json.loads(path.read_text(encoding="utf-8"))
    assert body["green"] is True
    assert body["inspector_lineage_wired"] is True
    assert body["footprint_diff_wired"] is True


def test_refresh_e2e_witness():
    assert refresh_grammar_iter_e2e_witness()
    body = json.loads((repo_root() / "debug_runs/grammar_iter_001_e2e_live.json").read_text(encoding="utf-8"))
    assert body["green"] is True
    assert body["massing_ok"] is True
    assert body["roof_iterate_ok"] is True
    assert body["aps1_ok"] is True


def test_refresh_grammar_002_roof_facade_witness():
    assert refresh_grammar_002_roof_facade_witness()
    body = json.loads((repo_root() / "debug_runs/grammar_002_roof_facade_live.json").read_text(encoding="utf-8"))
    assert body["green"] is True
