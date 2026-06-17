"""MCP landscape sign + LG-5 atlas — combined witness rollup."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.landscape_grammar_presets import (
    BATCH_WITNESS_REL,
    SIGN_WITNESS_REL,
    landscape_grammar_presets_batch,
)
from rust_engine_mcp.landscape_lg5_batch import run_landscape_lg5_atlas_batch
from rust_engine_mcp.paths import repo_root

ROLLUP_WITNESS_REL = "debug_runs/mcp_landscape_sign_atlas_live.json"
LG5_WITNESS_REL = "debug_runs/landscape_grammar_lg5_live.json"
LG5_TILE_WITNESS_REL = "debug_runs/art_pipeline/tile_tile_landscape_lg5_pilot_v1_live.json"
PLAN_REF = "src/dev/plan_landscape_grammar_mcp_sign_delegate_v1.md"
CHARTER_REF = "src/dev/design_landscape_lg5_atlas_v1.md"


def _load_witness(rel: str) -> dict[str, Any]:
    path = repo_root() / rel
    if not path.is_file():
        return {"ok": False, "missing": rel}
    try:
        body = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {"ok": False, "missing": rel}
    return body if isinstance(body, dict) else {"ok": False, "missing": rel}


def landscape_sign_atlas_status(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    sign = _load_witness(SIGN_WITNESS_REL)
    preset_batch = _load_witness(BATCH_WITNESS_REL)
    lg5 = _load_witness(LG5_WITNESS_REL)
    tile = _load_witness(LG5_TILE_WITNESS_REL)
    presets = landscape_grammar_presets_batch(repo=root)
    return {
        "sign": {
            "gate": "MCP-LANDSCAPE-GRAMMAR-SIGN-001",
            "green": bool(sign.get("signed")) and bool(sign.get("green")),
            "witness": SIGN_WITNESS_REL,
            "topology_preset_count": sign.get("topology_preset_count"),
            "ship_preset_count": sign.get("ship_preset_count"),
        },
        "preset_batch": {
            "gate": "MCP-LG-VALID-PRESET-001",
            "green": bool(preset_batch.get("green")) and bool(presets.get("green")),
            "witness": BATCH_WITNESS_REL,
            "passed": (presets.get("preset_validation") or {}).get("passed"),
            "total": (presets.get("preset_validation") or {}).get("total"),
        },
        "atlas": {
            "gate": "VEG-F02-MCP-ATLAS-001",
            "charter": CHARTER_REF,
            "green": bool(lg5.get("green")) and bool(tile.get("green")),
            "lg5_witness": LG5_WITNESS_REL,
            "tile_witness": LG5_TILE_WITNESS_REL,
            "atlas_id": lg5.get("atlas_id") or "landscape_lg5_pilot_v1",
            "registry_stamp": lg5.get("registry_stamp"),
            "landscape_index": lg5.get("landscape_index"),
        },
    }


def refresh_mcp_landscape_sign_atlas_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    status = landscape_sign_atlas_status(repo=root)
    green = all(
        block.get("green")
        for block in (status.get("sign"), status.get("preset_batch"), status.get("atlas"))
        if isinstance(block, dict)
    )
    body: dict[str, Any] = {
        "gate": "MCP-LANDSCAPE-SIGN-ATLAS-001",
        "green": green,
        "plan_ref": PLAN_REF,
        "charter_ref": CHARTER_REF,
        "lanes": status,
        "verify": [
            "python -m rust_engine_mcp.cli landscape-grammar-presets-witness",
            "python -m rust_engine_mcp.cli landscape-sign-atlas-witness",
        ],
        "_agent_meta": {
            "schema": "mcp_landscape_sign_atlas_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "MCP_LANDSCAPE_SIGN_ATLAS",
            "source_system": "landscape_sign_atlas",
            "relative_path": ROLLUP_WITNESS_REL,
        },
    }
    out = root / ROLLUP_WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    body["written"] = ROLLUP_WITNESS_REL
    return body


def run_landscape_sign_atlas_refresh(*, refresh_atlas: bool = False) -> dict[str, Any]:
    if refresh_atlas:
        run_landscape_lg5_atlas_batch(refresh_keyframes=False)
    from rust_engine_mcp import landscape_grammar_presets

    landscape_grammar_presets.write_landscape_grammar_presets_witness()
    landscape_grammar_presets.refresh_mcp_landscape_grammar_sign_witness()
    return refresh_mcp_landscape_sign_atlas_witness()
