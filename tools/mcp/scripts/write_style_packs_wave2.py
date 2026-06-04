"""MCP-D0-SP-001 + MCP-D0-SP-002 — 7 style pack RON files + manifest witness."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / "assets" / "configs" / "buildings" / "style_packs"

PACKS = {
    "style_victorian": {
        "label": "Victorian",
        "usage_bias": ["residential", "commercial"],
        "style_tags": ["brick", "residential", "pitched_roof"],
        "slots": {
            "wall_1u": "wall_brick_1u",
            "wall_2u": "wall_brick_2u",
            "door_default": "door_residential",
            "window_1u": "win_single_1u",
            "window_2u": "win_double_1u",
            "roof_default": "roof_pitched_gable",
            "roof_flat": "roof_flat",
            "corner_outer": "corner_L",
            "prop_clutter": "prop_chimney",
        },
    },
    "style_modern": {
        "label": "Modern",
        "usage_bias": ["commercial", "office"],
        "style_tags": ["glass", "curtain", "flat_roof"],
        "slots": {
            "wall_1u": "wall_glass_curtain_1u",
            "wall_2u": "wall_industrial_panel_2u",
            "door_default": "door_office",
            "window_1u": "win_office_1u",
            "window_2u": "win_strip_2u",
            "window_industrial": "win_industrial_3u",
            "roof_default": "roof_flat",
            "roof_flat": "roof_flat",
            "prop_clutter": "prop_ac",
        },
    },
    "style_industrial_west": {
        "label": "Industrial West",
        "usage_bias": ["industrial", "warehouse"],
        "style_tags": ["steel", "sawtooth", "metal"],
        "slots": {
            "wall_1u": "wall_steel_1u",
            "wall_2u": "wall_concrete_2u",
            "door_default": "door_shop",
            "door_wide": "door_warehouse",
            "window_1u": "win_double_1u",
            "window_industrial": "win_industrial_3u",
            "roof_default": "roof_sawtooth",
            "roof_industrial": "roof_shed",
            "roof_flat": "roof_metal_low",
            "corner_outer": "corner_L",
            "prop_clutter": "prop_vent",
        },
    },
    "style_industrial_soviet": {
        "label": "Industrial Soviet",
        "usage_bias": ["industrial", "factory"],
        "style_tags": ["concrete", "panel", "flat"],
        "slots": {
            "wall_1u": "wall_concrete_1u",
            "wall_2u": "wall_concrete_2u",
            "door_default": "door_factory",
            "door_wide": "door_gate_industrial",
            "window_industrial": "win_industrial_3u",
            "roof_default": "roof_sawtooth",
            "roof_flat": "roof_flat",
            "prop_clutter": "prop_transformer",
        },
    },
    "style_military": {
        "label": "Military",
        "usage_bias": ["military", "bunker"],
        "style_tags": ["bunker", "concrete", "parapet"],
        "slots": {
            "wall_1u": "wall_military_bunker_1u",
            "wall_2u": "wall_concrete_2u",
            "door_default": "door_military",
            "window_1u": "win_bunker_slit",
            "roof_default": "roof_bunker",
            "roof_flat": "roof_parapet",
            "corner_outer": "corner_parapet",
            "prop_clutter": "prop_tank",
        },
    },
    "style_rural": {
        "label": "Rural",
        "usage_bias": ["residential", "farm"],
        "style_tags": ["wood", "rural", "pitched_roof"],
        "slots": {
            "wall_1u": "wall_wood_1u",
            "wall_2u": "wall_wood_2u",
            "door_default": "door_residential",
            "door_wide": "door_garage",
            "window_1u": "win_house_1u",
            "window_2u": "win_shop_2u",
            "roof_default": "roof_pitched_hip",
            "roof_flat": "roof_tile",
            "prop_clutter": "prop_fence",
        },
    },
    "style_colonial": {
        "label": "Colonial",
        "usage_bias": ["civic", "commercial"],
        "style_tags": ["brick", "colonial", "canopy"],
        "slots": {
            "wall_1u": "wall_brick_1u",
            "wall_2u": "wall_brick_2u",
            "door_default": "door_civic",
            "door_wide": "door_double_shop",
            "window_1u": "win_arched_1u",
            "window_2u": "win_shop_2u",
            "roof_default": "roof_pitched_gable",
            "roof_flat": "roof_canopy",
            "corner_outer": "corner_T",
        },
    },
}


def ron_str(value: str) -> str:
    escaped = value.replace("\\", "\\\\").replace('"', '\\"')
    return f'"{escaped}"'


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    index = json.loads((ROOT / "assets/configs/buildings/_module_index.json").read_text(encoding="utf-8"))
    lod0 = {
        e["module_id"]
        for e in index["entries"]
        if e.get("development_tier") == "lod0"
    }
    pack_ids: list[str] = []
    unresolved: list[dict[str, str]] = []
    slots_per_pack: dict[str, int] = {}

    for pack_id, pack in PACKS.items():
        pack_ids.append(pack_id)
        lines = [
            "(",
            "    schema_version: 1,",
            f"    style_pack_id: {ron_str(pack_id)},",
            f"    label: {ron_str(pack['label'])},",
            f"    usage_bias: [{', '.join(ron_str(x) for x in pack['usage_bias'])}],",
            f"    style_tags: [{', '.join(ron_str(x) for x in pack['style_tags'])}],",
            "    slots: (",
        ]
        for slot_key, module_id in pack["slots"].items():
            if module_id not in lod0:
                unresolved.append({"pack": pack_id, "slot": slot_key, "module_id": module_id})
            lines.append(f"        {slot_key}: {ron_str(module_id)},")
        lines.extend(["    ),", '    fallback_policy: "hide_slot",', ")", ""])
        (OUT / f"{pack_id}.ron").write_text("\n".join(lines), encoding="utf-8")
        slots_per_pack[pack_id] = len(pack["slots"])

    witness = {
        "pack_count": len(pack_ids),
        "pack_ids": pack_ids,
        "slots_per_pack": slots_per_pack,
        "unresolved_slots": unresolved,
        "lod0_module_refs": len(lod0),
        "green": len(unresolved) == 0,
    }
    witness_path = ROOT / "debug_runs/art_pipeline/style_packs_manifest_live.json"
    witness_path.parent.mkdir(parents=True, exist_ok=True)
    witness_path.write_text(json.dumps(witness, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(witness))


if __name__ == "__main__":
    main()
