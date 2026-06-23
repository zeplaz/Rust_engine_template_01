"""DMCP-SPEC-TRANSFORMER-PAD-001 — transformer pad prop spec witness (spec ready, bpy pending)."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

WITNESS_REL = "debug_runs/art_pipeline/dmcp_transformer_pad_spec_live.json"
SPEC_REL = "assets/staging/specs/prop_transformer_production_run001.json"
DMCP_DOC_REL = "src/dev/dmcp_spec_transformer_pad_v1.md"
CATALOG_REL = "assets/configs/buildings/grid_distribution_transformer.json"
GATE_ID = "DMCP-SPEC-TRANSFORMER-PAD-001"

EXPECTED_ASSET_ID = "prop_transformer_production_run001"
EXPECTED_GRID = [2, 2]
EXPECTED_ARCHETYPE = "module_prop"
EXPECTED_STYLE_PACK = "style_industrial_west"
EXPECTED_BATCH_ID = "kit_utility_power_production_001"


def _load_spec(root: Path) -> dict[str, Any] | None:
    path = root / SPEC_REL
    if not path.is_file():
        return None
    return json.loads(path.read_text(encoding="utf-8"))


def run_transformer_pad_spec_audit(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    spec_path = root / SPEC_REL
    spec = _load_spec(root)
    errors: list[str] = []

    if spec is None:
        return {
            "gate": GATE_ID,
            "asset_id": EXPECTED_ASSET_ID,
            "spec_file_exists": False,
            "grid_units": None,
            "green": False,
            "verdict": "FAIL",
            "errors": [f"missing spec: {SPEC_REL}"],
        }

    module = spec.get("module") or {}
    grid_units = module.get("grid_units")
    geometry = spec.get("geometry_notes") or {}
    refs = [str(r) for r in (spec.get("references") or [])]

    checks = {
        "spec_file_exists": spec_path.is_file(),
        "dmcp_doc_exists": (root / DMCP_DOC_REL).is_file(),
        "catalog_exists": (root / CATALOG_REL).is_file(),
        "asset_id": str(spec.get("asset_id") or "") == EXPECTED_ASSET_ID,
        "grid_units_2x2": grid_units == EXPECTED_GRID,
        "archetype_module_prop": spec.get("archetype") == EXPECTED_ARCHETYPE,
        "style_pack_industrial_west": spec.get("style_pack") == EXPECTED_STYLE_PACK,
        "development_tier_production": spec.get("development_tier") == "production",
        "batch_id": spec.get("batch_id") == EXPECTED_BATCH_ID,
        "snap_floor_center": module.get("snap") == "floor_center",
        "pivot_bottom_center": module.get("pivot") == "bottom_center",
        "bushings_three": int(geometry.get("bushings") or 0) == 3,
        "dmcp_ref": any("DMCP-SPEC-TRANSFORMER-PAD-001" in r for r in refs),
        "catalog_ref": any("grid_distribution_transformer" in r for r in refs),
    }

    if not checks["bushings_three"]:
        errors.append("geometry_notes.bushings must be 3")
    if not checks["dmcp_ref"]:
        errors.append("references must include DMCP-SPEC-TRANSFORMER-PAD-001")

    green = all(checks.values()) and not errors
    return {
        "gate": GATE_ID,
        "asset_id": str(spec.get("asset_id") or ""),
        "spec_file_exists": checks["spec_file_exists"],
        "grid_units": grid_units,
        "checks": checks,
        "errors": errors,
        "spec_only": True,
        "bpy_pending": True,
        "audit_complete": True,
        "green": green,
        "verdict": "PASS" if green else "FAIL",
        "handoff": {
            "coder_mcp": "MCP-PWR-TRANSFORMER-BATCH-001",
            "deliverable": SPEC_REL,
            "dmcp_doc": DMCP_DOC_REL,
            "catalog": CATALOG_REL,
        },
    }


def refresh_dmcp_transformer_pad_spec_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    body = run_transformer_pad_spec_audit(repo=root)
    body["_agent_meta"] = {
        "schema": "dmcp_transformer_pad_spec_live_v1",
        "written_at_epoch_secs": int(time.time()),
        "profile": "DMCP_TRANSFORMER_PAD_SPEC",
        "source_system": "dmcp_transformer_pad_spec",
        "relative_path": WITNESS_REL,
        "ritual": "BLANG:WIT-HON→Q✓ DMCP-SPEC-TRANSFORMER-PAD-001" if body.get("green") else "BLANG:WIT-HON FAIL",
        "agent": "designer-mcp",
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = WITNESS_REL
    return body
