#!/usr/bin/env python3
"""Write debug_runs/art_pipeline/pbr_pilot_rowhouse_live.json (MCP-PROD-PBR-PILOT)."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / "debug_runs" / "art_pipeline" / "pbr_pilot_rowhouse_live.json"

from rust_engine_mcp.validators.tier import (  # noqa: E402
    PRODUCTION_TILEABLE_SET_IDS,
    tier_issues_for_spec,
)
from rust_engine_mcp.schemas import load_json_file  # noqa: E402

SPECS = [
    "assets/staging/specs/wall_brick_1u_production.json",
    "assets/staging/specs/corner_L_production.json",
    "assets/staging/specs/door_residential_production.json",
    "assets/staging/specs/roof_pitched_gable_production.json",
    "assets/staging/specs/prop_chimney_production.json",
]

JOBS = [
    "tools/mcp/schemas/examples/wall_brick_1u_production_run001.json",
    "tools/mcp/schemas/examples/corner_L_production_run001.json",
    "tools/mcp/schemas/examples/door_residential_production_run001.json",
    "tools/mcp/schemas/examples/roof_pitched_gable_production_run001.json",
    "tools/mcp/schemas/examples/prop_chimney_production_run001.json",
]


def _spec_rows() -> list[dict]:
    rows = []
    for rel in SPECS:
        path = ROOT / rel
        spec = load_json_file(path)
        issues = tier_issues_for_spec(spec, path)
        rows.append(
            {
                "spec": rel,
                "asset_id": spec.get("asset_id"),
                "pbr_status": spec.get("pbr_status"),
                "material_profile": spec.get("material_profile"),
                "tier_ok": not any(i.severity == "error" for i in issues),
                "issues": [i.hint for i in issues if i.severity == "error"],
            }
        )
    return rows


def main() -> None:
    OUT.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "witness_id": "pbr_pilot_rowhouse_live",
        "sprint": "MCP-PROD-PBR-PILOT",
        "batch_id": "kit_production_001",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "doc_ref": "src/dev/pbr_pilot_rowhouse_material_maker_v1.md",
        "tileable_allowlist": sorted(PRODUCTION_TILEABLE_SET_IDS),
        "production_specs": _spec_rows(),
        "production_jobs": [str(p).replace("\\", "/") for p in JOBS],
        "promote_gate": {
            "pbr_status": "shipped",
            "material_profile_required": True,
            "enforce_on": "validate_asset_glb + promote_module",
        },
        "bpy_profiles": {
            "module_wall": ["brick", "recess", "flat"],
            "module_door": ["residential", "frame"],
            "module_window": ["mullion", "arched"],
            "module_roof": ["pitched", "shed", "sawtooth"],
            "module_prop": ["corner_l", "chimney", "vent"],
        },
    }
    OUT.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote {OUT}")


if __name__ == "__main__":
    main()
