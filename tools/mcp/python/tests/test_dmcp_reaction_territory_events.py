"""Tests for dmcp_reaction_territory_events witness."""

from __future__ import annotations

from pathlib import Path

from rust_engine_mcp.dmcp_reaction_territory_events import (
    refresh_dmcp_reaction_territory_events_witness,
    run_reaction_territory_events_audit,
)
from rust_engine_mcp.paths import repo_root


def test_reaction_territory_events_audit_green() -> None:
    audit = run_reaction_territory_events_audit(repo=repo_root())
    assert audit["green"] is True
    assert audit["verdict"] == "PASS"
    assert audit["checks"]["variant_keys"] is True
    assert audit["checks"]["tag_anchors"] is True
    assert audit["checks"]["preview_states"] is True
    assert "heritage_site_destruction" in audit["event_ids"]


def test_reaction_territory_witness_writes(tmp_path: Path) -> None:
    body = refresh_dmcp_reaction_territory_events_witness(repo=tmp_path)
    assert body["green"] is False
    assert body["checks"].get("catalog_json") is False
    assert (tmp_path / "debug_runs/art_pipeline/dmcp_reaction_territory_events_live.json").is_file()
