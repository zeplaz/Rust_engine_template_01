"""VEG-F02-BURN-ATLAS-001 — burn atlas lane witness (catalog rows + expanded atlas)."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.veg_catalog_loader import refresh_veg_catalog_burn_rows_witness

WITNESS_REL = "debug_runs/veg_f02_burn_atlas_live.json"
EXPANDED_WITNESS = "debug_runs/art_pipeline/tile_landscape_expanded_live.json"
SIGN_ATLAS = "debug_runs/mcp_landscape_sign_atlas_live.json"


def refresh_veg_f02_burn_atlas_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    burn = refresh_veg_catalog_burn_rows_witness(repo=root)
    expanded = _load_json(root / EXPANDED_WITNESS)
    sign = _load_json(root / SIGN_ATLAS)
    burn_rows = int(burn.get("burn_rows") or 0)
    green = (
        bool(burn.get("green"))
        and burn_rows >= 1
        and bool(expanded.get("green"))
        and bool(sign.get("green"))
    )
    body: dict[str, Any] = {
        "gate": "VEG-F02-BURN-ATLAS-001",
        "green": green,
        "burn_catalog_rows": burn_rows,
        "expanded_atlas_green": bool(expanded.get("green")),
        "sign_atlas_green": bool(sign.get("green")),
        "ship": False,
        "_agent_meta": {
            "schema": "veg_f02_burn_atlas_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "VEG_F02_BURN_ATLAS",
            "relative_path": WITNESS_REL,
        },
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body


def _load_json(path: Path) -> dict[str, Any]:
    if not path.is_file():
        return {"green": False, "missing": str(path)}
    try:
        body = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {"green": False, "missing": str(path)}
    return body if isinstance(body, dict) else {"green": False}
