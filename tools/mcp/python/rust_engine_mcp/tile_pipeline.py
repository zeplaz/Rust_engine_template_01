"""Tile atlas + lod0 batch + automated tile_batch_run — CLI/MCP/viewer shared."""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Any

from rust_engine_mcp import assembly, blender_runner
from rust_engine_mcp.paths import jobs_root, repo_root
from rust_engine_mcp.schemas import load_json_file
from rust_engine_mcp.tile_index import register_tile_atlas_from_meta
from rust_engine_mcp.validators import run_validator
from rust_engine_mcp.witness import write_tile_batch_witness

# Minimal valid 1×1 PNG for dry-run bakes (pytest / no Blender).
_MINIMAL_PNG = (
    b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01"
    b"\x08\x06\x00\x00\x00\x1f\x15\xc4\x89\x00\x00\x00\nIDATx\x9cc\x00\x01"
    b"\x00\x00\x05\x00\x01\r\n-\xb4\x00\x00\x00\x00IEND\xaeB`\x82"
)


def light_setup_blend_path() -> Path:
    env = os.environ.get("RUST_ENGINE_TILE_LIGHT_BLEND")
    if env:
        return Path(env).resolve()
    return repo_root() / "utils" / "Tile_iso_rig_v1.blend"


def tile_dry_run_enabled() -> bool:
    return os.environ.get("RUST_ENGINE_TILE_DRY_RUN", "").strip() in ("1", "true", "TRUE")


def tile_keyframe_headless_enabled() -> bool:
    """Optional headless keyframe export (parity with civ truck) — after manual G4 green."""
    return os.environ.get("RUST_ENGINE_TILE_KEYFRAME_HEADLESS", "").strip() in (
        "1",
        "true",
        "TRUE",
    )


def _variant_key(variant: dict[str, Any]) -> str:
    if variant.get("variant_key"):
        return str(variant["variant_key"])
    state = variant.get("state", "clean")
    damage = variant.get("damage", 0.0)
    power = variant.get("power", "off")
    fill = variant.get("fill", "empty")
    lighting = variant.get("lighting", "day")
    return f"{state}_d{int(float(damage) * 100):02d}_{power}_{fill}_{lighting}"


def _write_stub_png(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(_MINIMAL_PNG)


def _dry_run_tile_job(job: dict[str, Any]) -> blender_runner.JobResult:
    job_id = str(job["job_id"])
    out = job.get("output") or {}
    outputs: list[str] = []
    if job.get("operation") == "assembly_build":
        blend = out.get("blend")
        if blend:
            p = Path(blend)
            if not p.is_absolute():
                p = repo_root() / blend
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text('{"dry_run": true}\n', encoding="utf-8")
            outputs.append(str(p.resolve()))
    elif job.get("operation") == "tile_variant_bake":
        png = out.get("png")
        if png:
            p = Path(png)
            if not p.is_absolute():
                p = repo_root() / png
            _write_stub_png(p)
            outputs.append(str(p.resolve()))
    blender_runner.write_status(
        job_id,
        {
            "job_id": job_id,
            "status": "done",
            "outputs": outputs,
            "dry_run": True,
            "operation": job.get("operation"),
        },
    )
    return blender_runner.JobResult(
        job_id=job_id, status="done", log_path="", outputs=outputs, error=None
    )


def _run_tile_job_path(job_path: Path) -> blender_runner.JobResult:
    if tile_dry_run_enabled():
        job = load_json_file(job_path)
        return _dry_run_tile_job(job)
    return blender_runner.run_tile_job(job_path)


def _write_atlas_meta(
    batch: dict[str, Any],
    variant_keys: list[str],
    png_paths: list[str],
    atlas_path: str | None,
) -> Path:
    atlas = batch.get("atlas") or {}
    meta_rel = str(atlas.get("meta_json") or "")
    if not meta_rel:
        batch_id = str(batch.get("batch_id") or "tile_batch")
        meta_rel = f"assets/staging/tiles/{batch_id}/atlas_meta.json"
    meta_path = Path(meta_rel)
    if not meta_path.is_absolute():
        meta_path = repo_root() / meta_rel
    meta_path.parent.mkdir(parents=True, exist_ok=True)

    cols = int(atlas.get("columns") or 4)
    rows = int(atlas.get("rows") or max(1, (len(variant_keys) + cols - 1) // cols))
    tile_px = int(atlas.get("tile_px") or batch.get("render", {}).get("tile_size_px") or 128)

    tiles = []
    for i, key in enumerate(variant_keys):
        col = i % cols
        row = i // cols
        tiles.append(
            {
                "variant_key": key,
                "png": png_paths[i] if i < len(png_paths) else None,
                "grid": [col, row],
                "uv": [
                    col / cols,
                    row / rows,
                    1.0 / cols,
                    1.0 / rows,
                ],
            }
        )

    body = {
        "schema_version": 1,
        "batch_id": batch.get("batch_id"),
        "tile_id": batch.get("tile_id"),
        "atlas_id": atlas.get("atlas_id"),
        "atlas_png": atlas_path,
        "columns": cols,
        "rows": rows,
        "tile_px": tile_px,
        "variant_count": len(variant_keys),
        "tiles": tiles,
    }
    meta_path.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return meta_path


def tile_atlas_pack(
    folder: str | Path,
    *,
    keyframe_rename: bool = False,
) -> dict:
    """Pack PNG stills in folder via legacy utils/tilemapgen."""
    folder = Path(folder).resolve()
    if not folder.is_dir():
        return {"ok": False, "error": f"Not a directory: {folder}"}

    crate = repo_root() / "utils" / "tilemapgen"
    if not (crate / "Cargo.toml").is_file():
        return {"ok": False, "error": f"tilemapgen crate missing: {crate}"}

    args = [
        "cargo",
        "run",
        "--quiet",
        "--manifest-path",
        str(crate / "Cargo.toml"),
        "--",
        str(folder),
    ]
    if keyframe_rename:
        args.append("-pk")

    proc = subprocess.run(
        args,
        cwd=str(crate),
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    atlases = sorted(folder.glob("tile_map_*.png"), key=lambda p: p.stat().st_mtime)
    return {
        "ok": proc.returncode == 0,
        "exit_code": proc.returncode,
        "log": ((proc.stdout or "") + (proc.stderr or "")).strip(),
        "atlas_path": str(atlases[-1]) if atlases else None,
        "folder": str(folder),
    }


def lod0_batch_run(batch_id: str, phase: str = "full") -> dict:
    """Run tools/mcp/scripts/kit_lod0_batch_runner.py."""
    allowed = {"g0g1", "geometry", "promote", "full", "all"}
    if phase not in allowed:
        return {"ok": False, "error": f"phase must be one of {sorted(allowed)}"}

    script = repo_root() / "tools" / "mcp" / "scripts" / "kit_lod0_batch_runner.py"
    if not script.is_file():
        return {"ok": False, "error": f"Missing script: {script}"}

    proc = subprocess.run(
        [sys.executable, str(script), "--batch", batch_id, "--phase", phase],
        cwd=str(repo_root()),
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return {
        "ok": proc.returncode == 0,
        "exit_code": proc.returncode,
        "batch_id": batch_id,
        "phase": phase,
        "log": ((proc.stdout or "") + (proc.stderr or "")).strip(),
    }


def assembly_build_run(snapshot_path: str | Path) -> dict[str, Any]:
    """Run assembly_build from an assembly snapshot JSON path."""
    from .material_authority import annotate_tile_bake_job, write_bake_job

    snap_path = Path(snapshot_path).resolve()
    snapshot = assembly.load_assembly_snapshot(snap_path)
    assembly_id = str(snapshot["assembly_id"])
    blend_rel = f"assets/staging/assemblies/{assembly_id}.blend"

    job_id = f"asm_{assembly_id}"
    job = annotate_tile_bake_job(
        {
            "schema_version": 1,
            "job_id": job_id,
            "operation": "assembly_build",
            "output": {"blend": blend_rel},
        },
        snapshot_path=snap_path,
        ensure_textures=True,
    )
    if job.get("_material_prep_failed"):
        prep = job["_material_prep_failed"]
        return {
            "ok": False,
            "job_id": job_id,
            "status": "missing_material_profiles",
            "materials": prep.get("materials"),
            "error": "material texture prep failed",
        }
    job_path = jobs_root() / f"{job_id}.json"
    write_bake_job(job_path, job)
    result = _run_tile_job_path(job_path)
    return {
        "ok": result.status == "done",
        "job_id": job_id,
        "status": result.status,
        "blend_path": blend_rel if result.status == "done" else None,
        "outputs": result.outputs,
        "error": result.error,
    }


def tile_batch_status(batch_id: str) -> dict[str, Any]:
    status_path = repo_root() / "assets" / "staging" / "tiles" / batch_id / "batch_status.json"
    if not status_path.is_file():
        return {"batch_id": batch_id, "status": "unknown"}
    return json.loads(status_path.read_text(encoding="utf-8"))


def _png_has_real_pixels(path: Path) -> bool:
    if not path.is_file():
        return False
    if path.stat().st_size < 200:
        return False
    try:
        from PIL import Image  # type: ignore[import-untyped]

        with Image.open(path) as im:
            return im.width >= 32 and im.height >= 32
    except Exception:
        return path.stat().st_size >= 1024


def _collect_variant_keys(batch: dict[str, Any]) -> list[str]:
    keys: list[str] = []
    for variant in batch.get("variants") or []:
        if isinstance(variant, dict):
            keys.append(_variant_key(variant))
    return keys


def _batch_assembly_blend(batch: dict[str, Any]) -> tuple[str | None, dict[str, Any] | None]:
    """Resolve assembly blend path for tile batches; error dict when build fails."""
    assembly_ref = batch.get("assembly_ref")
    if not assembly_ref:
        return None, None
    ref = dict(assembly_ref)
    style_pack = str(ref.get("style_pack_id") or "style_victorian")
    footprint = ref.get("footprint") or {}
    width = int(footprint.get("width") or 4)
    depth = int(footprint.get("depth") or 3)
    floors = int(footprint.get("floors") or 2)
    seed = int(batch.get("render", {}).get("seed") or 42)

    snap_path = ref.get("assembly_snapshot")
    if snap_path:
        snap_file = Path(str(snap_path))
        if not snap_file.is_absolute():
            snap_file = repo_root() / snap_file
    else:
        snap = assembly.generate_assembly_snapshot(
            style_pack_id=style_pack,
            width=width,
            depth=depth,
            floors=floors,
            seed=seed,
        )
        snap_file = repo_root() / str(snap["written_path"])

    blend_rel = (
        f"assets/staging/assemblies/{assembly.load_assembly_snapshot(snap_file)['assembly_id']}.blend"
    )
    blend_path = repo_root() / blend_rel
    snap_hash = hashlib.sha256(snap_file.read_bytes()).hexdigest()[:12]
    stale = not blend_path.is_file()
    if blend_path.is_file():
        hash_path = blend_path.with_suffix(".snap_hash")
        stale = not hash_path.is_file() or hash_path.read_text(encoding="utf-8").strip() != snap_hash

    if stale:
        build_result = assembly_build_run(snap_file)
        if not build_result.get("ok"):
            return None, {
                "ok": False,
                "status": "assembly_build_failed",
                "assembly": build_result,
            }
        blend_path.with_suffix(".snap_hash").write_text(snap_hash + "\n", encoding="utf-8")
    return blend_rel, None


def _tile_batch_keyframe_headless_export(
    batch: dict[str, Any],
    *,
    batch_id: str,
    staging: Path,
    only_keys: list[str] | None = None,
) -> dict[str, Any]:
    """Headless Light-rig stills → staging/{variant_key}.png (optional; civ-truck spine parity)."""
    assembly_blend, err = _batch_assembly_blend(batch)
    if err:
        return err
    if not assembly_blend:
        return {
            "ok": False,
            "status": "keyframe_headless_requires_assembly_ref",
            "hint": "keyframe_pack building batches need assembly_ref",
        }

    render = dict(batch.get("render") or {})
    render["method"] = "blender_keyframe_light_rig"
    light_rel = str(
        batch.get("light_blend")
        or os.environ.get("RUST_ENGINE_TILE_LIGHT_BLEND")
        or light_setup_blend_path()
    )
    if not Path(light_rel).is_absolute():
        light_rel = str((repo_root() / light_rel).resolve().relative_to(repo_root())).replace(
            "\\", "/"
        )

    bake_results: list[dict[str, Any]] = []
    png_paths: list[str] = []
    want = set(only_keys) if only_keys else None

    for variant in batch.get("variants") or []:
        vdict = dict(variant)
        vkey = _variant_key(vdict)
        if want is not None and vkey not in want:
            continue
        png_rel = str(staging.relative_to(repo_root()) / f"{vkey}.png").replace("\\", "/")
        job_id = f"tile_{batch_id}_{vkey}"[:120]
        snap_rel = None
        assembly_ref = batch.get("assembly_ref") or {}
        if assembly_ref.get("assembly_snapshot"):
            snap_rel = str(assembly_ref["assembly_snapshot"])
        job = {
            "schema_version": 1,
            "job_id": job_id,
            "operation": "tile_variant_bake",
            "mode": "assembly",
            "variant": {**vdict, "variant_key": vkey},
            "render": render,
            "light_blend": light_rel,
            "output": {"png": png_rel},
            "assembly_blend": assembly_blend,
        }
        from .material_authority import annotate_tile_bake_job, write_bake_job

        if snap_rel:
            job = annotate_tile_bake_job(job, snapshot_path=snap_rel, ensure_textures=True)
            if job.get("_material_prep_failed"):
                return {
                    "ok": False,
                    "status": "missing_material_profiles",
                    "materials": job["_material_prep_failed"].get("materials"),
                }
        job_path = jobs_root() / f"{job_id}.json"
        write_bake_job(job_path, job)
        result = _run_tile_job_path(job_path)
        bake_results.append(
            {
                "job_id": job_id,
                "variant_key": vkey,
                "status": result.status,
                "error": result.error,
            }
        )
        if result.status != "done":
            return {
                "ok": False,
                "status": "tile_keyframe_bake_failed",
                "failed_job": job_id,
                "bake_results": bake_results,
            }
        png_paths.extend(result.outputs)

    return {
        "ok": True,
        "status": "keyframe_headless_export_done",
        "bake_source": "keyframe_pack",
        "export_mode": "headless_light_rig",
        "variant_count": len(png_paths),
        "png_paths": png_paths,
        "bake_results": bake_results,
    }


def tile_keyframe_export(tile_batch_path: str | Path) -> dict[str, Any]:
    """Export all variants via headless keyframe rig (requires RUST_ENGINE_TILE_KEYFRAME_HEADLESS=1)."""
    if not tile_keyframe_headless_enabled():
        return {
            "ok": False,
            "status": "keyframe_headless_disabled",
            "hint": "Set RUST_ENGINE_TILE_KEYFRAME_HEADLESS=1 after manual keyframe G4 is green",
        }
    path = Path(tile_batch_path).resolve()
    batch = load_json_file(path)
    batch_id = str(batch["batch_id"])
    staging = repo_root() / "assets" / "staging" / "tiles" / batch_id
    staging.mkdir(parents=True, exist_ok=True)
    return _tile_batch_keyframe_headless_export(batch, batch_id=batch_id, staging=staging)


def _tile_batch_run_keyframe_pack(
    path: Path,
    batch: dict[str, Any],
    *,
    batch_id: str,
    staging: Path,
) -> dict[str, Any]:
    """Pack pre-baked keyframe PNGs — no headless tile_ortho_bake (DESIGN-TILE-SPINE-001)."""
    staging.mkdir(parents=True, exist_ok=True)
    variant_keys = _collect_variant_keys(batch)
    if len(variant_keys) < 2:
        return {
            "ok": False,
            "status": "missing_variants",
            "error": "keyframe_pack requires at least 2 variants in batch JSON",
        }

    pre_baked = batch.get("pre_baked_folder")
    if pre_baked:
        src = Path(str(pre_baked))
        if not src.is_absolute():
            src = repo_root() / src
        if not src.is_dir():
            return {"ok": False, "status": "pre_baked_missing", "error": str(src)}
        for vkey in variant_keys:
            candidates = list(src.glob(f"*{vkey}*.png")) + list(src.glob(f"{vkey}.png"))
            if not candidates:
                return {
                    "ok": False,
                    "status": "pre_baked_variant_missing",
                    "variant_key": vkey,
                    "folder": str(src),
                    "hint": "Export stills via utils/keyframe_render.py first",
                }
            dest = staging / f"{vkey}.png"
            dest.write_bytes(candidates[0].read_bytes())

    missing: list[str] = []
    png_paths: list[str] = []
    for vkey in variant_keys:
        png = staging / f"{vkey}.png"
        if not _png_has_real_pixels(png):
            missing.append(vkey)
            continue
        png_paths.append(str(png.resolve()))

    if (
        missing
        and tile_keyframe_headless_enabled()
        and not tile_dry_run_enabled()
    ):
        export = _tile_batch_keyframe_headless_export(
            batch, batch_id=batch_id, staging=staging, only_keys=missing
        )
        if not export.get("ok"):
            return export
        missing = [
            v
            for v in variant_keys
            if not _png_has_real_pixels(staging / f"{v}.png")
        ]
        png_paths = [
            str((staging / f"{vkey}.png").resolve())
            for vkey in variant_keys
            if _png_has_real_pixels(staging / f"{vkey}.png")
        ]

    ship = bool(batch.get("ship"))
    if missing and ship and not tile_dry_run_enabled():
        return {
            "ok": False,
            "status": "keyframe_pngs_missing",
            "missing_variant_keys": missing,
            "staging": str(staging),
            "hint": (
                "Ship batches use bake_source keyframe_pack: export PNGs with "
                "utils/Tile_iso_rig_v1.blend + utils/keyframe_render.py (manual), or "
                "RUST_ENGINE_TILE_KEYFRAME_HEADLESS=1 for optional headless export, then "
                "tile-batch-run / tile-atlas-pack. "
                "See docs/archive/2026-06-src-dev/plans/design_tile_bake_spine_convergence_v1.md"
            ),
        }

    if missing and tile_dry_run_enabled():
        for vkey in missing:
            _write_stub_png(staging / f"{vkey}.png")
            png_paths.append(str((staging / f"{vkey}.png").resolve()))

    pack = tile_atlas_pack(
        staging,
        keyframe_rename=bool(batch.get("keyframe_rename_pk", True)),
    )
    atlas_path = pack.get("atlas_path")
    if batch.get("atlas", {}).get("output_png") and atlas_path and pack.get("ok"):
        out_atlas = Path(str(batch["atlas"]["output_png"]))
        if not out_atlas.is_absolute():
            out_atlas = repo_root() / out_atlas
        out_atlas.parent.mkdir(parents=True, exist_ok=True)
        Path(atlas_path).replace(out_atlas)
        atlas_path = str(out_atlas)

    meta_path = _write_atlas_meta(batch, variant_keys, png_paths, atlas_path)
    status_body = {
        "batch_id": batch_id,
        "status": "done",
        "bake_source": "keyframe_pack",
        "variant_count": len(variant_keys),
        "png_paths": png_paths,
        "atlas_path": atlas_path,
        "meta_json": str(meta_path.relative_to(repo_root())).replace("\\", "/"),
        "dry_run": tile_dry_run_enabled(),
    }
    (staging / "batch_status.json").write_text(
        json.dumps(status_body, indent=2) + "\n", encoding="utf-8"
    )

    tile_index_result: dict[str, Any] | None = None
    if not tile_dry_run_enabled():
        try:
            if batch.get("atlas_domain") == "landscape":
                from .landscape_atlas_index import register_landscape_atlas_from_meta

                tile_index_result = register_landscape_atlas_from_meta(meta_path, batch=batch)
            else:
                tile_index_result = register_tile_atlas_from_meta(meta_path, batch=batch)
        except Exception as exc:  # noqa: BLE001
            tile_index_result = {"ok": False, "error": str(exc)}

    witness = write_tile_batch_witness(
        batch_id,
        batch=batch,
        png_count=len(png_paths),
        atlas_path=atlas_path,
        meta_path=str(meta_path),
        dry_run=tile_dry_run_enabled(),
        tile_index=tile_index_result,
    )
    return {
        "ok": True,
        "status": "done",
        "bake_source": "keyframe_pack",
        "batch_id": batch_id,
        "tile_batch_path": str(path),
        "variant_keys": variant_keys,
        "png_paths": png_paths,
        "atlas_path": atlas_path,
        "meta_json": str(meta_path),
        "atlas_pack": pack,
        "witness": witness,
        "dry_run": tile_dry_run_enabled(),
        "tile_atlas_index": tile_index_result,
    }


def tile_batch_run(tile_batch_path: str | Path) -> dict[str, Any]:
    """Tile pipeline — keyframe_pack (ship) or smoke_ortho_headless (CI only)."""
    path = Path(tile_batch_path).resolve()
    if not path.is_file():
        return {"ok": False, "error": f"Missing tile batch: {path}"}

    report = run_validator("tile_batch", str(path))
    if report.status == "failed":
        return {
            "ok": False,
            "status": "validation_failed",
            "validation": report.to_dict(),
        }

    batch = load_json_file(path)
    bake_source = str(batch.get("bake_source") or "smoke_ortho_headless")
    batch_id = str(batch["batch_id"])
    staging = repo_root() / "assets" / "staging" / "tiles" / batch_id
    if bake_source == "keyframe_pack":
        return _tile_batch_run_keyframe_pack(path, batch, batch_id=batch_id, staging=staging)

    matrix_ref = batch.get("matrix_ref")
    if matrix_ref and len(batch.get("variants") or []) < 2:
        from rust_engine_mcp.variant_matrix_expand import (
            expanded_variant_keys,
            load_variant_matrix,
            variant_row_for_key,
        )

        matrix = load_variant_matrix(str(matrix_ref))
        keys = expanded_variant_keys(
            matrix,
            include_fire_row=bool(batch.get("include_fire_row", True)),
            minimum_only=bool(batch.get("minimum_only", False)),
        )
        batch["variants"] = [variant_row_for_key(k) for k in keys]
    staging.mkdir(parents=True, exist_ok=True)

    assembly_blend: str | None = None
    assembly_ref = batch.get("assembly_ref")

    if assembly_ref:
        ref = dict(assembly_ref)
        style_pack = str(ref.get("style_pack_id") or "style_victorian")
        footprint = ref.get("footprint") or {}
        width = int(footprint.get("width") or 4)
        depth = int(footprint.get("depth") or 3)
        floors = int(footprint.get("floors") or 2)
        seed = int(batch.get("render", {}).get("seed") or 42)

        snap_path = ref.get("assembly_snapshot")
        if snap_path:
            snap_file = Path(str(snap_path))
            if not snap_file.is_absolute():
                snap_file = repo_root() / snap_file
        else:
            snap = assembly.generate_assembly_snapshot(
                style_pack_id=style_pack,
                width=width,
                depth=depth,
                floors=floors,
                seed=seed,
            )
            snap_file = repo_root() / str(snap["written_path"])

        blend_rel = f"assets/staging/assemblies/{assembly.load_assembly_snapshot(snap_file)['assembly_id']}.blend"
        blend_path = repo_root() / blend_rel
        snap_hash = hashlib.sha256(snap_file.read_bytes()).hexdigest()[:12]
        stale = not blend_path.is_file()
        if blend_path.is_file():
            hash_path = blend_path.with_suffix(".snap_hash")
            stale = not hash_path.is_file() or hash_path.read_text(encoding="utf-8").strip() != snap_hash

        if stale:
            build_result = assembly_build_run(snap_file)
            if not build_result.get("ok"):
                return {
                    "ok": False,
                    "status": "assembly_build_failed",
                    "assembly": build_result,
                }
            blend_path.with_suffix(".snap_hash").write_text(snap_hash + "\n", encoding="utf-8")
        assembly_blend = blend_rel

    render = dict(batch.get("render") or {})
    light_rel = str(
        batch.get("light_blend")
        or os.environ.get("RUST_ENGINE_TILE_LIGHT_BLEND")
        or light_setup_blend_path()
    )
    if not Path(light_rel).is_absolute():
        light_rel = str((repo_root() / light_rel).resolve().relative_to(repo_root())).replace("\\", "/")

    variant_keys: list[str] = []
    png_paths: list[str] = []
    bake_results: list[dict[str, Any]] = []

    for variant in batch.get("variants") or []:
        vdict = dict(variant)
        vkey = _variant_key(vdict)
        variant_keys.append(vkey)
        png_rel = str(staging.relative_to(repo_root()) / f"{vkey}.png").replace("\\", "/")
        job_id = f"tile_{batch_id}_{vkey}"[:120]
        snap_rel_batch = None
        if assembly_ref and assembly_ref.get("assembly_snapshot"):
            snap_rel_batch = str(assembly_ref["assembly_snapshot"])
        job = {
            "schema_version": 1,
            "job_id": job_id,
            "operation": "tile_variant_bake",
            "mode": "assembly" if assembly_blend else "terrain",
            "variant": {**vdict, "variant_key": vkey},
            "render": render,
            "light_blend": light_rel,
            "output": {"png": png_rel},
        }
        if assembly_blend:
            job["assembly_blend"] = assembly_blend
        else:
            job["terrain_base"] = str(batch.get("base") or "concrete")

        from .material_authority import annotate_tile_bake_job, write_bake_job

        if snap_rel_batch and assembly_blend:
            job = annotate_tile_bake_job(job, snapshot_path=snap_rel_batch, ensure_textures=True)
            if job.get("_material_prep_failed"):
                return {
                    "ok": False,
                    "status": "missing_material_profiles",
                    "materials": job["_material_prep_failed"].get("materials"),
                }
        job_path = jobs_root() / f"{job_id}.json"
        write_bake_job(job_path, job)
        result = _run_tile_job_path(job_path)
        bake_results.append(
            {
                "job_id": job_id,
                "variant_key": vkey,
                "status": result.status,
                "outputs": result.outputs,
                "error": result.error,
            }
        )
        if result.status != "done":
            return {
                "ok": False,
                "status": "tile_variant_bake_failed",
                "failed_job": job_id,
                "bake_results": bake_results,
            }
        png_paths.extend(result.outputs)

    pack = tile_atlas_pack(staging)
    atlas_path = pack.get("atlas_path")
    if batch.get("atlas", {}).get("output_png") and atlas_path and pack.get("ok"):
        out_atlas = Path(str(batch["atlas"]["output_png"]))
        if not out_atlas.is_absolute():
            out_atlas = repo_root() / out_atlas
        out_atlas.parent.mkdir(parents=True, exist_ok=True)
        Path(atlas_path).replace(out_atlas)
        atlas_path = str(out_atlas)

    meta_path = _write_atlas_meta(batch, variant_keys, png_paths, atlas_path)

    status_body = {
        "batch_id": batch_id,
        "status": "done",
        "bake_source": "smoke_ortho_headless",
        "variant_count": len(variant_keys),
        "png_paths": png_paths,
        "atlas_path": atlas_path,
        "meta_json": str(meta_path.relative_to(repo_root())).replace("\\", "/"),
        "dry_run": tile_dry_run_enabled(),
    }
    status_file = staging / "batch_status.json"
    status_file.write_text(json.dumps(status_body, indent=2) + "\n", encoding="utf-8")

    tile_index_result: dict[str, Any] | None = None
    if not tile_dry_run_enabled():
        try:
            tile_index_result = register_tile_atlas_from_meta(
                meta_path,
                batch=batch,
            )
        except Exception as exc:  # noqa: BLE001 — surface in result, do not fail bake
            tile_index_result = {"ok": False, "error": str(exc)}

    witness = write_tile_batch_witness(
        batch_id,
        batch=batch,
        png_count=len(png_paths),
        atlas_path=atlas_path,
        meta_path=str(meta_path),
        dry_run=tile_dry_run_enabled(),
        tile_index=tile_index_result,
    )

    return {
        "ok": True,
        "status": "done",
        "bake_source": "smoke_ortho_headless",
        "batch_id": batch_id,
        "tile_batch_path": str(path),
        "variant_keys": variant_keys,
        "png_paths": png_paths,
        "atlas_path": atlas_path,
        "meta_json": str(meta_path),
        "atlas_pack": pack,
        "witness": witness,
        "dry_run": tile_dry_run_enabled(),
        "tile_atlas_index": tile_index_result,
    }


def tile_batch_run_not_implemented(tile_batch_path: str = "") -> dict:
    """Deprecated alias — redirects to real pipeline."""
    if not tile_batch_path:
        return {
            "ok": False,
            "error": "tile_batch_path required",
            "hint": "python -m rust_engine_mcp.cli tile-batch-run <tile_batch_v1.json>",
        }
    return tile_batch_run(tile_batch_path)
