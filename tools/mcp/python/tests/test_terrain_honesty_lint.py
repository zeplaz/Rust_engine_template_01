"""Tests for terrain_honesty_lint."""

from __future__ import annotations

from rust_engine_mcp import terrain_honesty_lint


def test_terrain_honesty_lint_runs() -> None:
    body = terrain_honesty_lint.validate_terrain_honesty_report(
        write_witness=False,
        compress=3,
    )
    assert body["validator"] == "terrain_honesty"
    assert "summary" in body
    assert body["facts"]["minimap_terrain_source"] in ("world_raster", "gpu_atlas", "minimap pass.rs missing")
    # Current tree: dishonest docs + variant name → warnings OK, no hard fail required
    assert "ok" in body


def test_token_savings_lists_terrain_honesty() -> None:
    from rust_engine_mcp import agent_queue

    g = agent_queue.token_savings_guide()
    assert "terrain_honesty" in g["briefs"]
