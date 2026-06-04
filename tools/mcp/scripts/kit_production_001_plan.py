#!/usr/bin/env python3
"""kit_production_001 — Victorian rowhouse production pilot (5 modules, one style pack)."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / "tools/mcp/schemas/examples/batch_kit_production_001.manifest.json"

BATCH_ID = "kit_production_001"
TIER = "production"
STYLE = "style_victorian"
MAT = "brick_red_01"

# Rowhouse 4×3×2 assembly slots — MCP-PROD-PILOT-ROWHOUSE-001
MODULES = [
    ("wall_brick_1u", "brick", "wall_1u"),
    ("corner_L", "corner_l", "corner_outer"),
    ("door_residential", "residential", "door_default"),
    ("roof_pitched_gable", "pitched", "roof_default"),
    ("prop_chimney", "chimney", "prop_clutter"),
]

manifest = {
    "schema_version": 1,
    "batch_id": BATCH_ID,
    "development_tier": TIER,
    "pilot_scope": "MCP-PROD-PILOT-ROWHOUSE-001",
    "pilot_archetype": "rowhouse",
    "pilot_style_pack_id": STYLE,
    "plan_ref": "src/dev/mcp_fleet_production_pilot_rowhouse_v1.md",
    "wave": "001",
    "module_count": len(MODULES),
    "modules": [
        {
            "module_id": mid,
            "asset_id": mid,
            "job_id": f"{mid}_production_run001" if mid != "corner_L" else "corner_L_production_run001",
            "style_pack_id": STYLE,
            "material_id": MAT if mid != "door_residential" else "wood_plank_01",
            "profile": profile,
            "pbr_status": "shipped",
            "development_tier": TIER,
            "assembly_slot": slot,
            "status": "spec_ready",
        }
        for mid, profile, slot in MODULES
    ],
    "rules_applied": [
        "no_ai_generated_images",
        "deterministic_output",
        "batch_processing",
        "tier_production_pbr_shipped",
    ],
    "witness": "debug_runs/art_pipeline/kit_production_001_live.json",
    "note": "Orchestrator slice: Victorian rowhouse only — not multi-style wall sweep.",
}

if __name__ == "__main__":
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote {OUT}")
