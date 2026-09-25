"""PCI-21 — effect-chain-smoke stays isolated, honest, and under two minutes."""

from __future__ import annotations

from rust_engine_mcp.effect_chain_smoke import BUDGET_MS, CHAIN_STEPS, run_effect_chain_smoke
from rust_engine_mcp.paths import repo_root


def _mtime(path):
    return path.stat().st_mtime_ns if path.is_file() else None


def test_effect_chain_smoke_isolated_under_budget() -> None:
    root = repo_root()
    registry = root / "assets" / "effects" / "registry" / "spark_shower" / "manifest.json"
    witness = root / "debug_runs" / "artist_vfx_pipeline_live.json"
    before_registry = _mtime(registry)
    before_witness = _mtime(witness)

    body = run_effect_chain_smoke(write_witness=False)

    assert body["green"] is True
    assert body["cron_scheduled"] is False
    assert body["writes_live_registry"] is False
    assert body["force"] is False
    assert body["under_budget"] is True
    assert body["elapsed_ms"] <= BUDGET_MS
    assert body["budget_ms"] == 120_000
    assert tuple(body["steps"]) == CHAIN_STEPS
    assert body["count"] == 3
    assert body["errors"] == []
    for row in body["effects"]:
        assert row["validate_status"] == "passed"
        assert row["honest_gate"] == "honest"
        assert row["promoted_honest_gate"] == "honest"
        assert row["frames_captured"] >= 1
    assert _mtime(registry) == before_registry
    assert _mtime(witness) == before_witness
