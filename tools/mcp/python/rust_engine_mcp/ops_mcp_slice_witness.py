"""OPS slice witnesses — honest paused/done states for phase3 MCP ops rows."""

from __future__ import annotations

import json
from typing import Any

from .paths import repo_root

OPS_007_WITNESS_PATH = "debug_runs/ops_007_warehouse_production_pause_live.json"


def write_ops_007_warehouse_production_pause_witness() -> dict[str, Any]:
    root = repo_root()
    variant_matrix = root / "debug_runs/art_pipeline/variant_matrix_warehouse_v1.yaml"
    tile_batch = root / "tools/mcp/schemas/examples/tile_batch_warehouse_industrial_west_production_v1.json"
    variant_set = root / "tools/mcp/schemas/examples/variant_set_warehouse_industrial_west_production_v1.json"
    assembly = root / "tools/mcp/schemas/examples/assembly_snapshot_warehouse_industrial_west_production_v1.json"

    body: dict[str, Any] = {
        "ops_id": "OPS-007",
        "task_id": "OPS-007",
        "program_id": "art_B",
        "owner": "@coder-mcp",
        "status": "paused",
        "ok": True,
        "green": False,
        "proceed_ship": False,
        "honest_gate": "track_b_paused",
        "blocked_by": [
            "WH-TRACK-B-PAUSE",
            "MCP-PILOT-GRAMMAR-001",
            "variant_matrix_warehouse_v1 frozen in mcp_active_queue.json",
            "tile_batch_warehouse_industrial_west_production_v1 frozen",
            "manual_keyframe_render_required",
            "human_g4_rejected",
        ],
        "variant_matrix": "debug_runs/art_pipeline/variant_matrix_warehouse_v1.yaml",
        "variant_matrix_exists": variant_matrix.is_file(),
        "tile_batch": "tools/mcp/schemas/examples/tile_batch_warehouse_industrial_west_production_v1.json",
        "tile_batch_exists": tile_batch.is_file(),
        "variant_set": "tools/mcp/schemas/examples/variant_set_warehouse_industrial_west_production_v1.json",
        "variant_set_exists": variant_set.is_file(),
        "assembly_snapshot": "tools/mcp/schemas/examples/assembly_snapshot_warehouse_industrial_west_production_v1.json",
        "assembly_snapshot_exists": assembly.is_file(),
        "required_key_count": 15,
        "bake_source": "keyframe_pack",
        "ship": False,
        "resume_steps": [
            "Operator: utils/keyframe_render.py → assets/staging/tiles/keyframe_stills/warehouse_industrial/",
            "@designer-mcp: G4 sign-off per pilot_grammar_001_g4_checklist_v1.md",
            "Unfreeze tile_batch_warehouse_industrial_west_production_v1 in mcp_active_queue.json",
            "python -m rust_engine_mcp.cli tile-batch-run tools/mcp/schemas/examples/tile_batch_warehouse_industrial_west_production_v1.json",
            "python -m rust_engine_mcp.cli tile-atlas-pack assets/staging/tiles/warehouse_industrial -pk",
            "validate-report tile_promotion_honest before promote",
        ],
        "related_pilot": {
            "note": "Separate pilot path (ship=false) — tile_rail_warehouse_pilot_v1 batch contract only",
            "witness": "debug_runs/tile_rail_warehouse_pilot_batch_live.json",
        },
        "docs": [
            "docs/archive/2026-06-src-dev/plans/mcp_orchestrator_tile_fix_warehouse_slice_v2.md",
            "docs/archive/2026-06-src-dev/plans/pilot_grammar_001_g4_checklist_v1.md",
            "tools/orchestrator/queues/mcp_active_queue.json",
        ],
    }
    out = root / OPS_007_WITNESS_PATH
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body
