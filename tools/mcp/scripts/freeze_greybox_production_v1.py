#!/usr/bin/env python3
"""TILE-FIX-001 — move greybox production v1 atlases + staging to archive (no deletes)."""

from __future__ import annotations

import json
import shutil
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
ARCHIVE = REPO / "assets" / "archive" / "greybox_tile_production_v1_frozen_2026-06"
ATLASES = ARCHIVE / "atlases"
STAGING = ARCHIVE / "staging"
PROD_TEX = REPO / "assets" / "textures" / "buildings_iso" / "production"
STAGING_TILES = REPO / "assets" / "staging" / "tiles"
KEYFRAME_STILLS = STAGING_TILES / "keyframe_stills"

PRODUCTION_BATCH_DIRS = [
    "tile_bunker_military_production_v1",
    "tile_missile_silo_military_production_v1",
    "tile_rowhouse_victorian_production_v1",
    "tile_shopfront_colonial_production_v1",
    "tile_warehouse_industrial_west_production_v1",
]

PRODUCTION_ATLAS_NAMES = [
    "bunker_military_production_v1_atlas.png",
    "missile_silo_military_production_v1_atlas.png",
    "rowhouse_victorian_production_v1_atlas.png",
    "shopfront_colonial_production_v1_atlas.png",
    "warehouse_industrial_west_production_v1_atlas.png",
]


def _move(src: Path, dst: Path, log: list[dict]) -> None:
    if not src.exists():
        return
    dst.parent.mkdir(parents=True, exist_ok=True)
    if dst.exists():
        log.append({"action": "skip_exists", "src": str(src), "dst": str(dst)})
        return
    shutil.move(str(src), str(dst))
    log.append({"action": "moved", "src": str(src), "dst": str(dst)})


def main() -> int:
    log: list[dict] = []
    ATLASES.mkdir(parents=True, exist_ok=True)
    STAGING.mkdir(parents=True, exist_ok=True)

    for name in PRODUCTION_ATLAS_NAMES:
        _move(PROD_TEX / name, ATLASES / name, log)

    for batch_dir in PRODUCTION_BATCH_DIRS:
        src = STAGING_TILES / batch_dir
        dst = STAGING / batch_dir
        if src.is_dir():
            _move(src, dst, log)
            # rewrite atlas_png in archived meta if present
            meta = dst / "atlas_meta.json"
            if meta.is_file():
                data = json.loads(meta.read_text(encoding="utf-8"))
                aid = data.get("atlas_id", "")
                if aid:
                    rel = f"assets/archive/greybox_tile_production_v1_frozen_2026-06/atlases/{aid}_atlas.png"
                    data["atlas_png"] = rel
                    data["frozen"] = True
                    data["freeze_id"] = "TILE-FIX-001"
                    meta.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")

    if KEYFRAME_STILLS.is_dir():
        for child in KEYFRAME_STILLS.iterdir():
            if child.is_dir():
                _move(child, ARCHIVE / "keyframe_stills" / child.name, log)

    # remove empty production / keyframe dirs if safe
    for path in (PROD_TEX, KEYFRAME_STILLS):
        if path.is_dir() and not any(path.iterdir()):
            path.rmdir()
            log.append({"action": "rmdir_empty", "path": str(path)})

    manifest = {
        "freeze_id": "TILE-FIX-001",
        "moved_at": datetime.now(timezone.utc).isoformat(),
        "policy": "docs/archive/2026-06-src-dev/plans/tile_greybox_production_frozen_v1.md",
        "moves": log,
    }
    ARCHIVE.mkdir(parents=True, exist_ok=True)
    (ARCHIVE / "MOVED_LOG.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(f"TILE-FIX-001 freeze: {len(log)} operations -> {ARCHIVE}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
