"""Promote validated staging assets to assets/models/modules/."""

from __future__ import annotations

import json
import shutil
from pathlib import Path

from .blender_runner import read_status
from .library import (
    KIT_GREYBOX_001_JOB_IDS,
    KIT_GREYBOX_002_JOB_IDS,
    KIT_GREYBOX_003_JOB_IDS,
    KIT_INDUSTRIAL_WEST_PRODUCTION_001_JOB_IDS,
    KIT_LOD0_001_JOB_IDS,
    KIT_LOD0_002_JOB_IDS,
    KIT_LOD0_003_JOB_IDS,
    KIT_PRODUCTION_002_JOB_IDS,
    KIT_UTILITY_POWER_PRODUCTION_001_JOB_IDS,
    KIT_NUCLEAR_PWR_PRODUCTION_001_JOB_IDS,
    register_module,
)
from .paths import repo_root, staging_root
from .schemas import load_json_file
from .validators.asset import validate_asset_glb


def _resolve_spec_path(job_id: str) -> Path | None:
    specs_dir = staging_root() / "specs"
    direct = specs_dir / f"{job_id}.json"
    if direct.is_file():
        return direct

    status = read_status(job_id) or {}
    spec_ref = status.get("spec_ref")
    if spec_ref:
        spec_path = Path(str(spec_ref))
        if not spec_path.is_absolute():
            spec_path = repo_root() / spec_path
        if spec_path.is_file():
            return spec_path

    job_path = status.get("job_path")
    if job_path:
        job_file = Path(str(job_path))
        if job_file.is_file():
            spec_ref = load_json_file(job_file).get("spec_ref")
            if spec_ref:
                spec_path = Path(str(spec_ref))
                if not spec_path.is_absolute():
                    spec_path = repo_root() / spec_path
                if spec_path.is_file():
                    return spec_path

    examples = repo_root() / "tools" / "mcp" / "schemas" / "examples"
    if examples.is_dir():
        for candidate in examples.glob("*.json"):
            try:
                job = load_json_file(candidate)
            except (OSError, ValueError, KeyError):
                continue
            if job.get("job_id") != job_id:
                continue
            spec_ref = job.get("spec_ref")
            if not spec_ref:
                continue
            spec_path = Path(str(spec_ref))
            if not spec_path.is_absolute():
                spec_path = repo_root() / spec_path
            if spec_path.is_file():
                return spec_path
    return None


def _infer_batch_id(job_id: str) -> str:
    if job_id in KIT_GREYBOX_001_JOB_IDS:
        return "kit_greybox_001"
    if job_id in KIT_GREYBOX_002_JOB_IDS:
        return "kit_greybox_002"
    if job_id in KIT_GREYBOX_003_JOB_IDS:
        return "kit_greybox_003"
    if job_id in KIT_LOD0_001_JOB_IDS:
        return "kit_lod0_001"
    if job_id in KIT_LOD0_002_JOB_IDS:
        return "kit_lod0_002"
    if job_id in KIT_LOD0_003_JOB_IDS:
        return "kit_lod0_003"
    if job_id in KIT_INDUSTRIAL_WEST_PRODUCTION_001_JOB_IDS:
        return "kit_industrial_west_production_001"
    if job_id in KIT_PRODUCTION_002_JOB_IDS:
        return "kit_production_002"
    if job_id in KIT_UTILITY_POWER_PRODUCTION_001_JOB_IDS:
        return "kit_utility_power_production_001"
    if job_id in KIT_NUCLEAR_PWR_PRODUCTION_001_JOB_IDS:
        return "kit_nuclear_pwr_production_001"
    return ""


def promote_module(
    job_id: str,
    *,
    force: bool = False,
    register: bool = True,
    allow_smoke: bool = False,
) -> dict:
    src_dir = staging_root() / job_id
    glb = src_dir / "model.glb"
    if not glb.is_file():
        candidates = list(src_dir.glob("**/*.glb"))
        if not candidates:
            raise FileNotFoundError(f"No glb under {src_dir}")
        glb = candidates[0]

    spec_path = _resolve_spec_path(job_id)
    if spec_path is not None:
        spec = load_json_file(spec_path)
        tier = str(spec.get("development_tier") or "")
        if tier == "production":
            if spec.get("pbr_status") != "shipped":
                raise ValueError(
                    f"production spec {spec_path.name} requires pbr_status: shipped"
                )
            mat = str(
                spec.get("material_profile")
                or spec.get("tileable_set_id")
                or spec.get("material_id")
                or ""
            ).strip()
            if not mat:
                raise ValueError(
                    f"production spec {spec_path.name} requires material_profile or tileable_set_id"
                )
            from rust_engine_mcp.validators.material_textures import validate_material_textures

            mat_rep = validate_material_textures(spec, spec_path=str(spec_path), ship=True)
            if mat_rep.status == "failed":
                hints = [e.hint for e in mat_rep.errors if e.severity == "error"][:3]
                raise ValueError(
                    f"production material textures missing (TILE-FIX-005): {hints or mat_rep.summary}"
                )

    tier_report = validate_asset_glb(glb, compression_level=1)
    if tier_report.status == "failed" and not force:
        hints = [e.hint for e in tier_report.errors if e.severity == "error"][:3]
        raise ValueError(f"Tier/asset validation failed: {hints or tier_report.summary}")

    if not allow_smoke:
        blocked = any(
            e.kind in ("SmokeAsProduction", "BatchRetired") and e.severity == "error"
            for e in tier_report.errors
        )
        if blocked:
            raise ValueError(
                "Smoke/harness batch blocked — use kit_lod0_* / kit_production_* or --allow-smoke"
            )

    raw_valid = tier_report.status in ("passed", "warning")
    if not raw_valid and not force:
        raise ValueError(f"Validation failed: {tier_report.summary}")

    dest_dir = repo_root() / "assets" / "models" / "modules" / job_id
    dest_dir.mkdir(parents=True, exist_ok=True)
    dest_glb = dest_dir / "model.glb"
    shutil.copy2(glb, dest_glb)

    sidecar_src = _resolve_spec_path(job_id)
    if sidecar_src is not None:
        shutil.copy2(sidecar_src, dest_dir / f"{job_id}.module.json")

    batch_id = _infer_batch_id(job_id)
    manifest = {
        "job_id": job_id,
        "glb": str(dest_glb.relative_to(repo_root())).replace("\\", "/"),
        "valid": raw_valid,
        "vertex_count": None,
        "batch_id": batch_id,
        "validation": tier_report.to_dict(),
    }
    for token in tier_report.summary.split():
        if token.startswith("verts="):
            try:
                manifest["vertex_count"] = int(token.split("=", 1)[1])
            except ValueError:
                pass
    (dest_dir / "manifest.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")

    if register and raw_valid:
        reg = register_module(job_id)
        manifest["library"] = {
            "registered": reg.get("registered"),
            "entry_count": reg.get("entry_count"),
        }
    return manifest
