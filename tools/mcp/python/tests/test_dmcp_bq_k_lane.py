"""BQ-K1/K2/K3 designer-mcp charter witness tests."""

from __future__ import annotations

from rust_engine_mcp.dmcp_bq_k_lane import (
    K1_GATE,
    K2_GATE,
    K3_GATE,
    refresh_bq_k_lane_witness,
    run_bq_k1_kitfill_audit,
    run_bq_k3_grammar_audit,
)
from rust_engine_mcp.kit_coverage_audit import audit_bq_k2_coverage


def test_bq_k1_kitfill_audit_green() -> None:
    audit = run_bq_k1_kitfill_audit()
    assert audit["green"] is True
    assert audit["verdict"] == "PASS"
    assert audit["gate"] == K1_GATE
    assert audit["checks"]["job_count_11"] is True


def test_bq_k2_coverage_audit_green() -> None:
    audit = audit_bq_k2_coverage()
    assert audit["green"] is True
    assert audit["verdict"] == "PASS"
    assert audit["gate"] == K2_GATE
    assert audit["slot_resolution"]["green"] is True


def test_bq_k3_grammar_audit_green() -> None:
    audit = run_bq_k3_grammar_audit()
    assert audit["green"] is True
    assert audit["verdict"] == "PASS"
    assert audit["gate"] == K3_GATE
    assert audit["checks"]["three_grammars"] is True


def test_bq_k_lane_witness_writes() -> None:
    body = refresh_bq_k_lane_witness()
    assert body["green"] is True
    assert body["gates"][K1_GATE]["green"] is True
    assert body["gates"][K2_GATE]["green"] is True
    assert body["gates"][K3_GATE]["green"] is True
