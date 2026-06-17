"""MCP-APS-ATLAS-LAND-REGISTER-001 — landscape atlas index register witness + ensure."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness
from rust_engine_mcp.landscape_atlas_index import (
    LANDSCAPE_ATLAS_INDEX_JSON,
    landscape_atlas_registered,
    landscape_expanded_atlas_registered,
    landscape_lg5_registry_stamped,
    load_landscape_atlas_index,
    register_landscape_atlas_from_meta,
)
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.schemas import load_json_file

APS_ATLAS_LAND_REGISTER_WITNESS = "debug_runs/aps_atlas_land_register_live.json"
EXPANDED_BATCH_WITNESS = "debug_runs/art_pipeline/tile_tile_landscape_expanded_v1_live.json"
EXPANDED_META_REL = "assets/staging/tiles/tile_landscape_expanded_v1/atlas_meta.json"
EXPANDED_BATCH_REL = "assets/staging/specs/tile_batch_landscape_expanded_v1.json"
PILOT_ATLAS_ID = "landscape_lg5_pilot_v1"
EXPANDED_ATLAS_ID = "landscape_lg5_expanded_v1"


def _ensure_aps_suite_path() -> None:
    suite_root = repo_root() / "tools/mcp"
    if str(suite_root) not in sys.path:
        sys.path.insert(0, str(suite_root))


def _verify_atlas_panel_wired() -> bool:
    _ensure_aps_suite_path()
    try:
        from art_pipeline_suite.atlas_panel import AtlasPanel
    except ImportError:
        return False
    return hasattr(AtlasPanel, "refresh_landscape_register") and hasattr(AtlasPanel, "set_domain")


def ensure_expanded_landscape_registered(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    if landscape_expanded_atlas_registered():
        return {"ok": True, "already_registered": True, "atlas_id": EXPANDED_ATLAS_ID}
    meta_path = root / EXPANDED_META_REL
    if not meta_path.is_file():
        return {"ok": False, "error": f"missing meta: {EXPANDED_META_REL}"}
    batch_path = root / EXPANDED_BATCH_REL
    batch = load_json_file(batch_path) if batch_path.is_file() else None
    try:
        result = register_landscape_atlas_from_meta(meta_path, batch=batch)
    except (OSError, ValueError) as exc:
        return {"ok": False, "error": str(exc)}
    return {"ok": True, "registered": True, **result}


def check_atlas_land_register(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    ensure = ensure_expanded_landscape_registered(repo=root)
    entries = load_landscape_atlas_index()
    atlas_ids = sorted({str(e.get("atlas_id") or "") for e in entries if e.get("atlas_id")})
    batch = load_json_file(root / EXPANDED_BATCH_REL) if (root / EXPANDED_BATCH_REL).is_file() else {}
    batch_witness: dict[str, Any] = {}
    witness_path = root / EXPANDED_BATCH_WITNESS
    if witness_path.is_file():
        try:
            batch_witness = json.loads(witness_path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            batch_witness = {}
    panel_wired = _verify_atlas_panel_wired()
    pilot_registered = landscape_lg5_registry_stamped()
    expanded_registered = landscape_expanded_atlas_registered()
    atlas_domain_landscape_ok = str(batch.get("atlas_domain") or "") == "landscape"
    batch_witness_green = bool(batch_witness.get("green"))
    register_green = bool(
        panel_wired
        and pilot_registered
        and expanded_registered
        and atlas_domain_landscape_ok
        and batch_witness_green
    )
    return {
        "slice_id": "MCP-APS-ATLAS-LAND-REGISTER-001",
        "register_target": "_landscape_atlas_index",
        "index_path": LANDSCAPE_ATLAS_INDEX_JSON,
        "atlas_ids": atlas_ids,
        "pilot_atlas_id": PILOT_ATLAS_ID,
        "expanded_atlas_id": EXPANDED_ATLAS_ID,
        "pilot_registered": pilot_registered,
        "expanded_registered": expanded_registered,
        "atlas_domain_landscape_ok": atlas_domain_landscape_ok,
        "batch_witness_green": batch_witness_green,
        "batch_witness": EXPANDED_BATCH_WITNESS,
        "panel_wired": panel_wired,
        "ensure_expanded": ensure,
        "register_green": register_green,
        "expanded_batch_path": EXPANDED_BATCH_REL,
        "design_ref": "src/dev/design_landscape_lg5_expansion_matrix_v1.md",
    }


def refresh_aps_atlas_land_register_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    check = check_atlas_land_register(repo=root)
    green = bool(check.get("register_green"))
    body: dict[str, Any] = {
        "gate": "MCP-APS-ATLAS-LAND-REGISTER-001",
        "program_id": "APS-E4",
        "green": green,
        **check,
    }
    return write_aps_live_witness(
        body,
        APS_ATLAS_LAND_REGISTER_WITNESS,
        schema="aps_atlas_land_register_live_v1",
        profile="APS_ATLAS_LAND_REGISTER",
        source_system="aps_atlas_land_register",
        ritual="BLANG:WIT-HON MCP-APS-ATLAS-LAND-REGISTER-001" if green else None,
        exit_predicate_must=[
            {"path": "register_green", "eq": True},
            {"path": "pilot_registered", "eq": True},
            {"path": "expanded_registered", "eq": True},
            {"path": "panel_wired", "eq": True},
            {"path": "atlas_domain_landscape_ok", "eq": True},
        ],
        repo=root,
    )
