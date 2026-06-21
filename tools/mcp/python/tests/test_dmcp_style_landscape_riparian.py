"""Tests for dmcp_style_landscape_riparian witness."""

from __future__ import annotations

from pathlib import Path

from rust_engine_mcp.dmcp_style_landscape_riparian import (
    refresh_dmcp_style_landscape_riparian_witness,
    run_riparian_style_audit,
)
from rust_engine_mcp.paths import repo_root


def test_riparian_style_audit_green() -> None:
    audit = run_riparian_style_audit(repo=repo_root())
    assert audit["green"] is True
    assert audit["verdict"] == "PASS"
    assert audit["checks"]["topologies_in_preset"] is True


def test_riparian_witness_writes(tmp_path: Path) -> None:
    body = refresh_dmcp_style_landscape_riparian_witness(repo=tmp_path)
    assert body["green"] is False
    assert (tmp_path / "debug_runs/art_pipeline/dmcp_style_landscape_riparian_live.json").is_file()
