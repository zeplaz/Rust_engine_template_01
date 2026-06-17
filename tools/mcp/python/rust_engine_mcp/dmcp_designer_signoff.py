"""Designer-mcp sign-off witnesses — DMCP-E4 matrix charter + DMCP-E0 artist re-verdict."""

from __future__ import annotations

import json
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

E4_CHARTER_REL = "src/dev/design_landscape_lg5_expansion_matrix_v1.md"
E4_WITNESS_REL = "debug_runs/art_pipeline/dmcp_e4_matrix_charter_live.json"
E0_REVIEW_REL = "src/dev/design_aps_artist_ship_review_20260616_v1.md"
E0_WITNESS_REL = "debug_runs/art_pipeline/dmcp_e0_artist_reverdict_live.json"
E0_E2E_REL = "debug_runs/aps_artist_tool_e2e_live.json"
TILE_BATCH_REL = "assets/staging/specs/tile_batch_landscape_expanded_v1.json"
G0_RULES_REL = "debug_runs/art_pipeline/landscape_expanded_g0_rules.yaml"
CATALOG_REL = "assets/configs/landscape/_vegetation_variant_catalog.ron"
EXPECTED_MATRIX_CELLS = 16


def _load_json(rel: str, *, repo: Path) -> dict[str, Any]:
    return json.loads((repo / rel).read_text(encoding="utf-8"))


def verify_e4_matrix_charter(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    charter = root / E4_CHARTER_REL
    batch = _load_json(TILE_BATCH_REL, repo=root)
    keys = [str(v.get("variant_key")) for v in batch.get("variants") or []]
    atlas = batch.get("atlas") or {}
    cells = int(atlas.get("columns") or 0) * int(atlas.get("rows") or 0)
    matrix_ref = str(batch.get("matrix_ref") or batch.get("_meta", {}).get("charter") or "")
    g0_text = (root / G0_RULES_REL).read_text(encoding="utf-8") if (root / G0_RULES_REL).is_file() else ""
    catalog_text = (root / CATALOG_REL).read_text(encoding="utf-8") if (root / CATALOG_REL).is_file() else ""
    missing_in_catalog = [k for k in keys if k not in catalog_text]
    checks = {
        "charter_on_disk": charter.is_file(),
        "variant_count_16": len(keys) == EXPECTED_MATRIX_CELLS,
        "atlas_grid_4x4": cells == EXPECTED_MATRIX_CELLS,
        "matrix_ref_points_charter": E4_CHARTER_REL.replace("\\", "/") in matrix_ref.replace("\\", "/"),
        "g0_proceed_production_bake": "proceed_production_bake: yes" in g0_text,
        "catalog_topology_keys": len(missing_in_catalog) == 0,
        "bake_source_keyframe_pack": batch.get("bake_source") == "keyframe_pack",
        "render_seed_550005": (batch.get("render") or {}).get("seed") == 550005,
    }
    green = all(checks.values())
    return {
        "gate": "DMCP-E4-MATRIX-CHARTER-001",
        "green": green,
        "verdict": "PASS" if green else "FAIL",
        "charter": E4_CHARTER_REL,
        "tile_batch": TILE_BATCH_REL,
        "variant_keys": keys,
        "checks": checks,
        "blocks": ["APS-EVO-E4-ATLAS-EXPAND-001"] if green else [],
        "missing_in_catalog": missing_in_catalog,
    }


def refresh_dmcp_e4_matrix_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    body = verify_e4_matrix_charter(repo=root)
    body["reviewed_at"] = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    body["_agent_meta"] = {
        "schema": "dmcp_e4_matrix_charter_live_v1",
        "written_at_epoch_secs": int(time.time()),
        "profile": "DMCP_E4_MATRIX_CHARTER",
        "source_system": "dmcp_designer_signoff",
        "relative_path": E4_WITNESS_REL,
        "ritual": "BLANG:WIT-HON→Q✓ DMCP-E4-MATRIX-CHARTER-001" if body.get("green") else None,
        "agent": "designer-mcp",
    }
    out = root / E4_WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = E4_WITNESS_REL
    return body


def verify_e0_artist_reverdict(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    e2e_path = root / E0_E2E_REL
    review_path = root / E0_REVIEW_REL
    e2e = _load_json(E0_E2E_REL, repo=root) if e2e_path.is_file() else {}
    honest = str(e2e.get("honest_gate") or "")
    checks = {
        "e0_witness_green": e2e.get("green") is True,
        "import_guard_pass": e2e.get("import_guard_pass") is True,
        "steps_ok": e2e.get("steps_ok") is True,
        "honest_gate_not_dishonest": honest != "dishonest_gate",
        "review_doc_on_disk": review_path.is_file(),
    }
    green = all(checks.values())
    return {
        "gate": "DMCP-E0-ARTIST-REVERDICT-001",
        "green": green,
        "verdict": "PASS_WITH_NOTES" if green else "FAIL",
        "depends_on": "APS-EVO-E0-RELAUNCH-001",
        "e0_witness": E0_E2E_REL,
        "review_doc": E0_REVIEW_REL,
        "ship_score": "7/10",
        "checks": checks,
        "artist_path": e2e.get("artist_path"),
        "track_b_deferred": e2e.get("track_b_deferred"),
    }


def stamp_e0_designer_signoff(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    e2e_path = root / E0_E2E_REL
    e2e = _load_json(E0_E2E_REL, repo=root)
    e2e["designer_mcp_signoff"] = "signed"
    e2e["designer_mcp_review"] = E0_REVIEW_REL
    e2e["designer_mcp_reverdict_gate"] = "DMCP-E0-ARTIST-REVERDICT-001"
    meta = dict(e2e.get("_agent_meta") or {})
    meta["designer_mcp_signed_at_epoch_secs"] = int(time.time())
    e2e["_agent_meta"] = meta
    e2e_path.write_text(json.dumps(e2e, indent=2) + "\n", encoding="utf-8")
    return {"stamped": E0_E2E_REL, "designer_mcp_signoff": "signed"}


def refresh_dmcp_e0_artist_reverdict_witness(*, repo: Path | None = None, stamp: bool = True) -> dict[str, Any]:
    root = repo or repo_root()
    body = verify_e0_artist_reverdict(repo=root)
    if stamp and body.get("green"):
        body["e0_stamp"] = stamp_e0_designer_signoff(repo=root)
    body["reviewed_at"] = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    body["_agent_meta"] = {
        "schema": "dmcp_e0_artist_reverdict_live_v1",
        "written_at_epoch_secs": int(time.time()),
        "profile": "DMCP_E0_ARTIST_REVERDICT",
        "source_system": "dmcp_designer_signoff",
        "relative_path": E0_WITNESS_REL,
        "ritual": "BLANG:WIT-HON→Q✓ DMCP-E0-ARTIST-REVERDICT-001" if body.get("green") else None,
        "agent": "designer-mcp",
    }
    out = root / E0_WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = E0_WITNESS_REL
    return body
