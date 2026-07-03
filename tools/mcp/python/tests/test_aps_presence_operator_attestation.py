"""Tests for OVR-APS-PRESENCE-OPERATOR-001 attestation witness."""

from rust_engine_mcp.aps_presence_operator_attestation import validate_g3_presence_checklist


def test_g3_checklist_passes_sample_payload() -> None:
    presence = {
        "green": True,
        "grammar_set_tier": {"tier": "G3"},
        "g4_guards": {"building_set_coverage_green": False},
        "ui_presence": {
            "tier": "G3",
            "tier_chip": "G3 — layer depth",
            "kit_hint_visible": False,
            "dna_panel_visible": True,
            "iterate_panel_visible": True,
            "archetype_combo_count": 4,
            "assembly_empty_label": "No assembly yet — tune shape bias in the panels below.",
        },
    }
    _, failures = validate_g3_presence_checklist(presence)
    assert failures == []
