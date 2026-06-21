"""PWR-ART-DOWNSTREAM-CLOSE-001 rollup witness."""

from __future__ import annotations

import json

from rust_engine_mcp.mcp_pwr_utility import (
    CLOSE_WITNESS_REL,
    INDUSTRIAL_ACTIVATION_REL,
    refresh_power_grid_art_downstream_close_witness,
    sync_power_grid_queue_statuses,
)
from rust_engine_mcp.paths import repo_root


def test_power_grid_downstream_close_witness_green() -> None:
    body = refresh_power_grid_art_downstream_close_witness()
    assert body["green"] is True
    assert body["power_utility_art"]["utility_glb_paths_set"] is True
    path = repo_root() / CLOSE_WITNESS_REL
    assert path.is_file()
    loaded = json.loads(path.read_text(encoding="utf-8"))
    assert loaded["gate"] == "PWR-ART-DOWNSTREAM-CLOSE-001"
    assert all(loaded["child_witnesses"].values())


def test_industrial_activation_utility_art_block() -> None:
    body = refresh_power_grid_art_downstream_close_witness()
    ind = json.loads((repo_root() / INDUSTRIAL_ACTIVATION_REL).read_text(encoding="utf-8"))
    art = ind.get("power_utility_art") or {}
    assert art.get("substation_glb")
    assert art.get("transformer_glb")
    assert art.get("green") is True
    assert body["power_utility_art"]["green"] is True


def test_sync_power_grid_queue_marks_done() -> None:
    result = sync_power_grid_queue_statuses()
    assert "MCP-PWR-PROMOTE-SUBSTATION-001" in result["updated"] or result["updated"] == []
    queue = json.loads(
        (repo_root() / "tools/orchestrator/queues/power_grid_art_downstream_queue.json").read_text(
            encoding="utf-8"
        )
    )
    by_id = {row["id"]: row["status"] for row in queue["drain"]}
    assert by_id["MCP-PWR-UTILITY-MANIFEST-001"] == "done"
    assert by_id["PWR-ART-DOWNSTREAM-CLOSE-001"] == "done"
