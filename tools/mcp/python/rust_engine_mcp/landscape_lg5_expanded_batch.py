"""APS-EVO-E4-ATLAS-EXPAND-001 — expanded LG-5 keyframes + tile batch (16 cells)."""

from __future__ import annotations

import hashlib
import json
import time
from pathlib import Path
from typing import Any

from PIL import Image, ImageDraw

from rust_engine_mcp.landscape_lg5_batch import (
    _draw_corridor,
    _draw_patch,
    _draw_ring,
    SEED,
    SIZE,
)
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.schemas import load_json_file
from rust_engine_mcp.tile_pipeline import tile_batch_run

BATCH_JSON = repo_root() / "assets/staging/specs/tile_batch_landscape_expanded_v1.json"
KEYFRAME_REL = "assets/staging/tiles/keyframe_stills/tile_landscape_expanded_v1"
EXPANDED_WITNESS_REL = "debug_runs/art_pipeline/tile_landscape_expanded_live.json"
BATCH_WITNESS_REL = "debug_runs/art_pipeline/tile_tile_landscape_expanded_v1_live.json"


def _rng(seed: int, tag: str) -> int:
    h = hashlib.sha256(f"{seed}:{tag}".encode()).hexdigest()
    return int(h[:8], 16)


def _topology_drawer(variant_key: str):
    if "corridor" in variant_key:
        return _draw_corridor
    if "ring" in variant_key:
        return _draw_ring
    if "cluster" in variant_key:
        return _draw_cluster
    if "fringe" in variant_key:
        return _draw_fringe
    return _draw_patch


def _draw_cluster(im: Image.Image, draw: ImageDraw.ImageDraw) -> None:
    draw.rectangle([0, 0, SIZE, SIZE], fill=(44, 78, 42, 255))
    cx, cy = SIZE // 2, SIZE // 2
    for i in range(5):
        ox = (_rng(SEED, f"cl{i}") % 15) - 7
        oy = (_rng(SEED, f"clY{i}") % 13) - 6
        draw.ellipse([cx + ox - 5, cy + oy - 4, cx + ox + 5, cy + oy + 4], fill=(72, 130, 58, 230))


def _draw_fringe(im: Image.Image, draw: ImageDraw.ImageDraw) -> None:
    draw.rectangle([0, 0, SIZE, SIZE], fill=(52, 68, 40, 255))
    for x in range(4, SIZE - 4, 8):
        draw.rectangle([x, SIZE - 14, x + 4, SIZE - 4], fill=(96, 118, 62, 255))


def _apply_state_overlay(im: Image.Image, variant_key: str) -> None:
    overlay = Image.new("RGBA", im.size, (0, 0, 0, 0))
    draw = ImageDraw.Draw(overlay)
    if "_scar" in variant_key:
        draw.rectangle([0, 0, SIZE, SIZE], fill=(80, 70, 60, 90))
    elif "_burn_" in variant_key:
        draw.rectangle([0, 0, SIZE, SIZE], fill=(200, 60, 20, 110))
        draw.ellipse([8, 8, SIZE - 8, SIZE - 8], outline=(255, 140, 40, 200), width=2)
    elif "_regrowth_grass" in variant_key:
        draw.rectangle([0, SIZE // 2, SIZE, SIZE], fill=(60, 150, 70, 80))
    elif "_regrowth_shrub" in variant_key:
        draw.rectangle([0, SIZE // 3, SIZE, SIZE], fill=(40, 110, 50, 100))
    im.alpha_composite(overlay)


def write_landscape_expanded_keyframes(out_dir: Path | None = None) -> list[Path]:
    folder = out_dir or (repo_root() / KEYFRAME_REL)
    folder.mkdir(parents=True, exist_ok=True)
    batch = load_json_file(BATCH_JSON)
    variant_keys = [str(v.get("variant_key")) for v in batch.get("variants") or [] if v.get("variant_key")]
    written: list[Path] = []
    for key in variant_keys:
        im = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
        draw = ImageDraw.Draw(im)
        _topology_drawer(key)(im, draw)
        _apply_state_overlay(im, key)
        path = folder / f"{key}.png"
        im.save(path, format="PNG")
        written.append(path)
    return written


def _rollup_expanded_witness(batch_witness: dict[str, Any], *, batch: dict[str, Any]) -> dict[str, Any]:
    return {
        "gate": "APS-EVO-E4-ATLAS-EXPAND-001",
        "green": bool(batch_witness.get("green")),
        "batch_id": batch.get("batch_id"),
        "atlas_domain": batch.get("atlas_domain"),
        "bake_source": batch_witness.get("batch_status", {}).get("bake_source") or batch.get("bake_source"),
        "png_count": int(batch_witness.get("png_count") or 0),
        "variant_count": len(batch.get("variants") or []),
        "atlas_id": (batch.get("atlas") or {}).get("atlas_id"),
        "tile_batch_path": str(BATCH_JSON.relative_to(repo_root())).replace("\\", "/"),
        "batch_witness": BATCH_WITNESS_REL,
        "charter": "src/dev/design_landscape_lg5_expansion_matrix_v1.md",
        "_agent_meta": {
            "schema": "tile_landscape_expanded_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "APS_E4_ATLAS_EXPAND",
            "source_system": "landscape_lg5_expanded_batch",
            "relative_path": EXPANDED_WITNESS_REL,
            "ritual": "BLANG:WIT-HON APS-EVO-E4-ATLAS-EXPAND-001" if batch_witness.get("green") else None,
            "agent": "coder-mcp",
        },
    }


def refresh_tile_landscape_expanded_witness(*, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    root = repo or repo_root()
    result = run_landscape_expanded_atlas_batch(refresh_keyframes=True, repo=root)
    batch_witness_path = root / BATCH_WITNESS_REL
    batch_witness: dict[str, Any] = {}
    if batch_witness_path.is_file():
        batch_witness = json.loads(batch_witness_path.read_text(encoding="utf-8"))
    batch = load_json_file(BATCH_JSON)
    body = _rollup_expanded_witness(batch_witness, batch=batch)
    body["run_ok"] = bool(result.get("ok"))
    body["keyframe_count"] = len(list((root / KEYFRAME_REL).glob("*.png")))
    green = (
        body.get("green")
        and body.get("png_count", 0) >= 16
        and body.get("keyframe_count", 0) >= 16
        and body.get("atlas_domain") == "landscape"
        and body.get("bake_source") == "keyframe_pack"
        and batch_witness.get("ship") is False
    )
    body["green"] = green
    body["ship_honest"] = batch_witness.get("ship") is False
    return write_aps_live_witness(
        body,
        EXPANDED_WITNESS_REL,
        schema="tile_landscape_expanded_live_v1",
        profile="APS_E4_ATLAS_EXPAND",
        source_system="landscape_lg5_expanded_batch",
        ritual="BLANG:WIT-HON APS-EVO-E4-ATLAS-EXPAND-001" if green else None,
        exit_predicate_must=[
            {"path": "png_count", "eq": 16},
            {"path": "keyframe_count", "eq": 16},
            {"path": "atlas_domain", "eq": "landscape"},
            {"path": "bake_source", "eq": "keyframe_pack"},
            {"path": "ship_honest", "eq": True},
        ],
    )


def run_landscape_expanded_atlas_batch(
    *,
    refresh_keyframes: bool = True,
    repo: Path | None = None,
) -> dict[str, Any]:
    root = repo or repo_root()
    batch_path = root / "assets/staging/specs/tile_batch_landscape_expanded_v1.json"
    if refresh_keyframes:
        write_landscape_expanded_keyframes(root / KEYFRAME_REL)
    if not batch_path.is_file():
        return {"ok": False, "error": f"missing batch json: {batch_path}"}
    result = tile_batch_run(batch_path)
    witness_path = root / BATCH_WITNESS_REL
    result["art_pipeline_witness"] = str(witness_path) if witness_path.is_file() else None
    result["witness_green"] = False
    if witness_path.is_file():
        body = json.loads(witness_path.read_text(encoding="utf-8"))
        result["witness_green"] = bool(body.get("green"))
        batch = load_json_file(batch_path)
        rollup = _rollup_expanded_witness(body, batch=batch)
        rollup["green"] = (
            rollup.get("green")
            and rollup.get("png_count", 0) > 3
            and rollup.get("atlas_domain") == "landscape"
            and rollup.get("bake_source") == "keyframe_pack"
        )
        rollup_out = root / EXPANDED_WITNESS_REL
        rollup_out.write_text(json.dumps(rollup, indent=2) + "\n", encoding="utf-8")
        result["expanded_witness"] = str(rollup_out)
    return result


def main() -> int:
    body = refresh_tile_landscape_expanded_witness()
    print(body)
    return 0 if body.get("green") else 1


if __name__ == "__main__":
    raise SystemExit(main())
