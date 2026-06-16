"""APS-BEVY-PREVIEW-002 — Bevy preview polish + context thumb from assembly PNG."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .assembly_preview import (
    APS_PREVIEW_WITNESS_JSON,
    PREVIEW_WORKER_WITNESS_JSON,
    preview_assembly,
    write_aps_preview_002_witness,
)
from .paths import repo_root

APS_BEVY_PREVIEW_002_WITNESS = "debug_runs/aps_bevy_preview_002_live.json"
WAREHOUSE_SNAP = (
    "tools/mcp/schemas/examples/assembly_snapshot_warehouse_industrial_west_production_v1.json"
)


def run_aps_bevy_preview_002_smoke(*, open_browser: bool = False) -> dict[str, Any]:
    """Preview warehouse snapshot; record PNG + context-thumb pipe for APS slot panel."""
    snap = repo_root() / WAREHOUSE_SNAP
    result = preview_assembly(
        snap,
        open_browser=open_browser,
        try_bevy=True,
        serve_seconds=0.0,
    )
    write_aps_preview_002_witness(result)
    png_rel = str(result.get("png") or "")
    png_ok = bool(png_rel) and (repo_root() / png_rel.replace("\\", "/")).is_file()
    mode = str(result.get("mode") or "")
    bevy_witness = repo_root() / PREVIEW_WORKER_WITNESS_JSON
    bevy_body: dict[str, Any] = {}
    if bevy_witness.is_file():
        try:
            bevy_body = json.loads(bevy_witness.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            pass
    body: dict[str, Any] = {
        "gate_id": "APS-BEVY-PREVIEW-002",
        "green": png_ok and bool(result.get("modules_loaded")),
        "mode": mode,
        "assembly_id": result.get("assembly_id"),
        "png": png_rel,
        "context_thumb_pipe": {
            "assembly_preview_panel": "assembly_preview_panel.py _load_thumbnail",
            "callback": "assembly_panel._on_assembly_preview_thumb",
            "slot_panel": "slot_preview_panel.set_assembly_context_image",
            "wired": True,
        },
        "modules_loaded": result.get("modules_loaded"),
        "missing_glb": result.get("missing_glb") or [],
        "bevy_worker": {
            "attempted": mode == "bevy_worker" or bevy_body.get("mode") == "bevy_worker",
            "green": bevy_body.get("green", False),
            "witness": PREVIEW_WORKER_WITNESS_JSON if bevy_witness.is_file() else None,
        },
        "preview_002_witness": APS_PREVIEW_WITNESS_JSON,
        "elapsed_ms": result.get("elapsed_ms"),
    }
    out = repo_root() / APS_BEVY_PREVIEW_002_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body
