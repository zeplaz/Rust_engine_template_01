"""MCP-P2-KIT002-G4 — designer-mcp keyframe matrix sign-off (honest gate)."""

from __future__ import annotations

import json

from rust_engine_mcp import kit_production_002
from rust_engine_mcp.paths import repo_root


def test_g4_evaluate_honest_when_stills_missing() -> None:
    evaluation = kit_production_002.evaluate_kit_production_002_g4()
    assert evaluation.get("verdict") == "FAIL"
    assert evaluation.get("proceed_ship") is False
    assert evaluation.get("art_quality") == "rejected_headless_procedural"
    assert "keyframe_stills_folder_missing" in (evaluation.get("blocked_by") or [])
    gates = evaluation.get("gates") or {}
    assert gates.get("g4_3_keyframe_minimum_stills_review") == "fail"
    assert gates.get("g4_8_proceed_ship") == "fail"


def test_g4_witness_written_and_manifest_gate() -> None:
    body = kit_production_002.refresh_kit_production_002_g4_witness()
    assert body.get("gate") == "MCP-P2-KIT002-G4"
    assert body.get("green") is False
    assert body.get("proceed_tile_ship") is False
    assert body.get("unblocks") == []

    witness_path = repo_root() / kit_production_002.G4_WITNESS_REL
    assert witness_path.is_file()
    on_disk = json.loads(witness_path.read_text(encoding="utf-8"))
    assert on_disk.get("verdict") == "FAIL"
    assert on_disk.get("_agent_meta", {}).get("agent") == "designer-mcp"

    signoff_path = repo_root() / kit_production_002.G4_SIGNOFF_REL
    assert signoff_path.is_file()
    assert "proceed_ship: no" in signoff_path.read_text(encoding="utf-8")

    manifest = kit_production_002.load_manifest()
    assert manifest.get("gate") == "G4"

    pilot_g4 = repo_root() / "debug_runs/art_pipeline/warehouse_production_keyframe_g4_live.json"
    pilot_body = json.loads(pilot_g4.read_text(encoding="utf-8"))
    assert pilot_body.get("green") is False
    assert pilot_body.get("kit_gate") == "MCP-P2-KIT002-G4"
