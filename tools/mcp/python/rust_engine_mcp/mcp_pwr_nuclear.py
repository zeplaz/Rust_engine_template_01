"""MCP-PWR-NUCLEAR-BATCH-001 — nuclear PWR kit manifest, bpy batch, promote."""

from __future__ import annotations

import json
import shutil
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from rust_engine_mcp.mcp_pwr_utility import ensure_utility_manifest_assets
from rust_engine_mcp.paths import repo_root

MANIFEST_REL = "tools/mcp/schemas/examples/batch_kit_nuclear_pwr_production_001.manifest.json"
BATCH_ID = "kit_nuclear_pwr_production_001"

MANIFEST_WITNESS_REL = "debug_runs/art_pipeline/mcp_pwr_nuclear_manifest_live.json"
BATCH_WITNESS_REL = "debug_runs/art_pipeline/mcp_pwr_nuclear_batch_live.json"
PROMOTE_WITNESS_REL = "debug_runs/art_pipeline/mcp_pwr_nuclear_promote_live.json"

KIT_SPEC_REL = "assets/staging/specs/kit_nuclear_pwr_production_001.json"
KIT_JOB_ID = "kit_nuclear_pwr_production_run001"
KIT_ASSET_ID = "kit_nuclear_pwr_production_001"
CATALOG_ID = "nuclear_plant_pwr"
KIT_MESHES_REL = "assets/staging/meshes/kit_nuclear_pwr_production_001/model.glb"

FENCE_JOB_ID = "fence_chainlink_1u_production_run001"
FENCE_SPEC_REL = "assets/staging/specs/fence_chainlink_1u_production_run001.json"

COMPRESSION = 3


@dataclass(frozen=True)
class ModuleStub:
    module_id: str
    job_id: str
    spec_rel: str
    grid: tuple[int, int]
    dims_m: tuple[float, float, float]
    material_profile: str
    prop_kind: str
    snap: str = "floor_edge"


NUCLEAR_CONSTITUENT_STUBS: tuple[ModuleStub, ...] = (
    ModuleStub(
        "containment_dome_pwr",
        "containment_dome_pwr_production_run001",
        "assets/staging/specs/containment_dome_pwr_production_run001.json",
        (3, 3),
        (6.0, 6.0, 6.0),
        "concrete_grey_01",
        "containment_dome_pwr",
    ),
    ModuleStub(
        "turbine_hall_1u",
        "turbine_hall_1u_production_run001",
        "assets/staging/specs/turbine_hall_1u_production_run001.json",
        (2, 1),
        (4.0, 3.0, 2.0),
        "concrete_grey_01",
        "turbine_hall_1u",
    ),
    ModuleStub(
        "cooling_tower_1u",
        "cooling_tower_1u_production_run001",
        "assets/staging/specs/cooling_tower_1u_production_run001.json",
        (1, 1),
        (2.0, 5.0, 2.0),
        "concrete_grey_01",
        "cooling_tower_1u",
    ),
    ModuleStub(
        "diesel_gen_pad_2x2",
        "diesel_gen_pad_2x2_production_run001",
        "assets/staging/specs/diesel_gen_pad_2x2_production_run001.json",
        (2, 2),
        (4.0, 2.0, 4.0),
        "gravel_yard_01",
        "diesel_gen_pad_2x2",
    ),
    ModuleStub(
        "switchyard_edge_1u",
        "switchyard_edge_1u_production_run001",
        "assets/staging/specs/switchyard_edge_1u_production_run001.json",
        (1, 1),
        (2.0, 2.5, 2.0),
        "galvanized_steel_01",
        "switchyard_edge_1u",
    ),
    ModuleStub(
        "warning_sign_nuclear_1u",
        "warning_sign_nuclear_1u_production_run001",
        "assets/staging/specs/warning_sign_nuclear_1u_production_run001.json",
        (1, 1),
        (0.5, 1.5, 0.08),
        "warning_paint_yellow_01",
        "warning_sign_nuclear_1u",
        snap="floor_center",
    ),
)

KIT_NUCLEAR_PWR_PRODUCTION_001_JOB_IDS: frozenset[str] = frozenset(
    {stub.job_id for stub in NUCLEAR_CONSTITUENT_STUBS}
    | {KIT_JOB_ID, FENCE_JOB_ID}
)


def _root(repo: Path | None = None) -> Path:
    return repo or repo_root()


def _job_path(job_id: str, *, repo: Path | None = None) -> Path:
    return _root(repo) / "tools/mcp/schemas/examples" / f"{job_id}.json"


def load_manifest(*, repo: Path | None = None) -> dict[str, Any]:
    path = _root(repo) / MANIFEST_REL
    return json.loads(path.read_text(encoding="utf-8"))


def _spec_body(stub: ModuleStub) -> dict[str, Any]:
    w, h, d = stub.dims_m
    return {
        "schema_version": 1,
        "asset_id": stub.module_id,
        "archetype": "module_prop",
        "style_pack": "style_industrial_west",
        "development_tier": "production",
        "pbr_status": "pending",
        "batch_id": BATCH_ID,
        "module": {
            "grid_units": [stub.grid[0], stub.grid[1]],
            "snap": stub.snap,
            "pivot": "bottom_center",
        },
        "dimensions_m": {"w": w, "h": h, "d": d},
        "material_profile": stub.material_profile,
        "feeds": KIT_ASSET_ID,
        "references": [
            "ref:design:DES-ART-NUCLEAR-PLANT-001",
            "ref:dmcp:DMCP-SPEC-NUCLEAR-PWR-001",
            f"ref:kit:{KIT_ASSET_ID}",
        ],
    }


def _job_body(stub: ModuleStub) -> dict[str, Any]:
    w, h, d = stub.dims_m
    return {
        "schema_version": 1,
        "job_id": stub.job_id,
        "batch_id": BATCH_ID,
        "development_tier": "production",
        "spec_ref": stub.spec_rel,
        "operation": "module_prop",
        "params": {
            "width_m": w,
            "height_m": h,
            "depth_m": d,
            "material_profile": stub.material_profile,
            "seed": 44001,
            "profile": "box",
            "prop_kind": stub.prop_kind,
        },
        "output": {
            "glb": f"assets/staging/{stub.job_id}/model.glb",
            "thumbnail": f"assets/staging/{stub.job_id}/preview.png",
        },
    }


def _kit_job_body() -> dict[str, Any]:
    return {
        "schema_version": 1,
        "job_id": KIT_JOB_ID,
        "batch_id": BATCH_ID,
        "development_tier": "production",
        "spec_ref": KIT_SPEC_REL,
        "operation": "module_prop",
        "params": {
            "width_m": 12.0,
            "height_m": 9.0,
            "depth_m": 12.0,
            "material_profile": "concrete_grey_01",
            "seed": 44010,
            "profile": "box",
            "prop_kind": "nuclear_yard_kit",
            "name": KIT_ASSET_ID,
        },
        "output": {
            "glb": f"assets/staging/{KIT_JOB_ID}/model.glb",
            "thumbnail": f"assets/staging/{KIT_JOB_ID}/preview.png",
        },
    }


def ensure_nuclear_manifest_assets(*, repo: Path | None = None, force: bool = False) -> dict[str, Any]:
    root = _root(repo)
    ensure_utility_manifest_assets(repo=root)
    written_specs: list[str] = []
    written_jobs: list[str] = []

    for stub in NUCLEAR_CONSTITUENT_STUBS:
        spec_path = root / stub.spec_rel
        if force or not spec_path.is_file():
            spec_path.parent.mkdir(parents=True, exist_ok=True)
            spec_path.write_text(json.dumps(_spec_body(stub), indent=2) + "\n", encoding="utf-8")
            written_specs.append(stub.spec_rel)
        job_path = _job_path(stub.job_id, repo=root)
        if force or not job_path.is_file():
            job_path.write_text(json.dumps(_job_body(stub), indent=2) + "\n", encoding="utf-8")
            written_jobs.append(stub.job_id)

    kit_job = _job_path(KIT_JOB_ID, repo=root)
    if force or not kit_job.is_file():
        kit_job.write_text(json.dumps(_kit_job_body(), indent=2) + "\n", encoding="utf-8")
        written_jobs.append(KIT_JOB_ID)

    return {
        "written_specs": written_specs,
        "written_jobs": written_jobs,
        "constituent_count": len(NUCLEAR_CONSTITUENT_STUBS),
        "fence_reused": FENCE_JOB_ID,
    }


def _validate_spec(rel: str, *, repo: Path | None = None) -> bool:
    from rust_engine_mcp.validators.mcp_schema import validate_mcp_spec

    path = _root(repo) / rel
    if not path.is_file():
        return False
    report = validate_mcp_spec(path, compression_level=COMPRESSION)
    return report.status in ("passed", "warning")


def audit_nuclear_manifest(*, repo: Path | None = None) -> dict[str, Any]:
    root = _root(repo)
    ensure = ensure_nuclear_manifest_assets(repo=root)
    manifest = load_manifest(repo=root)
    modules = list(manifest.get("modules") or [])
    kit_valid = _validate_spec(KIT_SPEC_REL, repo=root)
    missing_specs = [
        stub.spec_rel for stub in NUCLEAR_CONSTITUENT_STUBS if not (root / stub.spec_rel).is_file()
    ]
    missing_jobs = [
        stub.job_id for stub in NUCLEAR_CONSTITUENT_STUBS if not _job_path(stub.job_id, repo=root).is_file()
    ]
    if not _job_path(KIT_JOB_ID, repo=root).is_file():
        missing_jobs.append(KIT_JOB_ID)
    if not (root / FENCE_SPEC_REL).is_file():
        missing_specs.append(FENCE_SPEC_REL)
    if not _job_path(FENCE_JOB_ID, repo=root).is_file():
        missing_jobs.append(FENCE_JOB_ID)

    green = (
        len(modules) >= 8
        and kit_valid
        and not missing_specs
        and not missing_jobs
    )
    return {
        "gate": "MCP-PWR-NUCLEAR-MANIFEST-001",
        "manifest_modules": len(modules),
        "kit_spec_valid": kit_valid,
        "missing_specs": missing_specs,
        "missing_jobs": missing_jobs,
        "ensure": ensure,
        "manifest": MANIFEST_REL,
        "green": green,
    }


def refresh_nuclear_manifest_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = _root(repo)
    body = audit_nuclear_manifest(repo=root)
    body["_agent_meta"] = {
        "schema": "mcp_pwr_nuclear_manifest_live_v1",
        "written_at_epoch_secs": int(time.time()),
        "profile": "MCP_PWR_NUCLEAR_MANIFEST",
        "source_system": "mcp_pwr_nuclear",
        "relative_path": MANIFEST_WITNESS_REL,
    }
    out = root / MANIFEST_WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = MANIFEST_WITNESS_REL
    return body


def _run_job(job_id: str, *, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp import blender_runner

    job_path = _job_path(job_id, repo=_root(repo))
    result = blender_runner.run_geometry_job(job_path)
    glb = _root(repo) / "assets/staging" / job_id / "model.glb"
    return {
        "job_id": job_id,
        "status": result.status,
        "staging_glb": glb.is_file(),
        "error": result.error,
        "ok": result.status == "done" and glb.is_file(),
    }


def run_nuclear_batch(*, repo: Path | None = None) -> dict[str, Any]:
    root = _root(repo)
    ensure_nuclear_manifest_assets(repo=root)
    constituent_results = [_run_job(stub.job_id, repo=root) for stub in NUCLEAR_CONSTITUENT_STUBS]
    fence_result = _run_job(FENCE_JOB_ID, repo=root)
    kit_result = _run_job(KIT_JOB_ID, repo=root)
    kit_staging = root / "assets/staging" / KIT_JOB_ID / "model.glb"
    mesh_out = root / KIT_MESHES_REL
    mesh_out.parent.mkdir(parents=True, exist_ok=True)
    if kit_staging.is_file():
        shutil.copy2(kit_staging, mesh_out)
    baked = sum(1 for r in constituent_results if r.get("ok"))
    fence_ok = bool(fence_result.get("ok"))
    return {
        "constituent_results": constituent_results,
        "fence_result": fence_result,
        "kit_result": kit_result,
        "constituent_modules_baked": baked,
        "fence_baked": fence_ok,
        "kit_staging_glb": kit_staging.is_file(),
        "meshes_glb": mesh_out.is_file(),
        "ok": baked >= 5 and fence_ok and kit_result.get("ok") and mesh_out.is_file(),
    }


def _prepare_spec_for_promote(spec_path: Path) -> None:
    from rust_engine_mcp.material_textures import PILOT_PROFILES, generate_profile, write_registry

    spec = json.loads(spec_path.read_text(encoding="utf-8"))
    mat = str(spec.get("material_profile") or "").strip()
    if mat in PILOT_PROFILES:
        generate_profile(PILOT_PROFILES[mat])
    write_registry()
    spec["pbr_status"] = "shipped"
    spec.pop("spec_only", None)
    spec.pop("bpy_blocked", None)
    spec_path.write_text(json.dumps(spec, indent=2) + "\n", encoding="utf-8")


def _ensure_catalog(*, repo: Path) -> None:
    catalog_path = repo / "assets/configs/buildings" / f"{CATALOG_ID}.json"
    if catalog_path.is_file():
        return
    body = {
        "asset_name": "Nuclear PWR plant",
        "asset_type": "Building",
        "segment": "Utilities",
        "description": "Grid-scale PWR generation site — containment dome + aux yard",
        "utility_role": "nuclear",
        "power_tier": "grid",
        "is_building": True,
        "is_power": True,
        "building_size_x": 6,
        "building_size_y": 6,
        "building_height": 4,
        "construction_cost": 48000,
        "power_consumption": 12,
        "power_generation": 1100,
        "plant_definition_id": "pwr_4loop_1100mw_v1",
        "version": "1.0.0",
    }
    catalog_path.write_text(json.dumps(body, indent=4) + "\n", encoding="utf-8")


def _update_catalog_module_id(module_id: str, glb_rel: str, *, repo: Path) -> None:
    catalog_path = repo / "assets/configs/buildings" / f"{CATALOG_ID}.json"
    _ensure_catalog(repo=repo)
    body = json.loads(catalog_path.read_text(encoding="utf-8"))
    body["procedural_module_id"] = module_id
    body["model_glb"] = glb_rel
    catalog_path.write_text(json.dumps(body, indent=4) + "\n", encoding="utf-8")


def _sync_manifest_status(job_id: str, *, repo: Path) -> None:
    manifest_path = repo / MANIFEST_REL
    body = json.loads(manifest_path.read_text(encoding="utf-8"))
    for row in body.get("modules") or []:
        if str(row.get("job_id") or "") == job_id:
            row["status"] = "promoted"
    manifest_path.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")


def promote_nuclear(*, register: bool = True, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp import promote
    from rust_engine_mcp.library import write_module_index

    root = _root(repo)
    spec_path = root / KIT_SPEC_REL
    _prepare_spec_for_promote(spec_path)
    manifest = promote.promote_module(KIT_JOB_ID, register=register)
    index = write_module_index()
    glb_rel = f"assets/models/modules/{KIT_JOB_ID}/model.glb"
    _update_catalog_module_id(KIT_ASSET_ID, glb_rel, repo=root)
    _sync_manifest_status(KIT_JOB_ID, repo=root)
    return {"promoted": manifest, "index_entries": index.get("entry_count"), "glb_rel": glb_rel}


def _asset_glb_status(rel: str, *, repo: Path, staging: bool = False) -> str:
    path = repo / rel
    if not path.is_file():
        return "missing"
    if staging:
        from rust_engine_mcp.validate_glb import validate_glb

        raw = validate_glb(path)
        if not raw.valid or not (raw.vertex_count or 0):
            return "failed"
        return "passed"
    from rust_engine_mcp.validators.asset import validate_asset_glb

    report = validate_asset_glb(path, compression_level=COMPRESSION)
    return report.status


def refresh_nuclear_batch_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = _root(repo)
    mesh_glb = root / KIT_MESHES_REL
    baked = sum(
        1
        for stub in NUCLEAR_CONSTITUENT_STUBS
        if (root / "assets/staging" / stub.job_id / "model.glb").is_file()
    )
    fence_baked = (root / "assets/staging" / FENCE_JOB_ID / "model.glb").is_file()
    validate_status = (
        _asset_glb_status(KIT_MESHES_REL, repo=root, staging=True) if mesh_glb.is_file() else "missing"
    )
    spec = json.loads((root / KIT_SPEC_REL).read_text(encoding="utf-8"))
    grid_units = (spec.get("module") or {}).get("grid_units")
    green = (
        mesh_glb.is_file()
        and validate_status in ("passed", "warning")
        and baked >= 5
        and fence_baked
        and grid_units == [6, 6]
    )
    body: dict[str, Any] = {
        "gate": "MCP-PWR-NUCLEAR-BATCH-001",
        "staging_glb_exists": mesh_glb.is_file(),
        "validate_asset_glb": validate_status,
        "constituent_modules_baked": baked,
        "fence_baked": fence_baked,
        "grid_units": grid_units,
        "meshes_path": KIT_MESHES_REL,
        "green": green,
        "_agent_meta": {
            "schema": "mcp_pwr_nuclear_batch_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "MCP_PWR_NUCLEAR_BATCH",
            "source_system": "mcp_pwr_nuclear",
            "relative_path": BATCH_WITNESS_REL,
        },
    }
    out = root / BATCH_WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = BATCH_WITNESS_REL
    return body


def refresh_nuclear_promote_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = _root(repo)
    glb_rel = f"assets/models/modules/{KIT_JOB_ID}/model.glb"
    promoted = (root / glb_rel).is_file()
    catalog_path = root / "assets/configs/buildings" / f"{CATALOG_ID}.json"
    ship_path_set = False
    catalog: dict[str, Any] = {}
    if catalog_path.is_file():
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        ship_path_set = bool(catalog.get("procedural_module_id")) and bool(catalog.get("model_glb"))
    green = promoted and ship_path_set and catalog.get("procedural_module_id") == KIT_ASSET_ID
    body: dict[str, Any] = {
        "gate": "MCP-PWR-PROMOTE-NUCLEAR-001",
        "promoted": promoted,
        "registry_row": CATALOG_ID,
        "ship_path_set": ship_path_set,
        "model_glb": catalog.get("model_glb"),
        "procedural_module_id": catalog.get("procedural_module_id"),
        "green": green,
        "_agent_meta": {
            "schema": "mcp_pwr_nuclear_promote_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "MCP_PWR_NUCLEAR_PROMOTE",
            "source_system": "mcp_pwr_nuclear",
            "relative_path": PROMOTE_WITNESS_REL,
        },
    }
    out = root / PROMOTE_WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = PROMOTE_WITNESS_REL
    return body
