"""DMCP art spine hub wave witness tests."""

from __future__ import annotations

from rust_engine_mcp.dmcp_art_spine_hub_wave import (
    refresh_dmcp_art_spine_hub_wave_witness,
    run_art_spine_hub_wave_audit,
)


def test_dmcp_art_spine_hub_wave_witness() -> None:
    body = refresh_dmcp_art_spine_hub_wave_witness()
    assert body.get("green") is True
    assert body.get("done_count") == 4


def test_rowhouse_burn_ladder() -> None:
    audit = run_art_spine_hub_wave_audit()
    row = next(r for r in audit["rows"] if r["id"] == "DMCP-TILE-ROWHOUSE-V2-001")
    assert row["audit"]["checks"]["burn_frames_8"] is True


def test_mat_pilot_24_profiles() -> None:
    audit = run_art_spine_hub_wave_audit()
    row = next(r for r in audit["rows"] if r["id"] == "DMCP-MAT-PROFILE-PILOT-002")
    assert row["audit"]["profile_count"] == 24
