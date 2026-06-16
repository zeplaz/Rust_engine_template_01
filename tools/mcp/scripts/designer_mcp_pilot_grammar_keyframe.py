#!/usr/bin/env python3
"""MCP-PILOT-GRAMMAR-001 — **CI / plumbing only** (NOT ship art).

Headless `blender_keyframe_light_rig` produces grey slabs — same class as minimum_bake.
Ship path requires **real** `utils/keyframe_render.py` in Blender UI + human G4 eyeball.

Policy: docs/archive/2026-06-src-dev/plans/mcp_orchestrator_tile_fix_warehouse_slice_v2.md
Checklist: docs/archive/2026-06-src-dev/plans/pilot_grammar_001_g4_checklist_v1.md
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

_REPO = Path(__file__).resolve().parents[3]
if str(_REPO / "tools" / "mcp" / "python") not in sys.path:
    sys.path.insert(0, str(_REPO / "tools" / "mcp" / "python"))

from rust_engine_mcp.building_definition import (
    MINIMUM_G4_CELLS,
    expand_bake_matrix_minimum,
    load_building_definition,
)
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.tile_compile_loop import (
    _staging_batch_id,
    pack_minimum_atlas_v2,
    run_designer_warehouse_phase_c,
    run_minimum_cell_bakes,
    write_compile_plan_json,
)

BDEF_DEFAULT = (
    "tools/mcp/schemas/examples/building_definition_warehouse_industrial_west_production_v1.json"
)
WITNESS = "debug_runs/art_pipeline/mcp_pilot_grammar_001_live.json"


def _write_keyframe_manual_marker(staging: Path, *, cell_count: int) -> Path:
    """Deprecated — do not use to bypass G4. Headless bakes must not claim manual ship."""
    raise RuntimeError(
        "keyframe_manual.export must come from Blender keyframe_render.py UI export, "
        "not headless tile_keyframe_bake — see pilot_grammar_001_g4_checklist_v1.md Phase 4"
    )


def _count_minimum_pngs(defn_path: Path, staging: Path) -> int:
    defn = load_building_definition(defn_path)
    cells = expand_bake_matrix_minimum(defn)
    from rust_engine_mcp.atlas_meta_v2_pack import cell_png_basename
    from rust_engine_mcp.tile_pipeline import _png_has_real_pixels

    return sum(
        1
        for cell in cells
        if _png_has_real_pixels(staging / cell_png_basename(cell))
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="MCP-PILOT-GRAMMAR-001 keyframe + G4 witness")
    parser.add_argument("--building", default=BDEF_DEFAULT)
    parser.add_argument("--skip-bake", action="store_true", help="Pack/G4 only (PNG folder must exist)")
    parser.add_argument("--force-bake", action="store_true", help="Re-render all 24 cells")
    args = parser.parse_args()

    os.environ.setdefault("RUST_ENGINE_TILE_KEYFRAME_HEADLESS", "1")
    os.environ.setdefault(
        "RUST_ENGINE_TILE_LIGHT_BLEND",
        str(repo_root() / "utils" / "Tile_iso_rig_v1.blend"),
    )

    bdef = repo_root() / args.building
    defn = load_building_definition(bdef)
    staging = repo_root() / "assets" / "staging" / "tiles" / _staging_batch_id(defn)

    plan_path = write_compile_plan_json(bdef, minimum_only=True)
    result: dict = {
        "slice_id": "MCP-PILOT-GRAMMAR-001",
        "compile_plan": str(plan_path.relative_to(repo_root())).replace("\\", "/"),
        "phases": {},
    }

    if not args.skip_bake:
        bake = run_minimum_cell_bakes(bdef, skip_existing=not args.force_bake)
        result["phases"]["4_keyframe_bake"] = bake
        if not bake.get("ok"):
            result["green"] = False
            result["proceed_ship"] = False
            result["blocked_by"] = ["keyframe_bake_failed"]
            _write_witness(result)
            print(json.dumps(result, indent=2))
            return 1

    png_count = _count_minimum_pngs(bdef, staging)
    result["phases"]["4_png_count"] = {"minimum_g4_cells": MINIMUM_G4_CELLS, "found": png_count}
    if png_count < MINIMUM_G4_CELLS:
        result["green"] = False
        result["proceed_ship"] = False
        result["blocked_by"] = ["minimum_pngs_incomplete"]
        _write_witness(result)
        print(json.dumps(result, indent=2))
        return 1

    marker = staging / "keyframe_manual.export"
    if marker.is_file():
        marker.unlink()

    result["phases"]["4_marker"] = "removed_fake_keyframe_manual_marker"
    result["phases"]["4_note"] = (
        "Headless PNGs are plumbing only — proceed_ship requires keyframe_render.py UI export"
    )

    pack = pack_minimum_atlas_v2(bdef)
    result["phases"]["5_pack"] = pack
    if not pack.get("ok"):
        result["green"] = False
        result["proceed_ship"] = False
        result["blocked_by"] = ["atlas_pack_failed"]
        _write_witness(result)
        print(json.dumps(result, indent=2))
        return 1

    # Promotion gates reject ship when HEADLESS env is still set — bake only.
    os.environ.pop("RUST_ENGINE_TILE_KEYFRAME_HEADLESS", None)

    phase_c = run_designer_warehouse_phase_c(bdef, require_manual_art=True)
    result["phases"]["6_phase_c"] = phase_c
    result["green"] = bool(phase_c.get("green"))
    result["proceed_ship"] = bool(phase_c.get("proceed_ship"))
    result["art_quality"] = phase_c.get("art_quality")
    result["minimum_g4_ship"] = phase_c.get("minimum_g4_ship")
    result["updated"] = datetime.now(timezone.utc).isoformat()
    _write_witness(result)
    print(json.dumps(result, indent=2))
    return 0 if result.get("green") else 1


def _write_witness(body: dict) -> None:
    out = repo_root() / WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    wrapped = {
        **body,
        "_agent_meta": {
            "agent": "designer-mcp",
            "lane": "MCP-PILOT-GRAMMAR-001",
            "policy": "docs/archive/2026-06-src-dev/plans/mcp_orchestrator_tile_fix_warehouse_slice_v2.md",
            "checklist": "docs/archive/2026-06-src-dev/plans/pilot_grammar_001_g4_checklist_v1.md",
        },
    }
    out.write_text(json.dumps(wrapped, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    raise SystemExit(main())
