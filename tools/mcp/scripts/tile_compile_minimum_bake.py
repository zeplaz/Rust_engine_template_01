#!/usr/bin/env python3
"""TILE-FIX-008/010 — bake minimum 24 G4 cells → atlas_meta v2 → promotion witness.

Requires production wall/roof GLBs promoted (building_definition modules) before ship green.

Usage:
  python tools/mcp/scripts/tile_compile_minimum_bake.py --building tools/mcp/schemas/examples/building_definition_warehouse_industrial_west_production_v1.json
  python tools/mcp/scripts/tile_compile_minimum_bake.py --plan-only
  python tools/mcp/scripts/tile_compile_minimum_bake.py --pack-only
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

_REPO = Path(__file__).resolve().parents[3]
if str(_REPO / "tools" / "mcp" / "python") not in sys.path:
    sys.path.insert(0, str(_REPO / "tools" / "mcp" / "python"))

from rust_engine_mcp.building_definition import production_shell_modules_ready, load_building_definition
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.tile_compile_loop import (
    pack_minimum_atlas_v2,
    run_minimum_cell_bakes,
    run_minimum_compile_pipeline,
    write_compile_plan_json,
)


def main() -> int:
    parser = argparse.ArgumentParser(description="TILE-FIX minimum G4 compile (24 cells)")
    parser.add_argument(
        "--building",
        default="tools/mcp/schemas/examples/building_definition_warehouse_industrial_west_production_v1.json",
        help="building_definition JSON path (repo-relative)",
    )
    parser.add_argument("--plan-only", action="store_true", help="Write compile plan JSON only")
    parser.add_argument("--bake-only", action="store_true", help="Bake 24 cell PNGs only")
    parser.add_argument("--pack-only", action="store_true", help="Pack existing PNGs to atlas_meta v2")
    parser.add_argument("--register", action="store_true", help="Register atlas in _tile_atlas_index.ron when green")
    parser.add_argument("--force-bake", action="store_true", help="Re-render even if PNG exists")
    args = parser.parse_args()

    bdef = repo_root() / args.building
    defn = load_building_definition(bdef)
    shell_ok, blockers = production_shell_modules_ready(defn)
    if not shell_ok:
        print("TILE-FIX-010 blocked — production shell modules not ready:")
        for b in blockers:
            print(f"  - {b}")
        print("Promote wall/roof production GLBs, update building_definition job_ids, then re-run.")
        if not args.plan_only:
            plan_path = write_compile_plan_json(bdef, minimum_only=True)
            print(f"Compile plan (blocked): {plan_path}")
            return 2

    if args.plan_only:
        path = write_compile_plan_json(bdef, minimum_only=True)
        print(json.dumps({"ok": True, "plan": str(path)}, indent=2))
        return 0

    if args.bake_only:
        result = run_minimum_cell_bakes(bdef, skip_existing=not args.force_bake)
        print(json.dumps(result, indent=2))
        return 0 if result.get("ok") else 1

    if args.pack_only:
        result = pack_minimum_atlas_v2(bdef)
        print(json.dumps(result, indent=2))
        return 0 if result.get("ok") else 1

    result = run_minimum_compile_pipeline(
        bdef,
        bake=True,
        pack=True,
        register_index=args.register,
    )
    print(json.dumps(result, indent=2))
    return 0 if result.get("ok") else 1


if __name__ == "__main__":
    raise SystemExit(main())
