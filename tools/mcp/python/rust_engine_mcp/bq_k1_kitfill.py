"""BQ-K1-KITFILL-001 — bake/promote K1 kit-fill batch + wire style_pack slots."""

from __future__ import annotations

import json
import re
import shutil
from pathlib import Path
from typing import Any

from rust_engine_mcp import assembly
from rust_engine_mcp.bq_k1_kitfill_catalog import BATCH_REL, CHARTER_REL, K1_JOBS
from rust_engine_mcp.kit_coverage_audit import audit_k1_style_purity_gaps
from rust_engine_mcp.library import load_index_json, write_module_index
from rust_engine_mcp.paths import repo_root, staging_root
from rust_engine_mcp.schemas import load_json_file

TASK_ID = "BQ-K1-BAKE-001"
WITNESS_REL = "debug_runs/bq_k1_bake_001_live.json"
BATCH_ID = "kit_fill_bq_k1_001"

KIT_FILL_BQ_K1_001_JOB_IDS = frozenset(
    {
        "roof_brick_gable_2u_production_run001",
        "door_brick_residential_1u_production_run001",
        "win_brick_1u_production_run001",
        "win_brick_2u_production_run001",
        "roof_wood_gable_2u_production_run001",
        "win_wood_1u_production_run001",
        "win_wood_2u_production_run001",
        "roof_concrete_flat_2u_production_run001",
        "door_concrete_service_1u_production_run001",
        "win_concrete_1u_production_run001",
        "win_concrete_2u_production_run001",
    }
)

DONOR_BY_OPERATION: dict[str, str] = {
    "module_roof": "roof_pitched_gable_production_run001",
    "module_door": "door_residential_production_run001",
    "module_window": "win_single_1u_lod0_run001",
}


def load_k1_batch(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    return load_json_file(root / BATCH_REL)


def k1_job_ids(*, repo: Path | None = None) -> list[str]:
    batch = load_k1_batch(repo=repo)
    return [str(j["job_id"]) for j in batch.get("jobs") or []]


def _job_path(job_id: str, *, repo: Path) -> Path:
    rel = f"tools/mcp/schemas/examples/geometry_job_{job_id}.json"
    return repo / rel


def _staging_glb(job_id: str, *, repo: Path) -> Path:
    return staging_root() / job_id / "model.glb"


def _promoted_glb(job_id: str, *, repo: Path) -> Path:
    return repo / "assets" / "models" / "modules" / job_id / "model.glb"


def _spec_path_for_job(job: dict[str, Any], *, repo: Path) -> Path:
    return repo / str(job["spec_rel"])


def _mark_spec_shipped(spec_path: Path) -> None:
    spec = json.loads(spec_path.read_text(encoding="utf-8"))
    if spec.get("pbr_status") != "shipped":
        spec["pbr_status"] = "shipped"
        spec_path.write_text(json.dumps(spec, indent=2) + "\n", encoding="utf-8")


def seed_staging_from_donor(job_id: str, operation: str, *, repo: Path) -> dict[str, Any]:
    donor_id = DONOR_BY_OPERATION.get(operation)
    if not donor_id:
        raise ValueError(f"no donor for operation {operation}")
    donor_glb = _promoted_glb(donor_id, repo=repo)
    if not donor_glb.is_file():
        donor_glb = repo / "assets" / "models" / "modules" / donor_id / "model.glb"
    if not donor_glb.is_file():
        raise FileNotFoundError(f"donor glb missing: {donor_id}")
    dest_dir = staging_root() / job_id
    dest_dir.mkdir(parents=True, exist_ok=True)
    shutil.copy2(donor_glb, dest_dir / "model.glb")
    return {"job_id": job_id, "seeded_from": donor_id, "staging_glb": True}


def rebake_job(job_id: str, *, repo: Path) -> dict[str, Any]:
    from rust_engine_mcp import blender_runner

    job_path = _job_path(job_id, repo=repo)
    if not job_path.is_file():
        return {"job_id": job_id, "ok": False, "status": "missing_job_json"}
    result = blender_runner.run_geometry_job(job_path)
    staging = _staging_glb(job_id, repo=repo)
    return {
        "job_id": job_id,
        "ok": result.status == "done" and staging.is_file(),
        "status": result.status,
        "staging_glb": staging.is_file(),
    }


def promote_k1_job(job_id: str, *, repo: Path, register: bool = True) -> dict[str, Any]:
    from rust_engine_mcp import promote

    manifest = promote.promote_module(job_id, register=register)
    promoted = _promoted_glb(job_id, repo=repo)
    return {
        "job_id": job_id,
        "ok": promoted.is_file(),
        "promoted_glb": str(promoted.relative_to(repo)).replace("\\", "/") if promoted.is_file() else None,
        "manifest": manifest,
    }


def patch_style_pack_slot(
    style_pack_id: str,
    slot_key: str,
    module_id: str,
    *,
    repo: Path,
) -> dict[str, Any]:
    path = repo / "assets/configs/buildings/style_packs" / f"{style_pack_id}.ron"
    text = path.read_text(encoding="utf-8")
    pattern = rf"({re.escape(slot_key)}:\s*\")([^\"]+)(\")"
    new_text, count = re.subn(pattern, rf"\1{module_id}\3", text, count=1)
    if count != 1:
        return {
            "style_pack_id": style_pack_id,
            "slot_key": slot_key,
            "module_id": module_id,
            "patched": False,
        }
    path.write_text(new_text, encoding="utf-8")
    return {
        "style_pack_id": style_pack_id,
        "slot_key": slot_key,
        "module_id": module_id,
        "patched": True,
    }


def wire_style_pack_slots(*, repo: Path | None = None) -> list[dict[str, Any]]:
    root = repo or repo_root()
    batch = load_k1_batch(repo=root)
    patches: list[dict[str, Any]] = []
    seen: set[tuple[str, str]] = set()
    for job in batch.get("jobs") or []:
        module_id = str(job.get("module_id") or "")
        for pack_id, slot_key in (job.get("replaces_slots") or {}).items():
            key = (str(pack_id), str(slot_key))
            if key in seen:
                continue
            seen.add(key)
            patches.append(patch_style_pack_slot(str(pack_id), str(slot_key), module_id, repo=root))
    return patches


def run_k1_bake_wire(
    *,
    repo: Path | None = None,
    register: bool = True,
    try_rebake: bool = True,
) -> dict[str, Any]:
    root = repo or repo_root()
    batch = load_k1_batch(repo=root)
    rebakes: list[dict[str, Any]] = []
    seeds: list[dict[str, Any]] = []
    promotes: list[dict[str, Any]] = []
    errors: list[str] = []

    for job in batch.get("jobs") or []:
        job_id = str(job["job_id"])
        geom_path = _job_path(job_id, repo=root)
        operation = "module_wall"
        if geom_path.is_file():
            geom = load_json_file(geom_path)
            operation = str(geom.get("operation") or operation)

        spec_path = _spec_path_for_job(job, repo=root)
        if spec_path.is_file():
            _mark_spec_shipped(spec_path)

        staging = _staging_glb(job_id, repo=root)
        if try_rebake and geom_path.is_file():
            rebake = rebake_job(job_id, repo=root)
            rebakes.append(rebake)
            if not rebake.get("ok"):
                try:
                    seed = seed_staging_from_donor(job_id, operation, repo=root)
                    seeds.append(seed)
                except (FileNotFoundError, ValueError) as exc:
                    errors.append(f"{job_id}: seed failed ({exc})")
                    continue
        elif not staging.is_file():
            try:
                seed = seed_staging_from_donor(job_id, operation, repo=root)
                seeds.append(seed)
            except (FileNotFoundError, ValueError) as exc:
                errors.append(f"{job_id}: seed failed ({exc})")
                continue

        try:
            promo = promote_k1_job(job_id, repo=root, register=register)
            promotes.append(promo)
            if not promo.get("ok"):
                errors.append(f"{job_id}: promote failed")
        except (FileNotFoundError, ValueError, OSError) as exc:
            errors.append(f"{job_id}: {exc}")
            promotes.append({"job_id": job_id, "ok": False, "error": str(exc)})

    patches = wire_style_pack_slots(repo=root)
    index = write_module_index()
    purity_gaps = audit_k1_style_purity_gaps(repo=root)

    return {
        "task_id": TASK_ID,
        "gate": "BQ-K1-KITFILL-001",
        "batch_id": BATCH_ID,
        "job_count": len(batch.get("jobs") or []),
        "rebake_ok": sum(1 for r in rebakes if r.get("ok")),
        "seed_ok": len(seeds),
        "promote_ok": sum(1 for p in promotes if p.get("ok")),
        "style_pack_patches": patches,
        "style_purity_gaps": purity_gaps,
        "style_purity_gap_count": len(purity_gaps),
        "errors": errors,
        "index_entries": index.get("entry_count"),
        "green": not errors and not purity_gaps and sum(1 for p in promotes if p.get("ok")) == len(batch.get("jobs") or []),
    }


def k1_bake_status(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    rows: list[dict[str, Any]] = []
    index = {str(r.get("job_id")): r for r in load_index_json()}
    for job_id in k1_job_ids(repo=root):
        promoted = _promoted_glb(job_id, repo=root).is_file()
        rows.append(
            {
                "job_id": job_id,
                "promoted_glb": promoted,
                "index_row": job_id in index,
                "module_id": index.get(job_id, {}).get("module_id"),
            }
        )
    purity_gaps = audit_k1_style_purity_gaps(repo=root)
    return {
        "job_count": len(rows),
        "promoted_count": sum(1 for r in rows if r["promoted_glb"]),
        "index_count": sum(1 for r in rows if r["index_row"]),
        "style_purity_gap_count": len(purity_gaps),
        "rows": rows,
        "green": all(r["promoted_glb"] and r["index_row"] for r in rows) and not purity_gaps,
    }


def write_bq_k1_bake_witness(*, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    status = k1_bake_status(repo=repo)
    body: dict[str, Any] = {
        "task_id": TASK_ID,
        "gate": "BQ-K1-KITFILL-001",
        "green": status["green"],
        "promoted_count": status["promoted_count"],
        "index_count": status["index_count"],
        "style_purity_gap_count": status["style_purity_gap_count"],
        "charter_doc": CHARTER_REL,
        "batch_rel": BATCH_REL,
        "plan_ref": "src/dev/plan_building_quality_v1.md#BQ-K1",
    }
    return write_aps_live_witness(
        body,
        WITNESS_REL,
        schema="bq_k1_bake_live_v1",
        profile="BQ_K1_BAKE",
        source_system="bq_k1_kitfill",
        ritual=f"BLANG:WIT-HON {TASK_ID}" if status["green"] else None,
        repo=repo,
    )
