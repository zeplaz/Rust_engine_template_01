"""APSR-MUTATION-REGRESS-001 — mutation ceiling + lane-roundtrip wiring lock."""

from __future__ import annotations

from rust_engine_mcp.apsr_mutation_regress import (
    NEXT_RESIDUAL,
    TASK_ID,
    WITNESS_REL,
    check_lane_roundtrip_wiring,
    run_mutation_regress_audit,
    write_apsr_mutation_regress_001_witness,
)
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.suite_state_mutation_inventory import MAX_DIRECT_MUTATION_SITES


def test_lane_roundtrip_wiring_static() -> None:
    body = check_lane_roundtrip_wiring()
    assert body["ok"] is True
    assert body["subscribes_lane_changed"] is True
    assert body["calls_sync_from_state"] is True


def test_mutation_regress_audit_green() -> None:
    body = run_mutation_regress_audit(sync_line_drift=True)
    assert body["task_id"] == TASK_ID
    assert body["mutation_inventory_green"] is True
    assert body["growth_ok"] is True
    assert body["live_mutation_count"] <= MAX_DIRECT_MUTATION_SITES
    assert body["lane_roundtrip_wiring"]["ok"] is True
    assert body["green"] is True
    assert body["next_residual"] == NEXT_RESIDUAL


def test_mutation_regress_witness_green() -> None:
    out = write_apsr_mutation_regress_001_witness(sync_line_drift=True)
    assert out.get("green") is True
    assert out.get("witness_honesty", {}).get("status") == "passed"
    assert (repo_root() / WITNESS_REL).is_file()
