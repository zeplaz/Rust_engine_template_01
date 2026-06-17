"""APS-EVO-E5-EXTRACT-PARITY-001 tests."""

from __future__ import annotations

from rust_engine_mcp.aps_veg_extract_parity import (
    APS_VEG_EXTRACT_PARITY_WITNESS,
    ENGINE_AUTHORITY,
    check_veg_extract_parity,
    refresh_aps_veg_extract_parity_witness,
)
from rust_engine_mcp.paths import repo_root


def test_veg_extract_parity_headless() -> None:
    body = check_veg_extract_parity()
    assert body.get("subset_ok") is True
    assert body.get("resolver_parity_ok") is True
    assert body.get("extract_witness_green") is True
    assert body.get("panel_wired") is True
    assert body.get("engine_authority") == ENGINE_AUTHORITY
    assert body.get("parity_green") is True
    assert not body.get("missing_from_resolver")


def test_veg_extract_parity_witness() -> None:
    body = refresh_aps_veg_extract_parity_witness()
    assert body.get("green") is True
    assert body.get("parity_green") is True
    assert body.get("witness_honesty", {}).get("status") == "passed"
    out = repo_root() / APS_VEG_EXTRACT_PARITY_WITNESS
    assert out.is_file()
