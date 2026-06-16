"""MCP-P2-HONEST-BAKE-001 — tile promotion honest bake gate."""

from __future__ import annotations

import json
import os

import pytest

from rust_engine_mcp import tile_promotion_honest
from rust_engine_mcp.paths import repo_root


def test_honest_check_rejects_smoke_ortho_ship():
    smoke = {
        "batch_id": "test_smoke",
        "bake_source": "smoke_ortho_headless",
        "render": {"method": "blender_orthographic_iso", "seed": 1},
    }
    path = repo_root() / "assets/staging/tiles/_test_honest_smoke.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(smoke) + "\n", encoding="utf-8")
    check = tile_promotion_honest.tile_promotion_honest_check(batch_path=path, ship=True)
    assert check["ok"] is False
    assert "keyframe_pack" in check["artist_message"] or "orthographic" in str(check.get("errors"))


def test_honest_check_rejects_dry_run():
    batch = repo_root() / tile_promotion_honest.EXAMPLE_BATCH
    prev = os.environ.get("RUST_ENGINE_TILE_DRY_RUN")
    os.environ["RUST_ENGINE_TILE_DRY_RUN"] = "1"
    try:
        check = tile_promotion_honest.tile_promotion_honest_check(batch_path=batch, ship=True)
        assert check["ok"] is False
        assert "DRY_RUN" in check["artist_message"]
    finally:
        if prev is None:
            os.environ.pop("RUST_ENGINE_TILE_DRY_RUN", None)
        else:
            os.environ["RUST_ENGINE_TILE_DRY_RUN"] = prev


def test_validate_report_tile_promotion_honest():
    from rust_engine_mcp.validators import run_validator

    batch = repo_root() / tile_promotion_honest.EXAMPLE_BATCH
    rep = run_validator("tile_promotion_honest", str(batch.relative_to(repo_root())), compression_level=4)
    assert rep.status in ("passed", "failed", "warning")


def test_mcp_p2_honest_bake_witness():
    body = tile_promotion_honest.write_mcp_p2_honest_bake_001_witness()
    assert body["gate_id"] == "MCP-P2-HONEST-BAKE-001"
    assert body["ok"] is True
    path = repo_root() / tile_promotion_honest.MCP_P2_HONEST_BAKE_WITNESS
    assert path.is_file()


def test_mcp_p2_run_event_witness():
    from rust_engine_mcp.mcp_p2_run_event import write_mcp_p2_run_event_001_witness, MCP_P2_RUN_EVENT_WITNESS

    body = write_mcp_p2_run_event_001_witness()
    assert body["gate_id"] == "MCP-P2-RUN-EVENT-001"
    assert body["ok"] is True
    assert (repo_root() / MCP_P2_RUN_EVENT_WITNESS).is_file()
