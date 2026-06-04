"""BUILD-WORKER-001 — snapshot material_profile authority → PBR textures → assembly blend → render still."""

from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Any

from . import assembly
from .material_textures import PILOT_PROFILES, generate_profile, write_registry
from .paths import jobs_root, repo_root
from .tile_pipeline import assembly_build_run

BUILD_WORKER_WITNESS_JSON = "debug_runs/build_worker_001_live.json"
DEFAULT_BDEF = (
    "tools/mcp/schemas/examples/building_definition_warehouse_industrial_west_production_v1.json"
)

def material_profiles_in_snapshot(snapshot: dict[str, Any]) -> list[str]:
    seen: set[str] = set()
    for row in snapshot.get("module_placements") or []:
        pid = str((row or {}).get("material_profile") or "").strip()
        if pid:
            seen.add(pid)
    return sorted(seen)


def ensure_snapshot_material_textures(
    snapshot: dict[str, Any],
    *,
    size: int = 512,
) -> dict[str, Any]:
    """Generate missing PBR PNGs for every material_profile on placements."""
    write_registry()
    profiles = material_profiles_in_snapshot(snapshot)
    generated: list[str] = []
    missing: list[str] = []
    for pid in profiles:
        tex_dir = repo_root() / "assets" / "materials" / "textures" / pid
        albedo = tex_dir / "albedo.png"
        if albedo.is_file():
            continue
        if pid in PILOT_PROFILES:
            generate_profile(PILOT_PROFILES[pid], size=size)
            generated.append(pid)
        else:
            missing.append(pid)
    return {
        "profiles_requested": profiles,
        "generated": generated,
        "missing_pilot": missing,
        "ok": not missing,
    }


def assembly_build_with_materials(
    snapshot_path: str | Path,
    *,
    ensure_textures: bool = True,
    render_still: bool = False,
    building_definition_path: str | Path | None = None,
    write_witness: bool = True,
) -> dict[str, Any]:
    """BUILD-WORKER-001 — ensure textures → assembly_import → optional render still."""
    raw = Path(snapshot_path)
    snap_path = raw if raw.is_file() else repo_root() / raw
    snapshot = assembly.load_assembly_snapshot(snap_path)
    assembly_id = str(snapshot.get("assembly_id") or snap_path.stem)

    mat_result: dict[str, Any] = {"skipped": True}
    if ensure_textures:
        mat_result = ensure_snapshot_material_textures(snapshot)
        if not mat_result.get("ok"):
            return {
                "gate_id": "BUILD-WORKER-001",
                "ok": False,
                "status": "missing_material_profiles",
                "assembly_id": assembly_id,
                "materials": mat_result,
            }

    build = assembly_build_run(snap_path)
    body: dict[str, Any] = {
        "gate_id": "BUILD-WORKER-001",
        "ok": bool(build.get("ok")),
        "status": build.get("status"),
        "assembly_id": assembly_id,
        "snapshot": _rel(snap_path),
        "blend_path": build.get("blend_path"),
        "materials": mat_result,
        "build": build,
        "authority": "snapshot material_profile → assembly_import apply_material_profile_to_meshes",
    }
    if render_still and body.get("ok"):
        body["render"] = render_worker_still(
            building_definition_path=building_definition_path,
            blend_path=build.get("blend_path"),
            snapshot_path=snap_path,
        )
        if body["render"].get("preview_png"):
            body["preview_png"] = body["render"]["preview_png"]
    if write_witness:
        write_build_worker_witness(body)
    return body


def render_worker_still(
    *,
    building_definition_path: str | Path | None = None,
    blend_path: str | Path | None = None,
    snapshot_path: str | Path | None = None,
    require_blender: bool = False,
) -> dict[str, Any]:
    """Optional render leg — first minimum G4 keyframe cell or trimesh assembly thumbnail."""
    bdef_raw = Path(building_definition_path or DEFAULT_BDEF)
    bdef_path = bdef_raw if bdef_raw.is_file() else repo_root() / bdef_raw
    out_png = repo_root() / "debug_runs" / "build_worker_001_preview.png"
    out_png.parent.mkdir(parents=True, exist_ok=True)

    headless = os.environ.get("RUST_ENGINE_TILE_KEYFRAME_HEADLESS", "").strip() in ("1", "true", "yes")
    if bdef_path.is_file() and (headless or require_blender):
        from dataclasses import replace

        from .building_definition import expand_bake_matrix_minimum, load_building_definition
        from .tile_compile_loop import keyframe_job_for_cell, light_setup_blend_path
        from .tile_pipeline import _run_tile_job_path, tile_keyframe_headless_enabled

        if require_blender and not tile_keyframe_headless_enabled():
            return {
                "ok": False,
                "status": "keyframe_headless_disabled",
                "hint": "Set RUST_ENGINE_TILE_KEYFRAME_HEADLESS=1",
            }
        defn = load_building_definition(bdef_path)
        if blend_path:
            blend_rel = str(blend_path).replace("\\", "/")
            if Path(blend_rel).is_absolute():
                blend_rel = _rel(Path(blend_rel))
            defn = replace(defn, assembly_blend=blend_rel)
        cells = expand_bake_matrix_minimum(defn)
        if not cells:
            return {"ok": False, "status": "no_bake_cells"}
        cell = cells[0]
        light_rel = str(os.environ.get("RUST_ENGINE_TILE_LIGHT_BLEND") or light_setup_blend_path())
        if not Path(light_rel).is_absolute():
            light_rel = str((repo_root() / light_rel).resolve().relative_to(repo_root())).replace(
                "\\", "/"
            )
        job = keyframe_job_for_cell(defn, cell, light_blend=light_rel)
        job_path = jobs_root() / f"{job['job_id']}.json"
        job_path.write_text(json.dumps(job, indent=2) + "\n", encoding="utf-8")
        result = _run_tile_job_path(job_path)
        png_rel = job.get("output", {}).get("png", "")
        png_path = repo_root() / str(png_rel).replace("\\", "/") if png_rel else out_png
        return {
            "ok": result.status == "done" and png_path.is_file(),
            "status": result.status,
            "method": "keyframe_minimum_cell",
            "variant_key": cell.variant_key,
            "facing": cell.facing,
            "preview_png": _rel(png_path) if png_path.is_file() else "",
            "job_id": job["job_id"],
            "error": result.error,
        }

    if snapshot_path:
        from .assembly_preview import collect_preview_placements, try_render_thumbnail_png

        snap_raw = Path(snapshot_path)
        snap_path = snap_raw if snap_raw.is_file() else repo_root() / snap_raw
        snapshot = assembly.load_assembly_snapshot(snap_path)
        placements, _missing = collect_preview_placements(snapshot)
        ok = try_render_thumbnail_png(placements, out_png)
        return {
            "ok": ok,
            "status": "trimesh_thumbnail" if ok else "trimesh_unavailable",
            "method": "assembly_preview_trimesh",
            "preview_png": _rel(out_png) if ok else "",
        }

    return {"ok": False, "status": "render_skipped", "hint": "Provide snapshot_path or enable keyframe headless"}


def write_build_worker_witness(result: dict[str, Any]) -> Path:
    out = repo_root() / BUILD_WORKER_WITNESS_JSON
    out.parent.mkdir(parents=True, exist_ok=True)
    slim = {k: v for k, v in result.items() if k != "build" or result.get("ok")}
    out.write_text(json.dumps(slim, indent=2) + "\n", encoding="utf-8")
    return out


def _rel(path: Path) -> str:
    try:
        return path.relative_to(repo_root()).as_posix()
    except ValueError:
        return path.as_posix()
