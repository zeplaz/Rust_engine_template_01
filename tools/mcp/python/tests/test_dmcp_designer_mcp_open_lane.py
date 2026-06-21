"""Tests for dmcp_designer_mcp_open_lane witness."""

from __future__ import annotations

from pathlib import Path

from rust_engine_mcp.dmcp_designer_mcp_open_lane import (
    refresh_dmcp_designer_mcp_open_lane_witness,
    run_open_lane_audit,
)
from rust_engine_mcp.paths import repo_root


def test_open_lane_audit_green() -> None:
    audit = run_open_lane_audit(repo=repo_root())
    assert audit["green"] is True
    assert audit["done_count"] == 2
    assert audit["verdict"] == "PASS_WITH_NOTES"
    veg = next(r for r in audit["rows"] if r["id"] == "DMCP-VEG-ATLAS-SHIP-001")
    assert veg["audit"]["checks"]["ship_false_honest"] is True
    assert veg["audit"]["checks"]["proceed_ship_honest_no"] is True
    plain = next(r for r in audit["rows"] if r["id"] == "DMCP-ATLAS-QC-PLAIN-002")
    assert plain["audit"]["verdict"] == "PASS"


def test_open_lane_witness_writes(tmp_path: Path, monkeypatch) -> None:
    import rust_engine_mcp.dmcp_designer_mcp_open_lane as mod

    monkeypatch.setattr(mod, "repo_root", lambda: tmp_path)
    body = refresh_dmcp_designer_mcp_open_lane_witness(repo=tmp_path)
    assert body["green"] is False
    assert (tmp_path / mod.WITNESS_REL).is_file()
