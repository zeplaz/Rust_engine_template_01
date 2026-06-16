"""BUILD-READ-GRAMMAR-v0-002 + BUILD-READ-CONSUMER-MCP-001 tests."""

from __future__ import annotations

from rust_engine_mcp import arch_build_grammar
from rust_engine_mcp.mcp_productivity_p0 import snapshot_digest


def test_load_all_presets_with_meta_strip() -> None:
    ids = arch_build_grammar.list_preset_ids()
    assert len(ids) >= 2
    for pid in ids:
        preset = arch_build_grammar.load_preset(pid)
        assert preset["preset_id"] == pid
        assert "_meta" not in preset


def test_apply_to_snapshot_roundtrip() -> None:
    snap = {"schema_version": 1, "assembly_id": "test", "seed": 1}
    out = arch_build_grammar.apply_to_snapshot(
        snap,
        preset_id=arch_build_grammar.DEFAULT_PRESET_ID,
        include=True,
    )
    assert out["arch_build_grammar_preset_id"] == arch_build_grammar.DEFAULT_PRESET_ID
    assert out["arch_dna"]["F"] == "logistics"
    assert "beta_yard" in out["pressure_field"]


def test_arch_dna_snapshot_brief_rail_warehouse() -> None:
    body = arch_build_grammar.arch_dna_snapshot_brief(
        "tools/mcp/schemas/examples/assembly_snapshot_rail_warehouse_pilot_v1.json"
    )
    assert body["ok"] is True
    assert body["wired"] is True
    assert body["f_axis"] == "logistics"
    assert body["pressure_field"]["beta_yard"] > 0


def test_snapshot_digest_includes_arch_dna() -> None:
    body = snapshot_digest(
        "tools/mcp/schemas/examples/assembly_snapshot_rail_warehouse_pilot_v1.json"
    )
    assert body["ok"] is True
    assert body["arch_dna"]["wired"] is True
    assert body["arch_dna"]["preset_id"]


def test_consumer_contract_fields() -> None:
    contract = arch_build_grammar.consumer_contract()
    assert contract["task_id"] == "BUILD-READ-CONSUMER-MCP-001"
    assert "arch_build_grammar_preset_id" in contract["snapshot_fields"]
    assert "pressure_field" in contract["snapshot_fields"]
    assert len(contract["preset_ids"]) >= 2


def test_write_aps_dna_consumer_witness_green() -> None:
    body = arch_build_grammar.write_aps_dna_consumer_witness()
    assert body["green"] is True
    assert body["preset_load_errors"] == []
