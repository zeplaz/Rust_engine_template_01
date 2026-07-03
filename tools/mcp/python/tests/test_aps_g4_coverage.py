"""APS-G4-COVERAGE-001 — building_set_coverage + pilot_hardcode lint → G4 tier."""

from __future__ import annotations

from rust_engine_mcp import grammar_build_set
from rust_engine_mcp.paths import repo_root


def test_building_set_coverage_green_full() -> None:
    body = grammar_build_set.building_set_coverage_report()
    assert body["green"] is True
    assert body["pilot_hardcode_green"] is True
    assert body["grammar_pilot_count"] >= 4
    assert body["preset_count"] >= 4


def test_grammar_set_tier_g4() -> None:
    tier = grammar_build_set.grammar_set_tier()
    assert tier["tier"] == "G4"
    assert tier["reasons"] == []


def test_write_aps_g4_coverage_witness_green() -> None:
    body = grammar_build_set.write_aps_g4_coverage_witness()
    assert body["green"] is True
    assert body["building_set_coverage_green"] is True
    assert body["grammar_set_tier"] == "G4"
    assert body.get("witness_honesty", {}).get("status") == "passed"
    assert (repo_root() / "debug_runs/aps_g4_coverage_live.json").is_file()
