"""DMCP parallel lane BKKF witness tests."""

from __future__ import annotations

from rust_engine_mcp.dmcp_parallel_lane_bkkf import (
    refresh_dmcp_parallel_lane_bkkf_witness,
    run_parallel_lane_bkkf_audit,
)


def test_dmcp_parallel_lane_bkkf_witness() -> None:
    body = refresh_dmcp_parallel_lane_bkkf_witness()
    assert body.get("green") is True
    assert body.get("done_count") == 3


def test_factory_cluster_grammar_on_disk() -> None:
    audit = run_parallel_lane_bkkf_audit()
    factory = next(r for r in audit["rows"] if r["id"] == "DMCP-GRAM-ARCHETYPE-FACTORY-001")
    assert factory["grammar_audit"]["green"] is True
