#!/usr/bin/env python3
"""TILE-FIX-WAREHOUSE-MIN-G4 Phase B — shell GLBs, bdef/snapshot, material gate."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools" / "mcp" / "python"))

from rust_engine_mcp import blender_runner, promote
from rust_engine_mcp.material_textures import generate_profile, PILOT_PROFILES
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.tile_pipeline import assembly_build_run
from rust_engine_mcp.validators.material_textures import validate_material_textures

JOBS = ROOT / "tools/mcp/schemas/examples"
BDEF = JOBS / "building_definition_warehouse_industrial_west_production_v1.json"
SNAP = JOBS / "assembly_snapshot_warehouse_industrial_west_production_v1.json"

SHELL_JOBS = [
    ("wall_steel_1u", "wall_steel_1u_production_run001", "wall_steel_1u_lod0_run001"),
    ("roof_sawtooth", "roof_sawtooth_production_run001", "roof_sawtooth_lod0_run001"),
]
MATERIAL_PROFILES = ["steel_panel_01", "roof_metal_01"]


def ensure_material_pngs() -> dict[str, Any]:
    missing_before = []
    for pid in MATERIAL_PROFILES:
        tex_dir = repo_root() / "assets/materials/textures" / pid
        for name in ("albedo", "normal", "roughness"):
            if not (tex_dir / f"{name}.png").is_file():
                missing_before.append(f"{pid}/{name}.png")
    if "steel_panel_01" in missing_before or any("steel_panel" in m for m in missing_before):
        if "steel_panel_01" in PILOT_PROFILES:
            generate_profile(PILOT_PROFILES["steel_panel_01"])
    errors = []
    for pid in MATERIAL_PROFILES:
        rep = validate_material_textures(
            {"development_tier": "production", "material_profile": pid},
            ship=True,
        )
        if rep.status == "failed":
            errors.extend(e.hint for e in rep.errors if e.severity == "error")
    if errors:
        raise RuntimeError(f"TILE-FIX-005 material gate failed: {errors}")
    return {"ok": True, "generated": missing_before}


def run_shell_geometry() -> list[dict[str, Any]]:
    results = []
    for _mid, prod_jid, _lod in SHELL_JOBS:
        job_path = JOBS / f"{prod_jid}.json"
        if not job_path.is_file():
            raise FileNotFoundError(job_path)
        r = blender_runner.run_geometry_job(job_path)
        glb = ROOT / "assets/staging" / prod_jid / "model.glb"
        results.append({"job_id": prod_jid, "status": r.status, "glb_bytes": glb.stat().st_size if glb.is_file() else 0})
        if r.status != "done" or not glb.is_file():
            raise RuntimeError(f"geometry_run_job failed {prod_jid}: {r.status} {r.error}")
    return results


def promote_shell() -> list[dict[str, Any]]:
    out = []
    for _mid, prod_jid, _lod in SHELL_JOBS:
        out.append(promote.promote_module(prod_jid, register=True))
    return out


def patch_bdef_and_snapshot() -> dict[str, str]:
    bdef = json.loads(BDEF.read_text(encoding="utf-8"))
    for mod in bdef.get("modules") or []:
        mid = str(mod.get("module_id") or "")
        for shell_mid, prod_jid, lod_jid in SHELL_JOBS:
            if mid == shell_mid:
                mod["job_id"] = prod_jid
    BDEF.write_text(json.dumps(bdef, indent=2) + "\n", encoding="utf-8")

    snap = json.loads(SNAP.read_text(encoding="utf-8"))
    for placement in snap.get("module_placements") or []:
        for shell_mid, prod_jid, lod_jid in SHELL_JOBS:
            if str(placement.get("module_id")) == shell_mid or str(placement.get("job_id")) == lod_jid:
                placement["job_id"] = prod_jid
                placement["glb_path"] = f"assets/models/modules/{prod_jid}/model.glb"
    SNAP.write_text(json.dumps(snap, indent=2) + "\n", encoding="utf-8")

    build = assembly_build_run(SNAP)
    return {
        "building_definition": str(BDEF.relative_to(ROOT)).replace("\\", "/"),
        "assembly_snapshot": str(SNAP.relative_to(ROOT)).replace("\\", "/"),
        "assembly_build": build,
    }


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--materials-only", action="store_true")
    p.add_argument("--skip-geometry", action="store_true")
    p.add_argument("--skip-promote", action="store_true")
    p.add_argument("--patch-only", action="store_true")
    args = p.parse_args()

    out: dict[str, Any] = {"phase": "B"}
    out["materials"] = ensure_material_pngs()
    if args.materials_only:
        print(json.dumps(out, indent=2))
        return 0

    if not args.skip_geometry and not args.patch_only:
        out["geometry"] = run_shell_geometry()
    if not args.skip_promote and not args.patch_only:
        out["promote"] = promote_shell()

    out["patch"] = patch_bdef_and_snapshot()
    print(json.dumps(out, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
