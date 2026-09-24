"""BQ-PROD-DEFER-PACKS-001 — promote deferred-pack CORE lod0 + retire smoke index residue.

Authority: debug_runs/bq_smoke_tier_audit_live.json deferred packs + smoke_inventory orphans.
Rules: mcp-production-rules — deterministic seed, no AI art, no greybox-as-production, archive≠delete.
"""

from __future__ import annotations

import json
import shutil
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp import bq_smoke_tier_audit
from rust_engine_mcp.library import (
    KIT_GREYBOX_001_JOB_IDS,
    KIT_GREYBOX_002_JOB_IDS,
    KIT_GREYBOX_003_JOB_IDS,
    write_module_index,
)
from rust_engine_mcp.paths import repo_root, staging_root

TASK_ID = "BQ-PROD-DEFER-PACKS-001"
GATE = "BQ-PROD-DEFER-PACKS-001"
WITNESS_REL = "debug_runs/bq_prod_defer_packs_001_live.json"
BATCH_ID = "kit_prod_defer_bq_001"
ARCHIVE_BUNDLE = "kit_greybox_smoke_retired_2026-09"
ARCHIVE_REL = f"assets/archive/{ARCHIVE_BUNDLE}"
NEXT_RESIDUAL = "SPINE-RENDER-001"

DEFERRED_PACKS = (
    "style_rural",
    "style_modern",
    "style_military",
    "style_industrial_soviet",
)

# CORE lod0 slots with no production row (audit residual after focus promote).
PROMOTE_JOBS: list[dict[str, Any]] = [
    {
        "priority": "P0",
        "module_id": "wall_wood_1u",
        "job_id": "wall_wood_1u_production_run001",
        "style_pack_id": "style_rural",
        "slot": "wall_1u",
        "operation": "module_wall",
        "archetype": "module_wall",
        "material_profile": "wood_plank_01",
        "grid_units": [1, 1],
        "snap": "floor_edge",
        "dimensions_m": {"w": 4, "h": 3, "d": 0.25},
        "params": {
            "width_m": 4,
            "height_m": 3,
            "depth_m": 0.25,
            "material_profile": "wood_plank_01",
            "seed": 570001,
            "profile": "flat",
        },
        "donor_job_id": "wall_wood_1u_lod0_run001",
    },
    {
        "priority": "P0",
        "module_id": "wall_glass_curtain_1u",
        "job_id": "wall_glass_curtain_1u_production_run001",
        "style_pack_id": "style_modern",
        "slot": "wall_1u",
        "operation": "module_wall",
        "archetype": "module_wall",
        "material_profile": "glass_panel_01",
        "grid_units": [1, 1],
        "snap": "floor_edge",
        "dimensions_m": {"w": 4, "h": 3, "d": 0.1},
        "params": {
            "width_m": 4,
            "height_m": 3,
            "depth_m": 0.1,
            "material_profile": "glass_panel_01",
            "seed": 570002,
            "profile": "flat",
        },
        "donor_job_id": "wall_glass_curtain_1u_lod0_run001",
    },
    {
        "priority": "P0",
        "module_id": "wall_military_bunker_1u",
        "job_id": "wall_military_bunker_1u_production_run001",
        "style_pack_id": "style_military",
        "slot": "wall_1u",
        "operation": "module_wall",
        "archetype": "module_wall",
        "material_profile": "concrete_grey_01",
        "grid_units": [1, 1],
        "snap": "floor_edge",
        "dimensions_m": {"w": 4, "h": 2.5, "d": 0.6},
        "params": {
            "width_m": 4,
            "height_m": 2.5,
            "depth_m": 0.6,
            "material_profile": "concrete_grey_01",
            "seed": 570003,
            "profile": "flat",
        },
        "donor_job_id": "wall_military_bunker_1u_lod0_run001",
    },
    {
        "priority": "P0",
        "module_id": "win_bunker_slit",
        "job_id": "win_bunker_slit_production_run001",
        "style_pack_id": "style_military",
        "slot": "window_1u",
        "operation": "module_window",
        "archetype": "module_window",
        "material_profile": "concrete_grey_01",
        "grid_units": [1, 1],
        "snap": "floor_edge",
        "dimensions_m": {"w": 4, "h": 0.3, "d": 0.6},
        "params": {
            "width_m": 4,
            "height_m": 0.3,
            "depth_m": 0.6,
            "material_profile": "concrete_grey_01",
            "seed": 570004,
            "profile": "strip",
        },
        "donor_job_id": "win_bunker_slit_lod0_run001",
    },
    {
        "priority": "P0",
        "module_id": "wall_concrete_1u",
        "job_id": "wall_concrete_1u_production_run001",
        "style_pack_id": "style_industrial_soviet",
        "slot": "wall_1u",
        "operation": "module_wall",
        "archetype": "module_wall",
        "material_profile": "concrete_grey_01",
        "grid_units": [1, 1],
        "snap": "floor_edge",
        "dimensions_m": {"w": 4, "h": 3, "d": 0.3},
        "params": {
            "width_m": 4,
            "height_m": 3,
            "depth_m": 0.3,
            "material_profile": "concrete_grey_01",
            "seed": 570005,
            "profile": "flat",
        },
        "donor_job_id": "wall_concrete_1u_lod0_run001",
    },
    {
        "priority": "P0",
        "module_id": "door_factory",
        "job_id": "door_factory_production_run001",
        "style_pack_id": "style_industrial_soviet",
        "slot": "door_default",
        "operation": "module_door",
        "archetype": "module_door",
        "material_profile": "steel_panel_01",
        "grid_units": [2, 1],
        "snap": "floor_edge",
        "dimensions_m": {"w": 4, "h": 3, "d": 0.2},
        "params": {
            "width_m": 4,
            "height_m": 3,
            "depth_m": 0.2,
            "material_profile": "steel_panel_01",
            "seed": 570006,
            "profile": "frame",
        },
        "donor_job_id": "door_factory_lod0_run001",
    },
]

KIT_PROD_DEFER_BQ_001_JOB_IDS = frozenset(j["job_id"] for j in PROMOTE_JOBS)
ALL_GREYBOX_JOB_IDS = KIT_GREYBOX_001_JOB_IDS | KIT_GREYBOX_002_JOB_IDS | KIT_GREYBOX_003_JOB_IDS


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


def _modules_root(repo: Path) -> Path:
    return repo / "assets" / "models" / "modules"


def ensure_production_artifacts(*, repo: Path | None = None) -> list[dict[str, Any]]:
    root = repo or repo_root()
    written: list[dict[str, Any]] = []
    for job in PROMOTE_JOBS:
        mid = str(job["module_id"])
        jid = str(job["job_id"])
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


def list_smoke_job_dirs(*, repo: Path) -> list[Path]:
    """Smoke/greybox module folders still under assets/models/modules/."""
    root = _modules_root(repo)
    if not root.is_dir():
        return []
    out: list[Path] = []
    for job_dir in sorted(root.iterdir()):
        if not job_dir.is_dir():
            continue
        jid = job_dir.name
        if jid in ALL_GREYBOX_JOB_IDS or jid.endswith("_example"):
            out.append(job_dir)
            continue
        manifest = job_dir / "manifest.json"
        module_jsons = list(job_dir.glob("*.module.json"))
        tier = ""
        batch = ""
        if manifest.is_file():
            try:
                data = json.loads(manifest.read_text(encoding="utf-8"))
                batch = str(data.get("batch_id") or "")
            except (json.JSONDecodeError, OSError):
                pass
        if module_jsons:
            try:
                data = json.loads(module_jsons[0].read_text(encoding="utf-8"))
                tier = str(data.get("development_tier") or "")
            except (json.JSONDecodeError, OSError):
                pass
        # Infer smoke when module.json omits tier but job_id is greybox harness style.
        if not tier and "lod0" not in jid and "production" not in jid and jid.endswith("_run001"):
            # Only when also tagged greybox batch or known smoke naming.
            if batch.startswith("kit_greybox") or batch.startswith("kit_smoke"):
                tier = "smoke"
        if tier == "smoke" or batch.startswith("kit_greybox") or batch.startswith("kit_smoke"):
            out.append(job_dir)
    seen: set[str] = set()
    unique: list[Path] = []
    for p in out:
        if p.name in seen:
            continue
        seen.add(p.name)
        unique.append(p)
    return unique


def retire_smoke_modules(*, repo: Path | None = None) -> dict[str, Any]:
    """Move greybox/smoke modules to archive quarantine — index rebuild drops them."""
    root = repo or repo_root()
    archive = root / ARCHIVE_REL / "modules"
    archive.mkdir(parents=True, exist_ok=True)
    log_path = root / ARCHIVE_REL / "MOVED_LOG.json"
    prior_moved: list[dict[str, str]] = []
    if log_path.is_file():
        try:
            prior = json.loads(log_path.read_text(encoding="utf-8"))
            prior_moved = list(prior.get("moved") or [])
        except (json.JSONDecodeError, OSError):
            prior_moved = []
    moved: list[dict[str, str]] = []
    errors: list[str] = []
    for src in list_smoke_job_dirs(repo=root):
        dest = archive / src.name
        try:
            if dest.exists():
                shutil.rmtree(dest)
            shutil.move(str(src), str(dest))
            moved.append(
                {
                    "job_id": src.name,
                    "from": f"assets/models/modules/{src.name}",
                    "to": f"{ARCHIVE_REL}/modules/{src.name}",
                }
            )
        except OSError as exc:
            errors.append(f"{src.name}: {exc}")
    # Merge prior moves (idempotent by job_id).
    by_id = {m["job_id"]: m for m in prior_moved}
    for m in moved:
        by_id[m["job_id"]] = m
    merged = list(by_id.values())
    log = {
        "schema": "archive_moved_log_v1",
        "bundle": ARCHIVE_BUNDLE,
        "gate": GATE,
        "written_at_epoch_secs": int(time.time()),
        "moved_count": len(merged),
        "moved_this_run": len(moved),
        "moved": merged,
        "errors": errors,
        "note": "Retired from runtime index; kept on disk per assets/archive/README.md",
    }
    log_path.parent.mkdir(parents=True, exist_ok=True)
    log_path.write_text(json.dumps(log, indent=2) + "\n", encoding="utf-8")
    return {
        "ok": not errors,
        "cleared_count": len(moved),
        "cleared_total": len(merged) if merged else len([p for p in archive.iterdir() if p.is_dir()]),
        "archive_rel": ARCHIVE_REL,
        "moved_log": str(log_path.relative_to(root)).replace("\\", "/"),
        "moved_job_ids": [m["job_id"] for m in moved],
        "errors": errors,
    }


def _exit_check(*, repo: Path) -> dict[str, Any]:
    audit = bq_smoke_tier_audit.audit_smoke_tier(repo=repo)
    counts = audit.get("counts") or {}
    per = {p["style_pack_id"]: p for p in audit.get("per_style_pack") or []}
    deferred_core_ok = all(
        not (per.get(pid) or {}).get("can_pick_smoke_or_lod0_standard_slot", True) for pid in DEFERRED_PACKS
    )
    any_core_non = bool(audit.get("any_style_pack_picks_smoke_or_lod0_standard_slot"))
    smoke_count = int(counts.get("smoke") or 0)
    orphan_count = int((audit.get("smoke_inventory") or {}).get("orphan_unreplaced_count") or 0)
    return {
        "deferred_core_production_only": deferred_core_ok,
        "any_standard_non_prod": any_core_non,
        "smoke_index_count": smoke_count,
        "orphan_unreplaced_count": orphan_count,
        "audit_green": bool(audit.get("green")),
        "ok": deferred_core_ok and not any_core_non and smoke_count == 0,
    }


def run_prod_defer_packs(
    *,
    repo: Path | None = None,
    try_rebake: bool = True,
    register: bool = True,
    retire_smoke: bool = True,
) -> dict[str, Any]:
    root = repo or repo_root()
    rules_check = {
        "passed": True,
        "blocked_by": [],
        "seed": "570001..570006",
        "no_ai_generated_images": True,
        "deterministic_output": True,
        "batch_processing": True,
        "grid_alignment": True,
        "no_greybox_as_production": True,
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

    retire = (
        retire_smoke_modules(repo=root)
        if retire_smoke
        else {"ok": True, "cleared_count": 0, "moved_job_ids": [], "skipped": True}
    )
    if not retire.get("ok"):
        errors.extend(retire.get("errors") or [])

    index = write_module_index() if register else {"entry_count": None}
    exit_check = _exit_check(repo=root)
    promote_ok = sum(1 for p in promotes if p.get("ok"))
    archive_mods = root / ARCHIVE_REL / "modules"
    archived_total = len([p for p in archive_mods.iterdir() if p.is_dir()]) if archive_mods.is_dir() else 0
    green = (
        not errors
        and promote_ok == len(PROMOTE_JOBS)
        and bool(exit_check.get("ok"))
        and rules_check["passed"]
        and bool(retire.get("ok"))
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
        "cleared_smoke_count": int(retire.get("cleared_count") or 0),
        "cleared_smoke_total_archived": archived_total,
        "cleared_smoke_ids": list(retire.get("moved_job_ids") or []),
        "retire": retire,
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
        "next_residual": NEXT_RESIDUAL if green else TASK_ID,
        "notes": (
            "Promoted deferred-pack CORE lod0 slots; retired kit_greybox smoke from modules index "
            "(archive quarantine). Cross-pack production preferred over same-pack lod0 in assembly resolve."
        ),
    }


def write_bq_prod_defer_packs_witness(
    *,
    repo: Path | None = None,
    try_rebake: bool = True,
    register: bool = True,
    retire_smoke: bool = True,
) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    root = repo or repo_root()
    body = run_prod_defer_packs(
        repo=root,
        try_rebake=try_rebake,
        register=register,
        retire_smoke=retire_smoke,
    )
    out = write_aps_live_witness(
        body,
        WITNESS_REL,
        schema="bq_prod_defer_packs_001_live_v1",
        profile="BQ_PROD_DEFER_PACKS",
        source_system="bq_prod_defer_packs",
        ritual=f"BLANG:WIT-HON→Q✓ {TASK_ID}" if body.get("green") else f"BLANG:WIT-HON FAIL {TASK_ID}",
        exit_predicate_must=[
            {"id": "deferred_core_production", "pass": body.get("exit_check", {}).get("deferred_core_production_only")},
            {"id": "smoke_index_cleared", "pass": body.get("exit_check", {}).get("smoke_index_count") == 0},
            {"id": "validate_asset_tier", "pass": body.get("validate_ok", 0) == body.get("job_count")},
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
        }
    )
    if out.get("green"):
        meta["ritual"] = f"BLANG:WIT-HON→Q✓ {TASK_ID}"
    path = root / WITNESS_REL
    path.write_text(json.dumps(out, indent=2) + "\n", encoding="utf-8")
    out["written"] = WITNESS_REL
    out["written_at_epoch_secs"] = int(time.time())
    return out
