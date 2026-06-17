"""MCP-APS-WIT-HON-HOOK-001 + atlas domain display."""

from __future__ import annotations

from rust_engine_mcp.aps_atlas_qc import format_atlas_qc_display
from rust_engine_mcp.aps_witness_honesty import gate_green_with_witness_honesty
from rust_engine_mcp.validators.report import ValidationReport


def test_wit_hon_blocks_dishonest_green() -> None:
    body = {
        "green": True,
        "art_quality": "rejected_stub",
        "_agent_meta": {"schema": "aps_test_live_v1"},
    }
    out = gate_green_with_witness_honesty(
        body,
        "tools/mcp/schemas/examples/witness_honesty_fixtures/bad_art_dishonest_live.json",
    )
    assert out["green"] is False
    assert out["witness_honesty"]["status"] == "failed"


def test_format_atlas_qc_display_landscape_domain() -> None:
    report = ValidationReport(validator="atlas_meta", status="passed")
    text, color = format_atlas_qc_display(report, ["ok"], atlas_domain="landscape")
    assert "Landscape" in text
    assert "_landscape_atlas_index" in text
    assert color == "#006400"
