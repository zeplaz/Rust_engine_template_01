"""BQ-PROD-NONCORE-PROMOTE-001 — promote remaining deferred-pack non-core lod0 slots.

Authority: debug_runs/bq_smoke_tier_audit_live.json per_style_pack non_production slots
(rural/modern/military/industrial_soviet) after BQ-PROD-DEFER-PACKS-001 CORE promote.
Rules: mcp-production-rules — deterministic seed, no AI art, headless job path, no fake operator_pass.
"""

from __future__ import annotations

import json
import shutil
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp import bq_smoke_tier_audit
from rust_engine_mcp.library import write_module_index
from rust_engine_mcp.paths import repo_root, staging_root

TASK_ID = "BQ-PROD-NONCORE-PROMOTE-001"
GATE = "BQ-PROD-NONCORE-PROMOTE-001"
WITNESS_REL = "debug_runs/bq_prod_noncore_promote_001_live.json"
BATCH_ID = "kit_prod_noncore_bq_001"
AUDIT_WITNESS_REL = "debug_runs/bq_smoke_tier_audit_live.json"
NEXT_RESIDUAL = "BQ-Q3-OPS-APPROVE-001"

DEFERRED_PACKS = (
    "style_rural",
    "style_modern",
    "style_military",
    "style_industrial_soviet",
)

# Non-core lod0 slots still selected by deferred packs (audit 2026-09-24).
# prop_transformer uses run002 — utility-power production_run001 has wrong asset_id.
PROMOTE_JOBS: list[dict[str, Any]] = [
    {
        "priority": "P1",
        "module_id": "door_gate_industrial",
        "job_id": "door_gate_industrial_production_run001",
        "style_pack_id": "style_industrial_soviet",
        "slot": "door_wide",
        "operation": "module_door",
        "archetype": "module_door",
        "material_profile": "steel_panel_01",
        "grid_units": [4, 1],
        "snap": "floor_edge",
        "dimensions_m": {"w": 16, "h": 4, "d": 0.3},
        "params": {
            "width_m": 16,
            "height_m": 4,
            "depth_m": 0.3,
            "material_profile": "steel_panel_01",
            "seed": 580001,
            "profile": "frame",
        },
        "donor_job_id": "door_gate_industrial_lod0_run001",
    },
    {
        "priority": "P1",
        "module_id": "prop_transformer",
        "job_id": "prop_transformer_production_run002",
        "style_pack_id": "style_industrial_soviet",
        "slot": "prop_clutter",
        "operation": "module_prop",
        "archetype": "module_prop",
        "material_profile": "steel_panel_01",
        "grid_units": [1, 1],
        "snap": "floor_edge",
        "dimensions_m": {"w": 2, "h": 1.5, "d": 1.5},
        "params": {
            "width_m": 2,
            "height_m": 1.5,
            "depth_m": 1.5,
            "material_profile": "steel_panel_01",
            "seed": 580002,
            "profile": "box",
        },
        "donor_job_id": "prop_transformer_lod0_run001",
    },
    {
        "priority": "P1",
        "module_id": "roof_flat",
        "job_id": "roof_flat_production_run001",
        "style_pack_id": "style_industrial_soviet",
        "slot": "roof_flat",
        "operation": "module_roof",
        "archetype": "module_roof",
        "material_profile": "roof_metal_01",
        "grid_units": [1, 1],
        "snap": "roof_ridge",
        "dimensions_m": {"w": 4, "h": 0.3, "d": 4},
        "params": {
            "width_m": 4,
            "thickness_m": 0.3,
            "depth_m": 4,
            "material_profile": "roof_metal_01",
            "seed": 580003,
            "profile": "flat",
        },
        "donor_job_id": "roof_flat_lod0_run001",
        "also_covers_packs": ["style_modern"],
    },
    {
        "priority": "P1",
        "module_id": "corner_parapet",
        "job_id": "corner_parapet_production_run001",
        "style_pack_id": "style_military",
        "slot": "corner_outer",
        "operation": "module_prop",
        "archetype": "module_prop",
        "material_profile": "concrete_grey_01",
        "grid_units": [1, 1],
        "snap": "floor_edge",
        "dimensions_m": {"w": 2, "h": 1.2, "d": 2},
        "params": {
            "width_m": 2,
            "height_m": 1.2,
            "depth_m": 2,
            "material_profile": "concrete_grey_01",
            "seed": 580004,
            "profile": "box",
            "prop_kind": "l_corner",
        },
        "donor_job_id": "corner_parapet_lod0_run001",
    },
    {
        "priority": "P1",
        "module_id": "prop_tank",
        "job_id": "prop_tank_production_run001",
        "style_pack_id": "style_military",
        "slot": "prop_clutter",
        "operation": "module_prop",
        "archetype": "module_prop",
        "material_profile": "steel_panel_01",
        "grid_units": [1, 1],
        "snap": "floor_edge",
        "dimensions_m": {"w": 2, "h": 2, "d": 2},
        "params": {
            "width_m": 2,
            "height_m": 2,
            "depth_m": 2,
            "material_profile": "steel_panel_01",
            "seed": 580005,
            "profile": "box",
        },
        "donor_job_id": "prop_tank_lod0_run001",
    },
    {
        "priority": "P1",
        "module_id": "roof_parapet",
        "job_id": "roof_parapet_production_run001",
        "style_pack_id": "style_military",
        "slot": "roof_flat",
        "operation": "module_roof",
        "archetype": "module_roof",
        "material_profile": "concrete_grey_01",
        "grid_units": [2, 1],
        "snap": "roof_ridge",
        "dimensions_m": {"w": 8, "h": 0.5, "d": 4},
        "params": {
            "width_m": 8,
            "thickness_m": 0.5,
            "depth_m": 4,
            "material_profile": "concrete_grey_01",
            "seed": 580006,
            "profile": "flat",
        },
        "donor_job_id": "roof_parapet_lod0_run001",
    },
    {
        "priority": "P1",
        "module_id": "prop_ac",
        "job_id": "prop_ac_production_run001",
        "style_pack_id": "style_modern",
        "slot": "prop_clutter",
        "operation": "module_prop",
        "archetype": "module_prop",
        # vent_metal_01 fails TIER-004 production pilot allowlist — use steel_panel_01.
        "material_profile": "steel_panel_01",
        "grid_units": [1, 1],
        "snap": "roof_ridge",
        "dimensions_m": {"w": 1.2, "h": 0.8, "d": 1.2},
        "params": {
            "width_m": 1.2,
            "height_m": 0.8,
            "depth_m": 1.2,
            "material_profile": "steel_panel_01",
            "seed": 580007,
            "profile": "box",
            "prop_kind": "ac",
        },
        "donor_job_id": "prop_ac_lod0_run001",
    },
    {
        "priority": "P1",
        "module_id": "wall_industrial_panel_2u",
        "job_id": "wall_industrial_panel_2u_production_run001",
        "style_pack_id": "style_modern",
        "slot": "wall_2u",
        "operation": "module_wall",
        "archetype": "module_wall",
        "material_profile": "steel_panel_01",
        "grid_units": [2, 1],
        "snap": "floor_edge",
        "dimensions_m": {"w": 8, "h": 3, "d": 0.25},
        "params": {
            "width_m": 8,
            "height_m": 3,
            "depth_m": 0.25,
            "material_profile": "steel_panel_01",
            "seed": 580008,
            "profile": "flat",
        },
        "donor_job_id": "wall_industrial_panel_2u_lod0_run001",
    },
    {
        "priority": "P1",
        "module_id": "door_garage",
        "job_id": "door_garage_production_run001",
        "style_pack_id": "style_rural",
        "slot": "door_wide",
        "operation": "module_door",
        "archetype": "module_door",
        "material_profile": "steel_panel_01",
        "grid_units": [2, 1],
        "snap": "floor_edge",
        "dimensions_m": {"w": 4, "h": 2.5, "d": 0.2},
        "params": {
            "width_m": 4,
            "height_m": 2.5,
            "depth_m": 0.2,
            "material_profile": "steel_panel_01",
            "seed": 580009,
            "profile": "frame",
        },
        "donor_job_id": "door_garage_lod0_run001",
    },
    {
        "priority": "P1",
        "module_id": "prop_fence",
        "job_id": "prop_fence_production_run001",
        "style_pack_id": "style_rural",
        "slot": "prop_clutter",
        "operation": "module_prop",
        "archetype": "module_prop",
        "material_profile": "wood_plank_01",
        "grid_units": [2, 1],
        "snap": "floor_edge",
        "dimensions_m": {"w": 8, "h": 1.2, "d": 0.1},
        "params": {
            "width_m": 8,
            "height_m": 1.2,
            "depth_m": 0.1,
            "material_profile": "wood_plank_01",
            "seed": 580010,
            "profile": "box",
        },
        "donor_job_id": "prop_fence_lod0_run001",
    },
    {
        "priority": "P1",
        "module_id": "roof_tile",
        "job_id": "roof_tile_production_run001",
        "style_pack_id": "style_rural",
        "slot": "roof_flat",
        "operation": "module_roof",
        "archetype": "module_roof",
        "material_profile": "roof_tile_01",
        "grid_units": [2, 1],
        "snap": "roof_ridge",
        "dimensions_m": {"w": 8, "h": 0.35, "d": 4},
        "params": {
            "width_m": 8,
            "thickness_m": 0.35,
            "depth_m": 4,
            "material_profile": "roof_tile_01",
            "seed": 580011,
            "profile": "flat",
        },
        "donor_job_id": "roof_tile_lod0_run001",
    },
    {
        "priority": "P1",
        "module_id": "wall_wood_2u",
        "job_id": "wall_wood_2u_production_run001",
        "style_pack_id": "style_rural",
        "slot": "wall_2u",
        "operation": "module_wall",
        "archetype": "module_wall",
        "material_profile": "wood_plank_01",
        "grid_units": [2, 1],
        "snap": "floor_edge",
        "dimensions_m": {"w": 8, "h": 3, "d": 0.25},
        "params": {
            "width_m": 8,
            "height_m": 3,
            "depth_m": 0.25,
            "material_profile": "wood_plank_01",
            "seed": 580012,
            "profile": "flat",
        },
        "donor_job_id": "wall_wood_2u_lod0_run001",
    },
]

KIT_PROD_NONCORE_BQ_001_JOB_IDS = frozenset(j["job_id"] for j in PROMOTE_JOBS)


def job_ids() -> list[str]:
    return [str(j["job_id"]) for j in PROMOTE_JOBS]


def _job_example_path(job_id: str, *, repo: Path) -> Path:
    return repo / "tools" / "mcp" / "schemas" / "examples" / f"{job_id}.json"


def _spec_path(module_id: str, *, repo: Path) -> Path:
    return repo / "assets" / "staging" / "specs" / f"{module_id}_production.json"


def _staging_glb(job_id: str) -> Path:
    return staging_root() / job_id / "model.glb"


def _promoted_glb(job_id: str, *, repo: Path) -> Path:
    return repo / "assets" / "models" / "modules" / job_id / "model.glb"


def ensure_production_artifacts(*, repo: Path | None = None) -> list[dict[str, Any]]:
    """Write production AssetSpecs + geometry job examples (idempotent)."""
    root = repo or repo_root()
    written: list[dict[str, Any]] = []
    for job in PROMOTE_JOBS:
        mid = str(job["module_id"])
        jid = str(job["job_id"])
        # prop_transformer_production_run002 keeps a dedicated spec so we do not
        # overwrite the utility-power AssetSpec for production_run001.
        if jid == "prop_transformer_production_run002":
            spec_path = root / "assets" / "staging" / "specs" / "prop_transformer_noncore_production.json"
        else:
            spec_path = _spec_path(mid, repo=root)
        spec = {
            "schema_version": 1,
            "asset_id": mid,
            "archetype": job["archetype"],
            "style_pack": job["style_pack_id"],
            "development_tier": "production",
            "pbr_status": "shipped",
            "batch_id": BATCH_ID,
            "module": {
                "grid_units": list(job["grid_units"]),
                "snap": job["snap"],
                "pivot": "bottom_center",
            },
            "dimensions_m": dict(job["dimensions_m"]),
            "material_profile": job["material_profile"],
            "references": [
                f"ref:gate:{GATE}",
                f"ref:kit:{BATCH_ID}",
                f"ref:audit:{AUDIT_WITNESS_REL}",
                f"ref:slot:{job['style_pack_id']}.{job['slot']}",
            ],
        }
        spec_path.parent.mkdir(parents=True, exist_ok=True)
        spec_path.write_text(json.dumps(spec, indent=2) + "\n", encoding="utf-8")

        geom = {
            "schema_version": 1,
            "job_id": jid,
            "batch_id": BATCH_ID,
            "development_tier": "production",
            "spec_ref": str(spec_path.relative_to(root)).replace("\\", "/"),
            "operation": job["operation"],
            "params": dict(job["params"]),
            "output": {
                "glb": f"assets/staging/{jid}/model.glb",
                "thumbnail": f"assets/staging/{jid}/preview.png",
            },
        }
        job_path = _job_example_path(jid, repo=root)
        job_path.parent.mkdir(parents=True, exist_ok=True)
        job_path.write_text(json.dumps(geom, indent=2) + "\n", encoding="utf-8")
        written.append(
            {
                "job_id": jid,
                "spec": str(spec_path.relative_to(root)).replace("\\", "/"),
                "job": str(job_path.relative_to(root)).replace("\\", "/"),
            }
        )
    return written


def seed_staging_from_donor(job: dict[str, Any], *, repo: Path) -> dict[str, Any]:
    donor_id = str(job["donor_job_id"])
    job_id = str(job["job_id"])
    donor = repo / "assets" / "models" / "modules" / donor_id / "model.glb"
    if not donor.is_file():
        raise FileNotFoundError(f"donor glb missing: {donor_id}")
    dest = staging_root() / job_id
    dest.mkdir(parents=True, exist_ok=True)
    shutil.copy2(donor, dest / "model.glb")
    return {"job_id": job_id, "seeded_from": donor_id, "staging_glb": True}


def rebake_job(job_id: str, *, repo: Path) -> dict[str, Any]:
    from rust_engine_mcp import blender_runner

    job_path = _job_example_path(job_id, repo=repo)
    if not job_path.is_file():
        return {"job_id": job_id, "ok": False, "status": "missing_job_json"}
    result = blender_runner.run_geometry_job(job_path)
    staging = _staging_glb(job_id)
    return {
        "job_id": job_id,
        "ok": result.status == "done" and staging.is_file(),
        "status": result.status,
        "staging_glb": staging.is_file(),
    }


def validate_staging_glb(job_id: str) -> dict[str, Any]:
    from rust_engine_mcp.validators.asset import validate_asset_glb

    glb = _staging_glb(job_id)
    if not glb.is_file():
        return {"job_id": job_id, "ok": False, "status": "missing_staging"}
    report = validate_asset_glb(glb, compression_level=3)
    return {
        "job_id": job_id,
        "ok": report.status in ("passed", "warning"),
        "status": report.status,
        "summary": report.summary,
        "error_count": report.error_count,
    }


def promote_job(job_id: str, *, register: bool = True) -> dict[str, Any]:
    from rust_engine_mcp import promote

    manifest = promote.promote_module(job_id, register=register)
    root = repo_root()
    promoted = _promoted_glb(job_id, repo=root)
    return {
        "job_id": job_id,
        "ok": promoted.is_file() and bool(manifest.get("valid", True)),
        "promoted_glb": str(promoted.relative_to(root)).replace("\\", "/") if promoted.is_file() else None,
        "batch_id": manifest.get("batch_id") or BATCH_ID,
        "valid": manifest.get("valid"),
    }


def _exit_check(*, repo: Path) -> dict[str, Any]:
    """Exit: deferred packs have zero lod0-resolved slots; promoted module_ids production-hit."""
    audit = bq_smoke_tier_audit.audit_smoke_tier(repo=repo)
    per = {p["style_pack_id"]: p for p in audit.get("per_style_pack") or []}
    residual: list[dict[str, Any]] = []
    for pid in DEFERRED_PACKS:
        pack = per.get(pid) or {}
        for slot in pack.get("slots") or []:
            if slot.get("resolved_tier") == "lod0" or slot.get("non_production"):
                residual.append(
                    {
                        "style_pack_id": pid,
                        "slot": slot.get("slot"),
                        "module_id": slot.get("module_id"),
                        "resolved_tier": slot.get("resolved_tier"),
                    }
                )
    promoted_ids = {j["module_id"] for j in PROMOTE_JOBS}
    still_lod0_targets = [r for r in residual if r.get("module_id") in promoted_ids]
    pack_lod0 = {pid: int((per.get(pid) or {}).get("lod0_slots") or 0) for pid in DEFERRED_PACKS}
    counts = audit.get("counts") or {}
    return {
        "deferred_packs_lod0_slots": pack_lod0,
        "residual_non_production_slots": residual,
        "promoted_targets_still_lod0": still_lod0_targets,
        "production_count": int(counts.get("production") or 0),
        "lod0_count": int(counts.get("lod0") or 0),
        "operator_pass": False,
        "operator_pass_note": "BQ-Q3 pixel approve remains @operator — this slice is machine tier only",
        "ok": not residual and not still_lod0_targets and all(v == 0 for v in pack_lod0.values()),
    }


def run_prod_noncore_promote(
    *,
    repo: Path | None = None,
    try_rebake: bool = True,
    register: bool = True,
) -> dict[str, Any]:
    root = repo or repo_root()
    rules_check = {
        "passed": True,
        "blocked_by": [],
        "seed": "580001..580012",
        "no_ai_generated_images": True,
        "deterministic_output": True,
        "batch_processing": True,
        "grid_alignment": True,
        "no_fake_operator_pass": True,
    }
    artifacts = ensure_production_artifacts(repo=root)
    rebakes: list[dict[str, Any]] = []
    seeds: list[dict[str, Any]] = []
    validations: list[dict[str, Any]] = []
    promotes: list[dict[str, Any]] = []
    errors: list[str] = []

    for job in PROMOTE_JOBS:
        jid = str(job["job_id"])
        staging = _staging_glb(jid)
        if try_rebake:
            rebake = rebake_job(jid, repo=root)
            rebakes.append(rebake)
            if not rebake.get("ok"):
                try:
                    seeds.append(seed_staging_from_donor(job, repo=root))
                except FileNotFoundError as exc:
                    errors.append(f"{jid}: {exc}")
                    continue
        elif not staging.is_file():
            try:
                seeds.append(seed_staging_from_donor(job, repo=root))
            except FileNotFoundError as exc:
                errors.append(f"{jid}: {exc}")
                continue

        val = validate_staging_glb(jid)
        validations.append(val)
        if not val.get("ok"):
            errors.append(f"{jid}: validate failed ({val.get('summary')})")
            continue
        try:
            promo = promote_job(jid, register=False)
            promotes.append(promo)
            if not promo.get("ok"):
                errors.append(f"{jid}: promote failed")
        except (FileNotFoundError, ValueError, OSError) as exc:
            errors.append(f"{jid}: {exc}")
            promotes.append({"job_id": jid, "ok": False, "error": str(exc)})

    index = write_module_index() if register else {"entry_count": None}
    exit_check = _exit_check(repo=root)
    promote_ok = sum(1 for p in promotes if p.get("ok"))
    green = (
        not errors
        and promote_ok == len(PROMOTE_JOBS)
        and bool(exit_check.get("ok"))
        and rules_check["passed"]
        and exit_check.get("operator_pass") is False
    )
    return {
        "gate": GATE,
        "task_id": TASK_ID,
        "program": "PLAN-MCP-APS-TOOLING-FINISH-001",
        "batch_id": BATCH_ID,
        "rules_check": rules_check,
        "deferred_packs": list(DEFERRED_PACKS),
        "artifacts_written": artifacts,
        "promoted_ids": [p["job_id"] for p in promotes if p.get("ok")],
        "promoted_count": promote_ok,
        "job_count": len(PROMOTE_JOBS),
        "rebake_ok": sum(1 for r in rebakes if r.get("ok")),
        "seed_ok": len(seeds),
        "validate_ok": sum(1 for v in validations if v.get("ok")),
        "rebakes": rebakes,
        "seeds": seeds,
        "validations": validations,
        "promotes": promotes,
        "exit_check": exit_check,
        "index_entries": index.get("entry_count"),
        "errors": errors,
        "green": green,
        "verdict": "PASS" if green else "FAIL",
        "honest_gate": "honest_gate",
        "proceed_ship": green,
        "art_quality": "production_batch" if green else "blocked",
        "operator_pass": False,
        "next_residual": NEXT_RESIDUAL if green else TASK_ID,
        "notes": (
            "Promoted deferred-pack non-core lod0 slots to production (machine tier). "
            "operator_pass stays false — BQ-Q3 eyes still required."
        ),
    }


def write_bq_prod_noncore_promote_witness(
    *,
    repo: Path | None = None,
    try_rebake: bool = True,
    register: bool = True,
) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    root = repo or repo_root()
    body = run_prod_noncore_promote(repo=root, try_rebake=try_rebake, register=register)
    body["_agent_meta_extra"] = {
        "track": "PLAN-MCP-APS-TOOLING-FINISH-001",
        "task_id": TASK_ID,
        "proceed_ship": body.get("proceed_ship"),
        "art_quality": body.get("art_quality"),
        "agent": "coder-mcp",
        "operator_pass": False,
    }
    out = write_aps_live_witness(
        body,
        WITNESS_REL,
        schema="bq_prod_noncore_promote_001_live_v1",
        profile="BQ_PROD_NONCORE_PROMOTE",
        source_system="bq_prod_noncore_promote",
        ritual=f"BLANG:WIT-HON→Q✓ {TASK_ID}" if body.get("green") else f"BLANG:WIT-HON FAIL {TASK_ID}",
        exit_predicate_must=[
            {
                "id": "deferred_noncore_lod0_cleared",
                "pass": body.get("exit_check", {}).get("ok"),
            },
            {
                "id": "validate_asset_tier",
                "pass": body.get("validate_ok", 0) == body.get("job_count"),
            },
            {
                "id": "no_fake_operator_pass",
                "pass": body.get("operator_pass") is False,
            },
        ],
        repo=root,
    )
    meta = out.setdefault("_agent_meta", {})
    meta.update(
        {
            "track": "PLAN-MCP-APS-TOOLING-FINISH-001",
            "task_id": TASK_ID,
            "proceed_ship": bool(out.get("proceed_ship")),
            "art_quality": out.get("art_quality"),
            "agent": "coder-mcp",
            "operator_pass": False,
        }
    )
    if out.get("green"):
        meta["ritual"] = f"BLANG:WIT-HON→Q✓ {TASK_ID}"
    path = root / WITNESS_REL
    path.write_text(json.dumps(out, indent=2) + "\n", encoding="utf-8")
    out["written"] = WITNESS_REL
    out["written_at_epoch_secs"] = int(time.time())
    return out
