"""Canonical module_id inventory — design_procedural_module_kit_v1.md § Module inventory."""

from __future__ import annotations

# Canonical IDs (lod0/production must match one of these).
CANONICAL_MODULE_IDS: frozenset[str] = frozenset(
    {
        # walls (10)
        "wall_brick_1u",
        "wall_brick_2u",
        "wall_concrete_1u",
        "wall_concrete_2u",
        "wall_wood_1u",
        "wall_wood_2u",
        "wall_steel_1u",
        "wall_glass_curtain_1u",
        "wall_industrial_panel_2u",
        "wall_military_bunker_1u",
        # windows (10)
        "win_single_1u",
        "win_double_1u",
        "win_strip_2u",
        "win_arched_1u",
        "win_industrial_3u",
        "win_shop_2u",
        "win_house_1u",
        "win_office_1u",
        "win_bunker_slit",
        "win_skylight_1u",
        # doors (10)
        "door_residential",
        "door_shop",
        "door_warehouse",
        "door_garage",
        "door_office",
        "door_civic",
        "door_military",
        "door_factory",
        "door_double_shop",
        "door_gate_industrial",
        # roofs (10)
        "roof_flat",
        "roof_pitched_gable",
        "roof_pitched_hip",
        "roof_shed",
        "roof_sawtooth",
        "roof_industrial_shed_2u",
        "roof_parapet",
        "roof_metal_low",
        "roof_tile",
        "roof_bunker",
        "roof_canopy",
        # corner / prop (10)
        "corner_L",
        "corner_T",
        "corner_parapet",
        "prop_fence",
        "prop_light",
        "prop_vent",
        "prop_tank",
        "prop_transformer",
        "prop_ac",
        "prop_chimney",
        # PG-MODULE-AUDIT-002 P3 grammar extensions (warehouse pilot)
        "stack_chimney_1u",
        "platform_dock_2u",
    }
)
