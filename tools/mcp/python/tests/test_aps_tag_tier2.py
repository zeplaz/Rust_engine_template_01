"""APS-TAG-TIER2-IMPL — archetype presets + reaction suggested tags."""

from __future__ import annotations

from rust_engine_mcp import aps_tag_tier2
from rust_engine_mcp.paths import repo_root


def test_archetype_presets_cover_four_grammars() -> None:
    body = aps_tag_tier2.load_archetype_tag_presets()
    archetypes = body.get("archetypes") or {}
    for aid in ("IndustrialWarehouse", "FactoryCluster", "CivicBlock", "RailEdge"):
        row = archetypes[aid]
        assert row.get("mandate_tags")
        assert row.get("semantic_tags")


def test_suggested_tags_from_heritage_event() -> None:
    tags = aps_tag_tier2.suggested_mandate_tags_for_event("heritage_site_destruction")
    assert "burn_origin" in tags
    assert "heritage_marker" in tags




