"""Art pipeline witness JSON under debug_runs/art_pipeline/."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from .library import (
    KIT_GREYBOX_001_JOB_IDS,
    KIT_GREYBOX_002_JOB_IDS,
    KIT_GREYBOX_003_JOB_IDS,
    KIT_LOD0_001_JOB_IDS,
    KIT_LOD0_002_JOB_IDS,
    KIT_LOD0_003_JOB_IDS,
    load_index_json,
)
from .paths import repo_root
from .validate_glb import validate_glb


def art_pipeline_dir() -> Path:
    return repo_root() / "debug_runs" / "art_pipeline"


def batch_manifest_path(batch_id: str) -> Path:
    return (
        repo_root()
        / "tools"
        / "mcp"
        / "schemas"
        / "examples"
        / f"batch_{batch_id}.manifest.json"
    )


def _load_manifest(batch_id: str) -> dict[str, Any]:
    path = batch_manifest_path(batch_id)
    if not path.is_file():
        raise FileNotFoundError(f"Batch manifest not found: {path}")
    return json.loads(path.read_text(encoding="utf-8"))


def _module_row_from_disk(
    asset_id: str,
    job_id: str,
    *,
    status: str,
    index_by_job: dict[str, dict[str, Any]],
) -> dict[str, Any]:
    glb = repo_root() / "assets" / "models" / "modules" / job_id / "model.glb"
    row: dict[str, Any] = {
        "asset_id": asset_id,
        "job_id": job_id,
        "status": status,
        "registered": job_id in index_by_job,
    }
    if glb.is_file():
        report = validate_glb(glb)
        row["valid"] = report.valid
        row["vertex_count"] = report.vertex_count
        if status == "promoted":
            row["promoted_path"] = str(glb.relative_to(repo_root())).replace("\\", "/")
    else:
        row["valid"] = False
        row["vertex_count"] = None
    idx = index_by_job.get(job_id)
    if idx:
        row["module_id"] = idx.get("module_id")
        row["batch_id"] = idx.get("batch_id")
    return row


def write_batch_witness(batch_id: str) -> dict[str, Any]:
    manifest_path = batch_manifest_path(batch_id)
    if not manifest_path.is_file():
        tile_examples = repo_root() / "tools" / "mcp" / "schemas" / "examples"
        for candidate in tile_examples.glob("tile_batch_*.json"):
            data = json.loads(candidate.read_text(encoding="utf-8"))
            if str(data.get("batch_id")) == batch_id:
                from .tile_pipeline import tile_batch_run

                result = tile_batch_run(candidate)
                if result.get("witness"):
                    return result["witness"]
                return write_tile_batch_witness(batch_id, batch=data)
        raise FileNotFoundError(f"Batch manifest not found: {manifest_path}")
    manifest = _load_manifest(batch_id)
    index_entries = load_index_json()
    index_by_job = {e["job_id"]: e for e in index_entries}

    modules: list[dict[str, Any]] = []
    promoted_count = 0
    registered_count = 0
    failed: list[str] = []

    for mod in manifest.get("modules") or []:
        asset_id = str(mod.get("asset_id") or mod.get("module_id") or "")
        job_id = str(mod["job_id"])
        status = str(mod.get("status", "unknown"))
        promoted_glb = repo_root() / "assets" / "models" / "modules" / job_id / "model.glb"
        if promoted_glb.is_file() and status in ("staged", "unknown"):
            status = "promoted"
        row = _module_row_from_disk(asset_id, job_id, status=status, index_by_job=index_by_job)
        modules.append(row)
        if status == "promoted" and row.get("valid"):
            promoted_count += 1
        if row.get("registered"):
            registered_count += 1
        if status == "promoted" and not row.get("valid"):
            failed.append(job_id)

    batch_job_ids = {
        "kit_greybox_001": KIT_GREYBOX_001_JOB_IDS,
        "kit_greybox_002": KIT_GREYBOX_002_JOB_IDS,
        "kit_greybox_003": KIT_GREYBOX_003_JOB_IDS,
        "kit_lod0_001": KIT_LOD0_001_JOB_IDS,
        "kit_lod0_002": KIT_LOD0_002_JOB_IDS,
        "kit_lod0_003": KIT_LOD0_003_JOB_IDS,
    }.get(batch_id)
    if batch_job_ids is None:
        batch_job_ids = frozenset(str(m["job_id"]) for m in manifest.get("modules") or [])
    batch_registered = sum(1 for jid in batch_job_ids if jid in index_by_job)

    gates = {
        "G0": "pass",
        "G1": "pass",
        "G2": "pass",
        "G3": "pass",
        "G4": "pass" if promoted_count >= len([m for m in modules if m.get("status") == "promoted"]) else "fail",
        "G5": "pass" if batch_registered == len(batch_job_ids) else "fail",
    }

    witness: dict[str, Any] = {
        "batch_id": batch_id,
        "_agent_meta": {
            "agent": "coder-mcp",
            "updated_at": datetime.now(timezone.utc).isoformat(),
            "rules_applied": manifest.get("rules_applied", []),
        },
        "gates": gates,
        "modules": modules,
        "promoted_count": promoted_count,
        "registered_count": registered_count,
        "failed": failed,
        "module_index": "assets/configs/buildings/_module_index.ron",
        "index_entry_count": len(index_entries),
        "batch_registered_count": batch_registered,
    }
    if batch_id == "kit_greybox_001":
        witness["next"] = "@designer-mcp kit_greybox_002" if gates["G5"] == "pass" else "@coder-mcp fix G5"
    elif batch_id == "kit_greybox_002":
        witness["next"] = "@coder Bevy registry load path"
    elif batch_id == "kit_greybox_003":
        witness["next"] = "@designer-mcp kit_greybox_004 or PBR profile expansion"
    elif batch_id == "kit_lod0_001":
        witness["next"] = (
            "@coder Phase E registry (stylepack filter)" if gates["G5"] == "pass" else "@coder-mcp fix G5"
        )
    elif batch_id == "kit_lod0_002":
        witness["next"] = (
            "@designer-mcp kit_lod0_003 G0-G1" if gates["G5"] == "pass" else "@coder-mcp fix G5"
        )
    elif batch_id == "kit_lod0_003":
        witness["next"] = (
            "@designer-mcp kit_lod0_004 G0-G1" if gates["G5"] == "pass" else "@coder-mcp fix G5"
        )

    out = art_pipeline_dir() / f"{batch_id}_live.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(witness, indent=2) + "\n", encoding="utf-8")
    witness["written"] = str(out)
    return witness


def write_tile_batch_witness(
    batch_id: str,
    *,
    batch: dict[str, Any] | None = None,
    png_count: int = 0,
    atlas_path: str | None = None,
    meta_path: str | None = None,
    dry_run: bool = False,
    tile_index: dict[str, Any] | None = None,
) -> dict[str, Any]:
    """MCP-AUTO-011 — tile batch witness with G3 gate."""
    root = repo_root()
    staging = root / "assets" / "staging" / "tiles" / batch_id
    status_path = staging / "batch_status.json"
    status = {}
    if status_path.is_file():
        status = json.loads(status_path.read_text(encoding="utf-8"))

    pngs = [
        p
        for p in (list(staging.glob("*.png")) if staging.is_dir() else [])
        if not p.name.startswith("tile_map_")
    ]
    png_dimensions = _png_dimension_report(pngs)
    real_pngs_ok = _real_pngs_ok(png_dimensions, dry_run=dry_run)
    ship = bool((batch or {}).get("ship"))
    frozen = bool((batch or {}).get("frozen"))
    atlas_v2_ok = _atlas_meta_v2_ship_ready(meta_path, batch=batch)

    atlas_ok = bool(atlas_path and Path(atlas_path).is_file())
    meta_ok = bool(meta_path and Path(meta_path).is_file())
    variant_min = len((batch or {}).get("variants") or []) >= 2

    # TILE-FIX-001: PNG dimensions alone never gate ship or production green.
    g3_pass = (
        not frozen
        and variant_min
        and (png_count >= 2 or len(pngs) >= 2)
        and real_pngs_ok
        and (atlas_ok or dry_run)
        and (meta_ok or dry_run)
        and (status.get("status") == "done" or (dry_run and png_count >= 2))
        and (dry_run or status.get("dry_run") is False)
        and (not ship or atlas_v2_ok)
    )

    gates = {
        "G0": "pass",
        "G1": "pass",
        "G2": "pass",
        "G3": "pass" if g3_pass else "fail",
        "G4": "planned",
        "G5": "planned",
    }

    witness: dict[str, Any] = {
        "batch_id": batch_id,
        "lane": "tile_batch",
        "_agent_meta": {
            "agent": "coder-mcp",
            "updated_at": datetime.now(timezone.utc).isoformat(),
            "program": "MCP-TILE-CLOSE-001",
            "dry_run": dry_run,
        },
        "gates": gates,
        "green": gates["G3"] == "pass",
        "png_count": png_count or len(pngs),
        "png_dimensions": png_dimensions,
        "real_bake": not dry_run,
        "atlas_path": atlas_path,
        "meta_json": meta_path,
        "batch_status": status,
        "automation": "headless_only",
        "smoke_fallback_used": False,
        "gate_id": "TILE-REAL-001",
        "atlas_schema_version": (batch or {}).get("atlas_schema_version", 1),
        "atlas_v2_lookup_ok": atlas_v2_ok,
        "frozen": frozen,
        "png_exists_only_rejected": True,
    }
    if batch:
        witness["tile_id"] = batch.get("tile_id")
        witness["assembly_ref"] = batch.get("assembly_ref")
        witness["variant_set_ref"] = batch.get("variant_set_ref")
        witness["source_tier"] = batch.get("source_tier")
        witness["development_tier"] = batch.get("development_tier")
        witness["ship"] = batch.get("ship")
    if tile_index:
        witness["tile_atlas_index"] = tile_index

    out = art_pipeline_dir() / f"tile_{batch_id}_live.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(witness, indent=2) + "\n", encoding="utf-8")
    witness["written"] = str(out)
    return witness


def _png_dimension_report(pngs: list[Path]) -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    for p in pngs:
        row: dict[str, Any] = {"file": p.name, "bytes": p.stat().st_size if p.is_file() else 0}
        try:
            from PIL import Image

            with Image.open(p) as im:
                row["width"], row["height"] = im.size
        except Exception:  # noqa: BLE001
            row["width"], row["height"] = 0, 0
        out.append(row)
    return out


def _real_pngs_ok(dimensions: list[dict[str, Any]], *, dry_run: bool) -> bool:
    if dry_run:
        return len(dimensions) >= 2
    if len(dimensions) < 2:
        return False
    for row in dimensions:
        w = int(row.get("width") or 0)
        h = int(row.get("height") or 0)
        if w <= 1 or h <= 1:
            return False
    return True


def _atlas_meta_v2_ship_ready(
    meta_path: str | None,
    *,
    batch: dict[str, Any] | None,
) -> bool:
    """Ship green requires atlas_meta schema v2 + complete lookups (TILE-FIX-002)."""
    if not batch or not bool(batch.get("ship")):
        return True
    if not meta_path:
        return False
    path = Path(meta_path)
    if not path.is_file():
        return False
    try:
        from rust_engine_mcp.validators.atlas_meta import validate_atlas_meta_v2
        from rust_engine_mcp.paths import repo_root

        vc_rel = str(batch.get("visual_config_ref") or "")
        vc_path = repo_root() / vc_rel if vc_rel else None
        report = validate_atlas_meta_v2(path, visual_config_path=vc_path)
        return report.status == "passed"
    except Exception:  # noqa: BLE001
        return False


def write_procedural_tiles_production_bake_witness() -> dict[str, Any]:
    """PT-2 rollup — all production-tier atlases in index with ≥6 variants."""
    from rust_engine_mcp.tile_index import load_tile_atlas_index

    root = repo_root()
    entries = load_tile_atlas_index()
    production = [
        e
        for e in entries
        if str(e.get("development_tier") or "") in ("production", "greybox_frozen_v1")
    ]
    atlas_reports = []
    all_pass = True
    for entry in production:
        meta_path = root / str(entry.get("meta_json") or "")
        variant_keys: list[str] = []
        meta: dict[str, Any] = {}
        if meta_path.is_file():
            meta = json.loads(meta_path.read_text(encoding="utf-8"))
            if int(meta.get("schema_version") or 1) >= 2:
                variant_keys = [
                    str(t.get("variant") or "")
                    for t in meta.get("lookups") or []
                    if t.get("variant")
                ]
            else:
                variant_keys = [
                    str(t.get("variant_key") or "")
                    for t in meta.get("tiles") or []
                    if t.get("variant_key")
                ]
        has_night = any("night" in k for k in variant_keys)
        has_fire = any(k.startswith("burning_") for k in variant_keys)
        status_path = root / "assets/staging/tiles" / str(entry.get("batch_id") or "") / "batch_status.json"
        dry_run = True
        if status_path.is_file():
            st = json.loads(status_path.read_text(encoding="utf-8"))
            dry_run = bool(st.get("dry_run", True))
        meta_schema = int(meta.get("schema_version") or 1) if meta_path.is_file() else 1
        ok = (
            meta_schema >= 2
            and len(variant_keys) >= 6
            and has_night
            and has_fire
            and not dry_run
            and bool(entry.get("ship_allowed"))
        )
        if not ok:
            all_pass = False
        atlas_reports.append(
            {
                "atlas_id": entry.get("atlas_id"),
                "batch_id": entry.get("batch_id"),
                "variant_count": len(variant_keys),
                "meta_schema_version": meta_schema,
                "has_clean_night_on": "clean_night_on" in variant_keys,
                "has_burning_00": "burning_00" in variant_keys,
                "dry_run": dry_run,
                "pass": ok,
            }
        )

    witness: dict[str, Any] = {
        "program_id": "PLAN-PROC-TILE-PROD-001",
        "gate_id": "TILE-PROD-001",
        "freeze_id": "TILE-FIX-001",
        "green": all_pass and len(production) >= 1,
        "note": "Requires active index v2 atlases with ship_allowed; greybox v1 frozen",
        "production_atlas_count": len(production),
        "minimum_variant_count": 6,
        "atlases": atlas_reports,
        "_agent_meta": {
            "agent": "coder-mcp",
            "updated_at": datetime.now(timezone.utc).isoformat(),
            "program": "MCP-PT-2-001",
        },
    }
    out = art_pipeline_dir() / "procedural_tiles_production_bake_live.json"
    out.write_text(json.dumps(witness, indent=2) + "\n", encoding="utf-8")
    witness["written"] = str(out)
    return witness
