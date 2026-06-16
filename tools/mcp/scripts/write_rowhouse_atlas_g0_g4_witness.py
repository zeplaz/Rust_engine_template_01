#!/usr/bin/env python3
"""Write MCP-PROD-ATLAS-G0-G4 witness for rowhouse production atlas."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / "debug_runs" / "art_pipeline" / "rowhouse_production_atlas_g0_g4_live.json"
YAML_OUT = ROOT / "debug_runs" / "art_pipeline" / "rowhouse_production_atlas_g0_g4_witness.yaml"
BATCH_JSON = ROOT / "tools/mcp/schemas/examples/tile_batch_rowhouse_victorian_production_v1.json"
TILE_WITNESS = ROOT / "debug_runs/art_pipeline/tile_tile_rowhouse_victorian_production_v1_live.json"
G4_WITNESS = ROOT / "debug_runs/art_pipeline/rowhouse_production_keyframe_g4_live.json"
SIGNOFF = ROOT / "debug_runs/art_pipeline/rowhouse_victorian_production_signoff.yaml"
ATLAS_PNG = ROOT / "assets/textures/buildings_iso/production/rowhouse_victorian_production_v1_atlas.png"
META_JSON = ROOT / "assets/staging/tiles/tile_rowhouse_victorian_production_v1/atlas_meta.json"
KEYFRAME_DIR = ROOT / "assets/staging/tiles/keyframe_stills/rowhouse_victorian"
STAGING_DIR = ROOT / "assets/staging/tiles/tile_rowhouse_victorian_production_v1"


def _load_json(path: Path) -> dict:
    if not path.is_file():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def _png_count(folder: Path) -> int:
    if not folder.is_dir():
        return 0
    return len([p for p in folder.glob("*.png") if not p.name.startswith("tile_map_")])


def _signoff_proceed_ship() -> bool:
    if not SIGNOFF.is_file():
        return False
    for line in SIGNOFF.read_text(encoding="utf-8").splitlines():
        if line.startswith("proceed_ship:"):
            return line.split(":", 1)[1].strip().lower() in ("yes", "true")
    return False


def _yaml(*, green: bool, gates: dict[str, bool], proceed_ship: bool) -> str:
    lines = [
        "# rowhouse_production_atlas_g0_g4_witness.yaml — MCP-PROD-ATLAS-G0-G4",
        "task_id: MCP-PROD-ATLAS-G0-G4",
        "agent: designer-mcp",
        "batch_id: tile_rowhouse_victorian_production_v1",
        "atlas_id: rowhouse_victorian_production_v1",
        f"green: {'true' if green else 'false'}",
        f"proceed_ship: {'yes' if proceed_ship else 'no'}",
        "",
        "gates:",
    ]
    for gate, ok in gates.items():
        lines.append(f"  {gate}: {'pass' if ok else 'fail'}")
    lines.extend(
        [
            "",
            "artifacts:",
            f"  tile_batch: {BATCH_JSON.relative_to(ROOT).as_posix()}",
            f"  g4_signoff: {SIGNOFF.relative_to(ROOT).as_posix()}",
            f"  atlas_png: {ATLAS_PNG.relative_to(ROOT).as_posix()}",
            f"  atlas_meta: {META_JSON.relative_to(ROOT).as_posix()}",
            f"  keyframe_stills: {KEYFRAME_DIR.relative_to(ROOT).as_posix()}/",
            "",
            "unblocks:",
            "  - MCP-PROD-TILE-VAL",
            "  - MCP-PROD-INDEX",
        ]
    )
    return "\n".join(lines) + "\n"


def main() -> None:
    tile = _load_json(TILE_WITNESS)
    g4 = _load_json(G4_WITNESS)
    tile_gates = tile.get("gates") or {}
    g4_gates = g4.get("gates") or {}
    proceed_ship = _signoff_proceed_ship() and g4_gates.get("g4_8_proceed_ship") == "pass"
    png_staging = _png_count(STAGING_DIR)
    png_keyframe = _png_count(KEYFRAME_DIR)
    gates = {
        "G0": tile_gates.get("G0") == "pass",
        "G1": tile_gates.get("G1") == "pass",
        "G2": tile_gates.get("G2") == "pass",
        "G3": tile_gates.get("G3") == "pass",
        "G4": g4.get("green") is True and g4_gates.get("g4_8_proceed_ship") == "pass",
    }
    green = (
        all(gates.values())
        and ATLAS_PNG.is_file()
        and META_JSON.is_file()
        and png_staging >= 14
        and png_keyframe >= 14
        and proceed_ship
    )
    payload = {
        "task_id": "MCP-PROD-ATLAS-G0-G4",
        "agent": "designer-mcp",
        "batch_id": "tile_rowhouse_victorian_production_v1",
        "ok": green,
        "green": green,
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "gates": gates,
        "proceed_ship": proceed_ship,
        "png_count_staging": png_staging,
        "png_count_keyframe": png_keyframe,
        "atlas_png": str(ATLAS_PNG.relative_to(ROOT)).replace("\\", "/"),
        "atlas_meta": str(META_JSON.relative_to(ROOT)).replace("\\", "/"),
        "g4_witness": str(G4_WITNESS.relative_to(ROOT)).replace("\\", "/"),
        "tile_witness": str(TILE_WITNESS.relative_to(ROOT)).replace("\\", "/"),
        "signoff": str(SIGNOFF.relative_to(ROOT)).replace("\\", "/"),
        "ship_allowed_index": (tile.get("tile_atlas_index") or {}).get("entry", {}).get("ship_allowed"),
        "unblocks": ["MCP-PROD-TILE-VAL", "MCP-PROD-INDEX"],
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    YAML_OUT.write_text(_yaml(green=green, gates=gates, proceed_ship=proceed_ship), encoding="utf-8")
    print(f"Wrote {OUT}")
    print(f"Wrote {YAML_OUT}")
    if not green:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
