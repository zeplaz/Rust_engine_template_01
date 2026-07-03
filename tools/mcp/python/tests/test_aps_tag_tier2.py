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


def test_write_aps_tag_tier2_witness_green() -> None:
    body = aps_tag_tier2.write_aps_tag_tier2_witness()
    assert body.get("green") is True
    assert body.get("witness_honesty", {}).get("status") == "passed"
    assert (repo_root() / aps_tag_tier2.WITNESS_REL).is_file()


def test_ux_polish_tail_witness() -> None:
    from rust_engine_mcp.aps_ux_polish_tail_witness import write_aps_ux_polish_tail_witness

    aps_tag_tier2.write_aps_tag_tier2_witness()
    body = write_aps_ux_polish_tail_witness()
    assert body.get("green") is True
    fixes = body.get("fixes") or {}
    assert fixes.get("f5_pipeline_valid_vs_saved") is True
    assert fixes.get("f7_flow_bar_feedback") is True
    assert fixes.get("tag_tier2_impl") is True
