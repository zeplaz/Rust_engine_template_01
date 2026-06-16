"""APS witness refresh smoke."""

from __future__ import annotations

from rust_engine_mcp.aps_witness_refresh import refresh_aps_witnesses
from rust_engine_mcp.paths import repo_root


def test_refresh_bundle_green() -> None:
    body = refresh_aps_witnesses()
    assert body.get("program_id") == "APS-WITNESS-REFRESH-001"
    assert (repo_root() / "debug_runs/aps_artist_tool_modules_live.json").is_file()
    assert body.get("module_count", 0) >= 10
