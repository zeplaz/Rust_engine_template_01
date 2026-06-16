"""MCP-SPINE-CHAIN-001 — tile_spine_run orchestrator."""

from __future__ import annotations

import json

from rust_engine_mcp import tile_spine_run
from rust_engine_mcp.paths import repo_root

EXAMPLE_SNAPSHOT = (
    "tools/mcp/schemas/examples/assembly_snapshot_rowhouse_victorian_production_v1.json"
)
EXAMPLE_BATCH = "tile_rowhouse_victorian_production_v1"


def test_tile_spine_run_digest_chain_green():
    result = tile_spine_run.tile_spine_run(
        {
            "schema": "tile_spine_run_request_v1",
            "snapshot_path": EXAMPLE_SNAPSHOT,
            "batch_id": EXAMPLE_BATCH,
            "steps": ["p0_gate", "snapshot_digest"],
            "ship": False,
            "write_witness": False,
        }
    )
    assert result["schema"] == "tile_spine_run_result_v1"
    assert result["ok"] is True
    assert result["stopped_at"] is None
    assert len(result["steps"]) == 2
    assert all(row["ok"] for row in result["steps"])


def test_tile_spine_run_stops_on_missing_snapshot():
    result = tile_spine_run.tile_spine_run(
        {
            "schema": "tile_spine_run_request_v1",
            "snapshot_path": "no/such/snapshot.json",
            "batch_id": EXAMPLE_BATCH,
            "steps": ["p0_gate", "snapshot_digest"],
            "write_witness": False,
        }
    )
    assert result["ok"] is False
    assert result["stopped_at"] == "p0_gate"


def test_tile_spine_run_honest_bake_blocks_dry_run_ship():
    import os

    prev = os.environ.get("RUST_ENGINE_TILE_DRY_RUN")
    os.environ["RUST_ENGINE_TILE_DRY_RUN"] = "1"
    try:
        check = tile_spine_run._tile_promotion_honest_check(
            batch_path=repo_root()
            / "tools/mcp/schemas/examples/tile_batch_rowhouse_victorian_production_v1.json",
            staging=repo_root() / "assets/staging/tiles" / EXAMPLE_BATCH,
            ship=True,
            honest_bake=True,
        )
        assert check["ok"] is False
        assert "DRY_RUN" in check["artist_message"]
    finally:
        if prev is None:
            os.environ.pop("RUST_ENGINE_TILE_DRY_RUN", None)
        else:
            os.environ["RUST_ENGINE_TILE_DRY_RUN"] = prev


def test_refresh_tile_spine_run_witness():
    assert tile_spine_run.refresh_tile_spine_run_witness()
    data = json.loads(
        (repo_root() / tile_spine_run.TILE_SPINE_RUN_WITNESS).read_text(encoding="utf-8")
    )
    assert data.get("gate_id") == "MCP-SPINE-CHAIN-001"
    assert data.get("green") is True
    assert data.get("ok") is True
