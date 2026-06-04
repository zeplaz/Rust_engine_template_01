"""TILE-FIX-008/010 — Blender compile: materials → variant → facing → render → pack."""

from __future__ import annotations

import json
import os
from dataclasses import asdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from rust_engine_mcp.atlas_meta_v2_pack import (
    cell_png_basename,
    pack_cells_to_atlas,
    write_atlas_meta_v2,
)
from rust_engine_mcp.building_definition import (
    MINIMUM_G4_CELLS,
    BakeCell,
    BuildingDefinition,
    expand_bake_matrix,
    expand_bake_matrix_minimum,
    load_building_definition,
    production_shell_modules_ready,
)
from rust_engine_mcp.paths import jobs_root, repo_root
from rust_engine_mcp.witness import art_pipeline_dir
from rust_engine_mcp.tile_pipeline import (
    _png_has_real_pixels,
    _run_tile_job_path,
    light_setup_blend_path,
    tile_dry_run_enabled,
    tile_keyframe_headless_enabled,
)
from rust_engine_mcp.validators.assembly_production import validate_assembly_snapshot_path
from rust_engine_mcp.validators.material_textures import validate_material_textures
from rust_engine_mcp.validators.tile_promotion import validate_tile_promotion


def _facing_yaw_deg(facing: int, facings: int) -> float:
    step = 360.0 / max(facings, 1)
    return float(facing) * step


def compile_plan(defn: BuildingDefinition, *, minimum_only: bool = False) -> dict[str, Any]:
    """Structured bake plan (no Blender side effects) for agents and CI."""
    cells = expand_bake_matrix_minimum(defn) if minimum_only else expand_bake_matrix(defn)
    steps = []
    for cell in cells:
        steps.append(
            {
                "phase": "render",
                "variant_key": cell.variant_key,
                "facing": cell.facing,
                "frame": cell.frame,
                "yaw_deg": _facing_yaw_deg(cell.facing, defn.facings),
                "variant_params": cell.variant_params,
                "output_png": (
                    f"assets/staging/tiles/keyframe_stills/{defn.building_id}/"
                    f"{cell_png_basename(cell)}"
                ),
            }
        )
    return {
        "building_id": defn.building_id,
        "assembly_blend": defn.assembly_blend,
        "assembly_snapshot": defn.assembly_snapshot,
        "facings": defn.facings,
        "cell_count": len(cells),
        "minimum_g4": minimum_only,
        "minimum_g4_cells": MINIMUM_G4_CELLS if minimum_only else None,
        "pipeline": [
            "validate_materials",
            "validate_assembly_snapshot",
            "validate_production_shell_modules",
            "assembly_import",
            "apply_materials",
            "for variant in variants",
            "  for facing in facings",
            "    keyframe_render",
            "pack_minimum_atlas_v2",
            "tile_promotion_gates",
        ],
        "steps": steps,
    }


def validate_compile_preconditions(defn: BuildingDefinition, *, ship: bool = True) -> list[str]:
    errors: list[str] = []
    for profile in defn.material_profiles or []:
        rep = validate_material_textures(
            {"development_tier": "production", "material_profile": profile},
            ship=ship,
        )
        if rep.status == "failed":
            errors.extend(e.hint for e in rep.errors if e.severity == "error")
    if defn.assembly_snapshot:
        snap_path = repo_root() / defn.assembly_snapshot
        rep = validate_assembly_snapshot_path(snap_path, ship=ship)
        if rep.status == "failed":
            errors.extend(e.hint for e in rep.errors if e.severity == "error")
    blend = repo_root() / defn.assembly_blend if defn.assembly_blend else None
    if ship and blend and not blend.is_file():
        errors.append(f"missing assembly blend: {defn.assembly_blend}")
    shell_ok, blockers = production_shell_modules_ready(defn)
    if ship and not shell_ok:
        errors.extend(blockers)
    return errors


def write_compile_plan_json(
    defn_path: str | Path,
    out_path: str | Path | None = None,
    *,
    minimum_only: bool = False,
) -> Path:
    defn = load_building_definition(defn_path)
    pre = validate_compile_preconditions(defn, ship=True)
    plan = compile_plan(defn, minimum_only=minimum_only)
    plan["precondition_errors"] = pre
    plan["preconditions_ok"] = not pre
    out = out_path or (
        repo_root()
        / "debug_runs/art_pipeline"
        / f"{defn.building_id}_compile_plan_v1.json"
    )
    out = Path(out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(plan, indent=2) + "\n", encoding="utf-8")
    return out


def keyframe_job_for_cell(defn: BuildingDefinition, cell: BakeCell, *, light_blend: str = "") -> dict[str, Any]:
    """Single headless ``tile_variant_bake`` job (keyframe light rig when configured)."""
    png_rel = (
        f"assets/staging/tiles/{_staging_batch_id(defn)}/"
        f"{cell_png_basename(cell)}"
    )
    job: dict[str, Any] = {
        "schema_version": 1,
        "job_id": f"tile_{defn.building_id}_{cell.variant_key}_f{cell.facing}"[:120],
        "operation": "tile_variant_bake",
        "mode": "assembly",
        "assembly_blend": defn.assembly_blend,
        "variant": {**cell.variant_params, "variant_key": cell.variant_key},
        "render": {
            "method": "blender_keyframe_light_rig",
            "seed": 43,
            "tile_size_px": defn.tile_px,
            "camera_elevation_deg": 35.264,
            "facing": cell.facing,
            "facing_yaw_deg": _facing_yaw_deg(cell.facing, defn.facings),
        },
        "output": {"png": png_rel},
    }
    if light_blend:
        job["light_blend"] = light_blend
    return job


def _staging_batch_id(defn: BuildingDefinition) -> str:
    return f"tile_{defn.building_id}_v2_minimum_g4"


def _default_warehouse_v2_batch(defn: BuildingDefinition) -> dict[str, Any]:
    return {
        "schema_version": 1,
        "batch_id": _staging_batch_id(defn),
        "tile_id": defn.building_id,
        "atlas_id": "warehouse_industrial_west_v2",
        "development_tier": "production",
        "source_tier": "production",
        "ship": False,
        "dry_run": False,
        "bake_source": "keyframe_pack",
        "atlas_schema_version": 2,
        "building_definition": (
            "tools/mcp/schemas/examples/building_definition_warehouse_industrial_west_production_v1.json"
        ),
        "visual_config_ref": "assets/configs/buildings/visual_config_warehouse_industrial_west_v2.json",
        "render_contract": defn.render_contract,
        "render": {
            "method": "blender_keyframe_light_rig",
            "tile_size_px": defn.tile_px,
            "seed": 43,
        },
        "atlas": {
            "atlas_id": "warehouse_industrial_west_v2",
            "meta_json": f"assets/staging/tiles/{_staging_batch_id(defn)}/atlas_meta.json",
            "output_png": (
                f"assets/staging/tiles/{_staging_batch_id(defn)}/"
                f"{defn.building_id}_west_v2_atlas.png"
            ),
            "columns": 8,
            "rows": 3,
            "tile_px": defn.tile_px,
        },
    }


def run_minimum_cell_bakes(
    defn_path: str | Path,
    *,
    skip_existing: bool = True,
    require_blender: bool = True,
) -> dict[str, Any]:
    """Bake MINIMUM_G4_CELLS (24) stills via headless keyframe rig."""
    defn = load_building_definition(defn_path)
    pre = validate_compile_preconditions(defn, ship=True)
    if pre:
        return {"ok": False, "status": "preconditions_failed", "errors": pre}

    if require_blender and not tile_keyframe_headless_enabled() and not tile_dry_run_enabled():
        return {
            "ok": False,
            "status": "keyframe_headless_disabled",
            "hint": "Set RUST_ENGINE_TILE_KEYFRAME_HEADLESS=1 (and ensure Blender on PATH)",
        }

    cells = expand_bake_matrix_minimum(defn)
    if len(cells) != MINIMUM_G4_CELLS:
        return {
            "ok": False,
            "status": "minimum_cell_count_mismatch",
            "expected": MINIMUM_G4_CELLS,
            "got": len(cells),
        }

    staging = repo_root() / "assets" / "staging" / "tiles" / _staging_batch_id(defn)
    staging.mkdir(parents=True, exist_ok=True)
    light_rel = str(
        os.environ.get("RUST_ENGINE_TILE_LIGHT_BLEND") or light_setup_blend_path()
    )
    if not Path(light_rel).is_absolute():
        light_rel = str((repo_root() / light_rel).resolve().relative_to(repo_root())).replace(
            "\\", "/"
        )

    bake_results: list[dict[str, Any]] = []
    for cell in cells:
        dest = staging / cell_png_basename(cell)
        if skip_existing and _png_has_real_pixels(dest):
            bake_results.append(
                {"variant_key": cell.variant_key, "facing": cell.facing, "status": "skipped"}
            )
            continue
        job = keyframe_job_for_cell(defn, cell, light_blend=light_rel)
        job_path = jobs_root() / f"{job['job_id']}.json"
        job_path.write_text(json.dumps(job, indent=2) + "\n", encoding="utf-8")
        result = _run_tile_job_path(job_path)
        bake_results.append(
            {
                "job_id": job["job_id"],
                "variant_key": cell.variant_key,
                "facing": cell.facing,
                "status": result.status,
                "error": result.error,
            }
        )
        if result.status != "done":
            return {
                "ok": False,
                "status": "tile_keyframe_bake_failed",
                "failed_job": job["job_id"],
                "bake_results": bake_results,
            }

    return {
        "ok": True,
        "status": "minimum_g4_bake_done",
        "cell_count": len(cells),
        "staging": str(staging.relative_to(repo_root())).replace("\\", "/"),
        "bake_results": bake_results,
    }


def pack_minimum_atlas_v2(
    defn_path: str | Path,
    *,
    batch: dict[str, Any] | None = None,
) -> dict[str, Any]:
    """Pack 24 facing stills → atlas.png + atlas_meta v2 with full minimum lookups."""
    defn = load_building_definition(defn_path)
    cells = expand_bake_matrix_minimum(defn)
    if len(cells) != MINIMUM_G4_CELLS:
        return {"ok": False, "status": "minimum_cell_count_mismatch", "got": len(cells)}

    batch_body = batch or _default_warehouse_v2_batch(defn)
    staging = repo_root() / "assets" / "staging" / "tiles" / _staging_batch_id(defn)
    atlas_rel = str(batch_body.get("atlas", {}).get("output_png") or "")
    atlas_path = repo_root() / atlas_rel if atlas_rel else staging / "tile_map_minimum_g4.png"
    if not atlas_rel:
        atlas_path = staging / "tile_map_minimum_g4.png"
        atlas_rel = str(atlas_path.relative_to(repo_root())).replace("\\", "/")

    try:
        pack_info = pack_cells_to_atlas(
            cells,
            staging,
            atlas_png=atlas_path,
            tile_px=defn.tile_px,
            columns=int(batch_body.get("atlas", {}).get("columns") or 8),
        )
    except FileNotFoundError as exc:
        return {"ok": False, "status": "minimum_pngs_missing", "error": str(exc)}

    vc_rel = str(batch_body.get("visual_config_ref") or "")
    meta_path = write_atlas_meta_v2(
        batch=batch_body,
        pack_info=pack_info,
        atlas_png_rel=atlas_rel,
        visual_config_rel=vc_rel,
        minimum_g4_ship=True,
    )

    from rust_engine_mcp.validators.atlas_meta import validate_atlas_meta_v2

    meta_rep = validate_atlas_meta_v2(
        meta_path,
        visual_config_path=repo_root() / vc_rel if vc_rel else None,
    )
    promo_rep = validate_tile_promotion(
        building_definition_path=defn_path,
        batch=batch_body,
        meta_path=meta_path,
        staging_dir=staging,
        ship=True,
    )

    status_body = {
        "batch_id": batch_body.get("batch_id"),
        "status": "done",
        "bake_source": "keyframe_pack",
        "minimum_g4_cells": len(cells),
        "lookup_count": len(pack_info["lookups"]),
        "atlas_path": atlas_rel,
        "meta_json": str(meta_path.relative_to(repo_root())).replace("\\", "/"),
        "dry_run": tile_dry_run_enabled(),
        "atlas_meta_v2_valid": meta_rep.status == "passed",
        "promotion_valid": promo_rep.status == "passed",
    }
    (staging / "batch_status.json").write_text(
        json.dumps(status_body, indent=2) + "\n", encoding="utf-8"
    )

    return {
        "ok": meta_rep.status == "passed",
        "status": "minimum_g4_pack_done",
        "atlas_png": atlas_rel,
        "meta_json": str(meta_path.relative_to(repo_root())).replace("\\", "/"),
        "lookup_count": len(pack_info["lookups"]),
        "atlas_meta_validation": meta_rep.to_dict(),
        "promotion_validation": promo_rep.to_dict(),
        "batch_status": status_body,
    }


def run_minimum_compile_pipeline(
    defn_path: str | Path,
    *,
    bake: bool = True,
    pack: bool = True,
    register_index: bool = False,
) -> dict[str, Any]:
    """Full TILE-FIX-008 minimum path: bake 24 → pack v2 → promotion witness."""
    defn = load_building_definition(defn_path)
    out: dict[str, Any] = {
        "building_id": defn.building_id,
        "minimum_g4_cells": MINIMUM_G4_CELLS,
        "shell_production_ready": production_shell_modules_ready(defn)[0],
    }
    if bake:
        out["bake"] = run_minimum_cell_bakes(defn_path)
        if not out["bake"].get("ok"):
            return {**out, "ok": False, "status": out["bake"].get("status")}
    if pack:
        out["pack"] = pack_minimum_atlas_v2(defn_path)
        if not out["pack"].get("ok"):
            return {**out, "ok": False, "status": out["pack"].get("status")}

    if register_index and out.get("pack", {}).get("meta_json"):
        try:
            from rust_engine_mcp.tile_index import register_tile_atlas_from_meta

            meta = repo_root() / str(out["pack"]["meta_json"])
            batch = _default_warehouse_v2_batch(defn)
            batch["ship"] = out.get("pack", {}).get("promotion_validation", {}).get("status") == "passed"
            out["tile_index"] = register_tile_atlas_from_meta(meta, batch=batch)
        except Exception as exc:  # noqa: BLE001
            out["tile_index"] = {"ok": False, "error": str(exc)}

    promo_status = (out.get("pack") or {}).get("promotion_validation", {}).get("status")
    witness = write_tile_fix_10_witness(
        defn_path,
        pipeline_summary={
            "bake_status": (out.get("bake") or {}).get("status"),
            "pack_status": (out.get("pack") or {}).get("status"),
            "promotion_status": promo_status,
            "lookup_count": (out.get("pack") or {}).get("lookup_count"),
        },
    )
    out["witness"] = {
        "green": witness.get("green"),
        "written": witness.get("written"),
        "promotion_status": promo_status,
    }
    out["ok"] = promo_status == "passed"
    out["status"] = "tile_fix_10_green" if out["ok"] else "tile_fix_10_blocked"
    return out


def write_tile_fix_10_witness(
    defn_path: str | Path,
    *,
    pipeline_summary: dict[str, Any] | None = None,
) -> dict[str, Any]:
    defn = load_building_definition(defn_path)
    shell_ok, shell_blockers = production_shell_modules_ready(defn)
    staging = repo_root() / "assets" / "staging" / "tiles" / _staging_batch_id(defn)
    meta_path = staging / "atlas_meta.json"
    from rust_engine_mcp.validators.tile_promotion import validate_tile_promotion

    batch = _default_warehouse_v2_batch(defn)
    batch["dry_run"] = False
    promo_rep = validate_tile_promotion(
        building_definition_path=defn_path,
        batch=batch,
        meta_path=meta_path if meta_path.is_file() else None,
        staging_dir=staging,
        ship=True,
    )
    art_quality = _detect_art_quality(staging)
    schema_pass = promo_rep.status == "passed"
    witness: dict[str, Any] = {
        "program_id": "PLAN-TILE-FIX-AUTO-BUILD-001",
        "task_id": "TILE-FIX-10",
        "gate": "TILE-FIX-10-promotion",
        "green": schema_pass and art_quality == "keyframe_manual",
        "schema_validation_passed": schema_pass,
        "art_quality": art_quality,
        "updated": datetime.now(timezone.utc).isoformat(),
        "building_id": defn.building_id,
        "minimum_g4_cells": MINIMUM_G4_CELLS,
        "shell_production_ready": shell_ok,
        "shell_blockers": shell_blockers,
        "promotion_validation": promo_rep.to_dict(),
        "pipeline_summary": pipeline_summary or {},
        "meta_json": str(meta_path.relative_to(repo_root())).replace("\\", "/")
        if meta_path.is_file()
        else None,
        "_agent_meta": {"agent": "coder-mcp", "lane": "tile_compile_minimum"},
    }
    if art_quality != "keyframe_manual":
        witness["freeze_reason"] = (
            "TILE-FIX-010 schema pass only — headless tile_keyframe_bake is not ship art; "
            "export manual keyframe_render stills and add keyframe_manual.export marker"
        )
    out = art_pipeline_dir() / f"tile_fix_10_{defn.building_id}_live.json"
    out.write_text(json.dumps(witness, indent=2) + "\n", encoding="utf-8")
    witness["written"] = str(out)
    return witness


def _detect_art_quality(staging: Path) -> str:
    """Classify minimum G4 stills — manual keyframe_render vs headless procedural."""
    _HEADLESS_METHODS = frozenset(
        {
            "blender_keyframe_light_rig",
            "tile_compile_minimum_bake",
            "blender_orthographic_iso",
            "smoke_ortho_headless",
        }
    )

    def _marker_body(path: Path) -> dict[str, Any] | None:
        if not path.is_file():
            return None
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
            return data if isinstance(data, dict) else None
        except json.JSONDecodeError:
            return None

    marker = _marker_body(staging / "keyframe_manual.export")
    if marker is not None:
        method = str(marker.get("method") or "")
        if method in _HEADLESS_METHODS:
            return "rejected_headless_procedural"
        if str(marker.get("export_mode") or "") == "keyframe_manual" and method in (
            "",
            "keyframe_render_addon",
            "keyframe_render.py",
        ):
            return "keyframe_manual"

    batch_status = _marker_body(staging / "batch_status.json")
    if batch_status is not None:
        method = str(batch_status.get("method") or batch_status.get("render_method") or "")
        if method in _HEADLESS_METHODS:
            return "rejected_headless_procedural"
        if str(batch_status.get("export_mode") or "") == "keyframe_manual":
            if method and method not in ("keyframe_render_addon", "keyframe_render.py"):
                return "rejected_headless_procedural"
            return "keyframe_manual"

    return "rejected_headless_procedural"


def _default_visual_config_path(defn: BuildingDefinition) -> Path:
    if defn.building_id == "warehouse_industrial":
        return repo_root() / "assets/configs/buildings/visual_config_warehouse_industrial_west_v2.json"
    return repo_root() / f"assets/configs/buildings/visual_config_{defn.building_id}_v2.json"


def run_designer_warehouse_phase_c(
    defn_path: str | Path,
    *,
    require_manual_art: bool = True,
) -> dict[str, Any]:
    """TILE-FIX Phase C — same steps as CLI/MCP validate-report + promotion witness (no ad-hoc audit)."""
    from rust_engine_mcp.validators import run_validator
    from rust_engine_mcp.validators.tile_promotion import validate_tile_promotion

    root = repo_root()
    bdef = Path(defn_path)
    if not bdef.is_absolute():
        bdef = root / bdef
    defn = load_building_definition(bdef)
    staging = root / "assets/staging/tiles" / _staging_batch_id(defn)
    meta_path = staging / "atlas_meta.json"
    vc_path = _default_visual_config_path(defn)

    steps: list[dict[str, Any]] = []

    vc_rep = run_validator("visual_config", str(vc_path.relative_to(root)))
    steps.append(
        {
            "step": 1,
            "cli": f"validate-report visual_config {vc_path.relative_to(root).as_posix()}",
            "status": vc_rep.status,
        }
    )

    atlas_rep = run_validator("atlas_meta_v2", str(meta_path.relative_to(root)))
    steps.append(
        {
            "step": 2,
            "cli": f"validate-report atlas_meta_v2 {meta_path.relative_to(root).as_posix()}",
            "status": atlas_rep.status,
        }
    )

    tf10 = write_tile_fix_10_witness(bdef)
    steps.append(
        {
            "step": 3,
            "cli": "write-tile-fix-10-witness --building <building_definition>",
            "status": "passed" if tf10.get("green") else "failed",
            "written": tf10.get("written"),
        }
    )

    promo_rep = validate_tile_promotion(building_definition_path=bdef, ship=True)
    steps.append(
        {
            "step": 4,
            "cli": f"validate-report tile_promotion {bdef.relative_to(root).as_posix()}",
            "status": promo_rep.status,
        }
    )

    art_quality = str(tf10.get("art_quality") or _detect_art_quality(staging))
    minimum_g4_ship = False
    if meta_path.is_file():
        try:
            minimum_g4_ship = bool(json.loads(meta_path.read_text(encoding="utf-8")).get("minimum_g4_ship"))
        except json.JSONDecodeError:
            pass

    schema_green = all(
        s.get("status") == "passed" for s in steps if s.get("step") in (1, 2, 4)
    ) and bool(minimum_g4_ship)
    proceed_ship = schema_green and tf10.get("green") and (
        not require_manual_art or art_quality == "keyframe_manual"
    )

    witness: dict[str, Any] = {
        "program_id": "PLAN-TILE-FIX-AUTO-BUILD-001",
        "task_id": "TILE-FIX-09-PHASE-C",
        "gate": "G4-minimum-stills",
        "green": proceed_ship,
        "updated": datetime.now(timezone.utc).isoformat(),
        "matrix": {"states": 3, "facings": 8, "cells": MINIMUM_G4_CELLS},
        "cli_steps": steps,
        "tile_fix_10": tf10.get("written"),
        "promotion_validation": promo_rep.to_dict(),
        "minimum_g4_ship": minimum_g4_ship,
        "art_quality": art_quality,
        "proceed_ship": proceed_ship,
        "proceed_ship_basis": (
            "minimum_g4_ship + validate_tile_promotion + atlas_meta_v2 + visual_config"
            if proceed_ship
            else "blocked — schema may pass but art_quality != keyframe_manual (see slice v2)"
        ),
        "_agent_meta": {"agent": "designer-mcp", "lane": "cli_validate_report"},
    }
    out = art_pipeline_dir() / "tile_fix_09_phase_c_warehouse_g4_live.json"
    out.write_text(json.dumps(witness, indent=2) + "\n", encoding="utf-8")
    witness["written"] = str(out)
    return witness


def cell_to_dict(cell: BakeCell) -> dict[str, Any]:
    return asdict(cell)
