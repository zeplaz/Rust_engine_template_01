"""DES-APS-SESSION-DUMP-001 — bundled APS session presence witness."""

from __future__ import annotations

import json

from rust_engine_mcp import grammar_build_set
from rust_engine_mcp.paths import repo_root


def test_aps_session_presence_dump_tier_aligned() -> None:
    body = grammar_build_set.aps_session_presence_dump()
    tier = body["grammar_set_tier"]["tier"]
    assert body["ui_presence"]["tier"] == tier
    assert body["green"] is True
    assert body["gate"] == "DES-APS-SESSION-DUMP-001"
    assert "g4_guards" in body
    assert "expansion" in body


def test_write_aps_session_presence_witness() -> None:
    body = grammar_build_set.write_aps_session_presence_witness()
    tier = body["grammar_set_tier"]["tier"]
    assert body["green"] is True
    assert body["ui_presence"]["tier"] == tier
    assert body.get("witness_honesty", {}).get("status") == "passed"
    live_path = repo_root() / grammar_build_set.SESSION_PRESENCE_WITNESS
    assert live_path.is_file()
    live = json.loads(live_path.read_text(encoding="utf-8"))
    assert live["ui_presence"]["tier"] == live["grammar_set_tier"]["tier"]


def test_grammar_tier_ui_presence_g3_exposure() -> None:
    ui = grammar_build_set.grammar_tier_ui_presence_from_tier("G3")
    assert ui["tier"] == "G3"
    assert ui["tier_chip"] == "G3 — layer depth"
    assert ui["kit_hint_visible"] is False
    assert ui["dna_panel_visible"] is True
    assert ui["iterate_panel_visible"] is True
    assert ui["set_health_visible"] is True
    assert ui["archetype_combo_count"] >= 4
    assert "shape bias" in ui["assembly_empty_label"]
