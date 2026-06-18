"""MCP-PWR-UTILITY — manifest, bpy batch, promote for substation + transformer."""

from __future__ import annotations

import json
import shutil
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

MANIFEST_REL = "tools/mcp/schemas/examples/batch_kit_utility_power_production_001.manifest.json"
BATCH_ID = "kit_utility_power_production_001"

MANIFEST_WITNESS_REL = "debug_runs/art_pipeline/mcp_pwr_utility_manifest_live.json"
SUBSTATION_BATCH_WITNESS_REL = "debug_runs/art_pipeline/mcp_pwr_substation_batch_live.json"
TRANSFORMER_BATCH_WITNESS_REL = "debug_runs/art_pipeline/mcp_pwr_transformer_batch_live.json"
SUBSTATION_PROMOTE_WITNESS_REL = "debug_runs/art_pipeline/mcp_pwr_substation_promote_live.json"
TRANSFORMER_PROMOTE_WITNESS_REL = "debug_runs/art_pipeline/mcp_pwr_transformer_promote_live.json"

SUBSTATION_SPEC_REL = "assets/staging/specs/kit_substation_yard_production_001.json"
TRANSFORMER_SPEC_REL = "assets/staging/specs/prop_transformer_production_run001.json"
SUBSTATION_JOB_ID = "kit_substation_yard_production_run001"
TRANSFORMER_JOB_ID = "prop_transformer_production_run001"
SUBSTATION_ASSET_ID = "kit_substation_yard_production_001"
TRANSFORMER_ASSET_ID = "prop_transformer_production_run001"
SUBSTATION_CATALOG_ID = "grid_substation"
TRANSFORMER_CATALOG_ID = "grid_distribution_transformer"
SUBSTATION_MESHES_REL = "assets/staging/meshes/kit_substation_yard_production_001/model.glb"

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


CONSTITUENT_STUBS: tuple[ModuleStub, ...] = (
    ModuleStub(
        "bus_bay_simplified",
        "bus_bay_simplified_production_run001",
        "assets/staging/specs/bus_bay_simplified_production_run001.json",
        (2, 1),
        (4.0, 2.5, 2.0),
        "galvanized_steel_01",
        "bus_bay",
    ),
    ModuleStub(
        "breaker_block",
        "breaker_block_production_run001",
        "assets/staging/specs/breaker_block_production_run001.json",
        (1, 1),
        (2.0, 2.0, 2.0),
        "galvanized_steel_01",
        "breaker",
    ),
    ModuleStub(
        "control_shack_1u",
        "control_shack_1u_production_run001",
        "assets/staging/specs/control_shack_1u_production_run001.json",
        (1, 1),
        (2.0, 2.5, 2.0),
        "galvanized_steel_01",
        "shack",
        snap="floor_center",
    ),
    ModuleStub(
        "fence_chainlink_1u",
        "fence_chainlink_1u_production_run001",
        "assets/staging/specs/fence_chainlink_1u_production_run001.json",
        (1, 1),
        (2.0, 2.0, 0.12),
        "galvanized_steel_01",
        "fence",
    ),
    ModuleStub(
        "gravel_pad_1u",
        "gravel_pad_1u_production_run001",
        "assets/staging/specs/gravel_pad_1u_production_run001.json",
        (1, 1),
        (2.0, 0.15, 2.0),
        "gravel_yard_01",
        "gravel_pad",
        snap="floor_center",
    ),
    ModuleStub(
        "warning_sign_1u",
        "warning_sign_1u_production_run001",
        "assets/staging/specs/warning_sign_1u_production_run001.json",
        (1, 1),
        (0.5, 1.5, 0.08),
        "warning_paint_yellow_01",
        "warning_sign",
        snap="floor_center",
    ),
)

KIT_UTILITY_POWER_PRODUCTION_001_JOB_IDS: frozenset[str] = frozenset(
    {stub.job_id for stub in CONSTITUENT_STUBS}
    | {SUBSTATION_JOB_ID, TRANSFORMER_JOB_ID}
)


def _root(repo: Path | None = None) -> Path:
    return repo or repo_root()


def _job_path(job_id: str, *, repo: Path | None = None) -> Path:
    return _root(repo) / "tools/mcp/schemas/examples" / f"{job_id}.json"


def load_manifest(*, repo: Path | None = None) -> dict[str, Any]:
    path = _root(repo) / MANIFEST_REL
    return json.loads(path.read_text(encoding="utf-8"))


def _spec_body(
    stub: ModuleStub,
    *,
    feeds: str | None = SUBSTATION_ASSET_ID,
) -> dict[str, Any]:
    w, h, d = stub.dims_m
    body: dict[str, Any] = {
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
        "references": [
            "ref:design:DES-ART-UTILITY-STYLE-001",
            "ref:dmcp:DMCP-SPEC-SUBSTATION-YARD-001",
            f"ref:kit:{SUBSTATION_ASSET_ID}",
        ],
    }
    if feeds:
        body["feeds"] = feeds
    return body


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
            "seed": 43001,
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
        "job_id": SUBSTATION_JOB_ID,
        "batch_id": BATCH_ID,
        "development_tier": "production",
        "spec_ref": SUBSTATION_SPEC_REL,
        "operation": "module_prop",
        "params": {
            "width_m": 8.0,
            "height_m": 3.0,
            "depth_m": 6.0,
            "material_profile": "galvanized_steel_01",
            "seed": 43010,
            "profile": "box",
            "prop_kind": "yard_kit",
            "name": SUBSTATION_ASSET_ID,
        },
        "output": {
            "glb": f"assets/staging/{SUBSTATION_JOB_ID}/model.glb",
            "thumbnail": f"assets/staging/{SUBSTATION_JOB_ID}/preview.png",
        },
    }


def _transformer_job_body() -> dict[str, Any]:
    return {
        "schema_version": 1,
        "job_id": TRANSFORMER_JOB_ID,
        "batch_id": BATCH_ID,
        "development_tier": "production",
        "spec_ref": TRANSFORMER_SPEC_REL,
        "operation": "module_prop",
        "params": {
            "width_m": 3.6,
            "height_m": 1.5,
            "depth_m": 3.6,
            "material_profile": "galvanized_steel_01",
            "seed": 43020,
            "profile": "box",
            "prop_kind": "transformer",
            "name": TRANSFORMER_ASSET_ID,
        },
        "output": {
            "glb": f"assets/staging/{TRANSFORMER_JOB_ID}/model.glb",
            "thumbnail": f"assets/staging/{TRANSFORMER_JOB_ID}/preview.png",
        },
    }


def ensure_utility_manifest_assets(*, repo: Path | None = None, force: bool = False) -> dict[str, Any]:
    root = _root(repo)
    written_specs: list[str] = []
    written_jobs: list[str] = []

    for stub in CONSTITUENT_STUBS:
        spec_path = root / stub.spec_rel
        if force or not spec_path.is_file():
            spec_path.parent.mkdir(parents=True, exist_ok=True)
            spec_path.write_text(json.dumps(_spec_body(stub), indent=2) + "\n", encoding="utf-8")
            written_specs.append(stub.spec_rel)
        job_path = _job_path(stub.job_id, repo=root)
        if force or not job_path.is_file():
            job_path.write_text(json.dumps(_job_body(stub), indent=2) + "\n", encoding="utf-8")
            written_jobs.append(stub.job_id)

    kit_job = _job_path(SUBSTATION_JOB_ID, repo=root)
    if force or not kit_job.is_file():
        kit_job.write_text(json.dumps(_kit_job_body(), indent=2) + "\n", encoding="utf-8")
        written_jobs.append(SUBSTATION_JOB_ID)

    xfm_job = _job_path(TRANSFORMER_JOB_ID, repo=root)
    if force or not xfm_job.is_file():
        xfm_job.write_text(json.dumps(_transformer_job_body(), indent=2) + "\n", encoding="utf-8")
        written_jobs.append(TRANSFORMER_JOB_ID)

    return {
        "written_specs": written_specs,
        "written_jobs": written_jobs,
        "constituent_count": len(CONSTITUENT_STUBS),
    }


def _validate_spec(rel: str, *, repo: Path | None = None) -> bool:
    from rust_engine_mcp.validators.mcp_schema import validate_mcp_spec

    path = _root(repo) / rel
    if not path.is_file():
        return False
    report = validate_mcp_spec(path, compression_level=COMPRESSION)
    return report.status in ("passed", "warning")


def audit_utility_manifest(*, repo: Path | None = None) -> dict[str, Any]:
    root = _root(repo)
    ensure = ensure_utility_manifest_assets(repo=root)
    manifest = load_manifest(repo=root)
    modules = list(manifest.get("modules") or [])
    substation_valid = _validate_spec(SUBSTATION_SPEC_REL, repo=root)
    transformer_valid = _validate_spec(TRANSFORMER_SPEC_REL, repo=root)
    missing_specs = [
        stub.spec_rel for stub in CONSTITUENT_STUBS if not (root / stub.spec_rel).is_file()
    ]
    missing_jobs = [
        stub.job_id for stub in CONSTITUENT_STUBS if not _job_path(stub.job_id, repo=root).is_file()
    ]
    if not _job_path(SUBSTATION_JOB_ID, repo=root).is_file():
        missing_jobs.append(SUBSTATION_JOB_ID)
    if not _job_path(TRANSFORMER_JOB_ID, repo=root).is_file():
        missing_jobs.append(TRANSFORMER_JOB_ID)

    green = (
        len(modules) >= 8
        and substation_valid
        and transformer_valid
        and not missing_specs
        and not missing_jobs
    )
    return {
        "gate": "MCP-PWR-UTILITY-MANIFEST-001",
        "manifest_modules": len(modules),
        "substation_spec_valid": substation_valid,
        "transformer_spec_valid": transformer_valid,
        "missing_specs": missing_specs,
        "missing_jobs": missing_jobs,
        "ensure": ensure,
        "manifest": MANIFEST_REL,
        "green": green,
    }


def refresh_pwr_utility_manifest_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = _root(repo)
    body = audit_utility_manifest(repo=root)
    body["_agent_meta"] = {
        "schema": "mcp_pwr_utility_manifest_live_v1",
        "written_at_epoch_secs": int(time.time()),
        "profile": "MCP_PWR_UTILITY_MANIFEST",
        "source_system": "mcp_pwr_utility",
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


def run_substation_batch(*, repo: Path | None = None) -> dict[str, Any]:
    root = _root(repo)
    ensure_utility_manifest_assets(repo=root)
    constituent_results = [_run_job(stub.job_id, repo=root) for stub in CONSTITUENT_STUBS]
    kit_result = _run_job(SUBSTATION_JOB_ID, repo=root)
    kit_staging = root / "assets/staging" / SUBSTATION_JOB_ID / "model.glb"
    mesh_out = root / SUBSTATION_MESHES_REL
    mesh_out.parent.mkdir(parents=True, exist_ok=True)
    if kit_staging.is_file():
        shutil.copy2(kit_staging, mesh_out)
    baked = sum(1 for r in constituent_results if r.get("ok"))
    return {
        "constituent_results": constituent_results,
        "kit_result": kit_result,
        "constituent_modules_baked": baked,
        "kit_staging_glb": kit_staging.is_file(),
        "meshes_glb": mesh_out.is_file(),
        "ok": baked >= 4 and kit_result.get("ok") and mesh_out.is_file(),
    }


def run_transformer_batch(*, repo: Path | None = None) -> dict[str, Any]:
    root = _root(repo)
    ensure_utility_manifest_assets(repo=root)
    result = _run_job(TRANSFORMER_JOB_ID, repo=root)
    spec = json.loads((root / TRANSFORMER_SPEC_REL).read_text(encoding="utf-8"))
    grid_units = (spec.get("module") or {}).get("grid_units")
    return {
        "transformer_result": result,
        "grid_units": grid_units,
        "staging_glb_exists": (root / "assets/staging" / TRANSFORMER_JOB_ID / "model.glb").is_file(),
        "ok": bool(result.get("ok")),
    }


def _prepare_spec_for_promote(spec_path: Path) -> None:
    from rust_engine_mcp.material_textures import PILOT_PROFILES, generate_profile, write_registry

    spec = json.loads(spec_path.read_text(encoding="utf-8"))
    mat = str(spec.get("material_profile") or "").strip()
    if mat in PILOT_PROFILES:
        generate_profile(PILOT_PROFILES[mat])
    write_registry()
    spec["pbr_status"] = "shipped"
    spec_path.write_text(json.dumps(spec, indent=2) + "\n", encoding="utf-8")


def _update_catalog_module_id(catalog_id: str, module_id: str, glb_rel: str, *, repo: Path) -> None:
    catalog_path = repo / "assets/configs/buildings" / f"{catalog_id}.json"
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


def promote_substation(*, register: bool = True, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp import promote
    from rust_engine_mcp.library import write_module_index

    root = _root(repo)
    spec_path = root / SUBSTATION_SPEC_REL
    _prepare_spec_for_promote(spec_path)
    manifest = promote.promote_module(SUBSTATION_JOB_ID, register=register)
    index = write_module_index()
    glb_rel = f"assets/models/modules/{SUBSTATION_JOB_ID}/model.glb"
    _update_catalog_module_id(SUBSTATION_CATALOG_ID, SUBSTATION_ASSET_ID, glb_rel, repo=root)
    _sync_manifest_status(SUBSTATION_JOB_ID, repo=root)
    return {"promoted": manifest, "index_entries": index.get("entry_count"), "glb_rel": glb_rel}


def promote_transformer(*, register: bool = True, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp import promote
    from rust_engine_mcp.library import write_module_index

    root = _root(repo)
    spec_path = root / TRANSFORMER_SPEC_REL
    _prepare_spec_for_promote(spec_path)
    manifest = promote.promote_module(TRANSFORMER_JOB_ID, register=register)
    index = write_module_index()
    glb_rel = f"assets/models/modules/{TRANSFORMER_JOB_ID}/model.glb"
    _update_catalog_module_id(TRANSFORMER_CATALOG_ID, TRANSFORMER_ASSET_ID, glb_rel, repo=root)
    _sync_manifest_status(TRANSFORMER_JOB_ID, repo=root)
    lod0_glb = root / "assets/models/modules/prop_transformer_lod0_run001/model.glb"
    production_glb = root / "assets/models/modules" / TRANSFORMER_JOB_ID / "model.glb"
    return {
        "promoted": manifest,
        "index_entries": index.get("entry_count"),
        "glb_rel": glb_rel,
        "supersedes_lod0_stub": lod0_glb.is_file() and production_glb.is_file(),
    }


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


def refresh_substation_batch_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = _root(repo)
    mesh_glb = root / SUBSTATION_MESHES_REL
    baked = sum(
        1
        for stub in CONSTITUENT_STUBS
        if (root / "assets/staging" / stub.job_id / "model.glb").is_file()
    )
    validate_status = _asset_glb_status(SUBSTATION_MESHES_REL, repo=root, staging=True) if mesh_glb.is_file() else "missing"
    green = mesh_glb.is_file() and validate_status in ("passed", "warning") and baked >= 4
    body: dict[str, Any] = {
        "gate": "MCP-PWR-SUBSTATION-BATCH-001",
        "staging_glb_exists": mesh_glb.is_file(),
        "validate_asset_glb": validate_status,
        "constituent_modules_baked": baked,
        "meshes_path": SUBSTATION_MESHES_REL,
        "green": green,
        "_agent_meta": {
            "schema": "mcp_pwr_substation_batch_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "MCP_PWR_SUBSTATION_BATCH",
            "source_system": "mcp_pwr_utility",
            "relative_path": SUBSTATION_BATCH_WITNESS_REL,
        },
    }
    out = root / SUBSTATION_BATCH_WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = SUBSTATION_BATCH_WITNESS_REL
    return body


def refresh_transformer_batch_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = _root(repo)
    staging = root / "assets/staging" / TRANSFORMER_JOB_ID / "model.glb"
    spec = json.loads((root / TRANSFORMER_SPEC_REL).read_text(encoding="utf-8"))
    grid_units = (spec.get("module") or {}).get("grid_units")
    validate_status = _asset_glb_status(
        f"assets/staging/{TRANSFORMER_JOB_ID}/model.glb", repo=root, staging=True
    ) if staging.is_file() else "missing"
    green = staging.is_file() and validate_status in ("passed", "warning") and grid_units == [2, 2]
    body: dict[str, Any] = {
        "gate": "MCP-PWR-TRANSFORMER-BATCH-001",
        "staging_glb_exists": staging.is_file(),
        "validate_asset_glb": validate_status,
        "grid_units": grid_units,
        "green": green,
        "_agent_meta": {
            "schema": "mcp_pwr_transformer_batch_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "MCP_PWR_TRANSFORMER_BATCH",
            "source_system": "mcp_pwr_utility",
            "relative_path": TRANSFORMER_BATCH_WITNESS_REL,
        },
    }
    out = root / TRANSFORMER_BATCH_WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = TRANSFORMER_BATCH_WITNESS_REL
    return body


def refresh_substation_promote_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = _root(repo)
    glb_rel = f"assets/models/modules/{SUBSTATION_JOB_ID}/model.glb"
    promoted = (root / glb_rel).is_file()
    catalog = json.loads((root / "assets/configs/buildings" / f"{SUBSTATION_CATALOG_ID}.json").read_text())
    ship_path_set = bool(catalog.get("procedural_module_id")) and bool(catalog.get("model_glb"))
    green = promoted and ship_path_set and catalog.get("procedural_module_id") == SUBSTATION_ASSET_ID
    body: dict[str, Any] = {
        "gate": "MCP-PWR-PROMOTE-SUBSTATION-001",
        "promoted": promoted,
        "registry_row": SUBSTATION_CATALOG_ID,
        "ship_path_set": ship_path_set,
        "model_glb": catalog.get("model_glb"),
        "procedural_module_id": catalog.get("procedural_module_id"),
        "green": green,
        "_agent_meta": {
            "schema": "mcp_pwr_substation_promote_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "MCP_PWR_SUBSTATION_PROMOTE",
            "source_system": "mcp_pwr_utility",
            "relative_path": SUBSTATION_PROMOTE_WITNESS_REL,
        },
    }
    out = root / SUBSTATION_PROMOTE_WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = SUBSTATION_PROMOTE_WITNESS_REL
    return body


def refresh_transformer_promote_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = _root(repo)
    glb_rel = f"assets/models/modules/{TRANSFORMER_JOB_ID}/model.glb"
    promoted = (root / glb_rel).is_file()
    catalog = json.loads((root / "assets/configs/buildings" / f"{TRANSFORMER_CATALOG_ID}.json").read_text())
    lod0 = root / "assets/models/modules/prop_transformer_lod0_run001/model.glb"
    production = root / glb_rel
    supersedes = lod0.is_file() and production.is_file()
    green = promoted and catalog.get("procedural_module_id") == TRANSFORMER_ASSET_ID and supersedes
    body: dict[str, Any] = {
        "gate": "MCP-PWR-PROMOTE-TRANSFORMER-001",
        "promoted": promoted,
        "registry_row": TRANSFORMER_CATALOG_ID,
        "supersedes_lod0_stub": supersedes,
        "procedural_module_id": catalog.get("procedural_module_id"),
        "model_glb": catalog.get("model_glb"),
        "green": green,
        "_agent_meta": {
            "schema": "mcp_pwr_transformer_promote_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "MCP_PWR_TRANSFORMER_PROMOTE",
            "source_system": "mcp_pwr_utility",
            "relative_path": TRANSFORMER_PROMOTE_WITNESS_REL,
        },
    }
    out = root / TRANSFORMER_PROMOTE_WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = TRANSFORMER_PROMOTE_WITNESS_REL
    return body
