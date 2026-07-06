"""Tests for BQ-K1 kit fill bake + wire."""

from __future__ import annotations

from rust_engine_mcp import bq_k1_kitfill
from rust_engine_mcp.kit_coverage_audit import audit_k1_style_purity_gaps
from rust_engine_mcp.paths import repo_root


def test_k1_job_ids_count() -> None:
    assert len(bq_k1_kitfill.k1_job_ids()) == 11


def test_bq_k1_bake_wire_green() -> None:
    body = bq_k1_kitfill.run_k1_bake_wire(try_rebake=False)
    assert body["promote_ok"] == 11
    assert body["style_purity_gap_count"] == 0
    assert body["green"] is True


def test_bq_k1_bake_witness() -> None:
    body = bq_k1_kitfill.write_bq_k1_bake_witness()
    assert body["green"] is True
    assert (repo_root() / "debug_runs/bq_k1_bake_001_live.json").is_file()


def test_style_purity_gaps_closed() -> None:
    assert audit_k1_style_purity_gaps() == []
