# APS tag vocabulary — labels, hints, audit coverage

from __future__ import annotations

from rust_engine_mcp.aps_tag_vocabulary import (
    assembly_variant_tag_label,
    mandate_tag_label,
    tag_vocabulary_audit,
)
from rust_engine_mcp.reaction_territory import TAG_FAMILIES


def test_mandate_tag_vocabulary_full_coverage() -> None:
    audit = tag_vocabulary_audit()
    assert audit["green"] is True, audit
    assert not audit["missing_labels"]
    assert not audit["families_with_duplicates"]


def test_mandate_tag_labels_are_human_not_snake() -> None:
    for _family, tags in TAG_FAMILIES.items():
        for tag in set(tags):
            label = mandate_tag_label(tag)
            assert label
            assert label != tag or " " in label


def test_assembly_variant_tag_labels() -> None:
    assert assembly_variant_tag_label("clean") == "Clean"
    assert "construction" in assembly_variant_tag_label("construction").lower()


def test_reaction_event_context_includes_anchors() -> None:
    from rust_engine_mcp.aps_tag_vocabulary import reaction_event_context

    line = reaction_event_context(
        {
            "label": "Heritage site destruction",
            "tag_anchors": ["burn_origin", "heritage_marker"],
            "preview_states": ["damaged", "burning"],
        }
    )
    assert "Heritage site destruction" in line
    assert "Burn origin" in line


def test_tag_families_no_duplicate_entries() -> None:
    for family, tags in TAG_FAMILIES.items():
        assert len(tags) == len(set(tags)), f"duplicate tags in {family}"
