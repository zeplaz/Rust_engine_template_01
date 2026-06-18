"""DMCP-SPEC-NUCLEAR-PWR-001 — nuclear PWR kit spec witness (spec only, no bpy)."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

WITNESS_REL = "debug_runs/art_pipeline/dmcp_nuclear_pwr_spec_live.json"
SPEC_REL = "assets/staging/specs/kit_nuclear_pwr_production_001.json"
DMCP_DOC_REL = "src/dev/dmcp_spec_nuclear_pwr_v1.md"
GATE_ID = "DMCP-SPEC-NUCLEAR-PWR-001"

EXPECTED_ASSET_ID = "kit_nuclear_pwr_production_001"
EXPECTED_GRID = [6, 6]
EXPECTED_UTILITY_ROLE = "nuclear"
EXPECTED_PLANT_ID = "pwr_4loop_1100mw_v1"
HERO_MODULE = "containment_dome_pwr"
HERO_GRID = [3, 3]

MODULE_WHITELIST: frozenset[str] = frozenset(
    {
        "containment_dome_pwr",
        "turbine_hall_1u",
        "cooling_tower_1u",
        "diesel_gen_pad_2x2",
        "switchyard_edge_1u",
        "fence_chainlink_1u",
        "warning_sign_nuclear_1u",
    }
)


def _load_spec(root: Path) -> dict[str, Any] | None:
    path = root / SPEC_REL
    if not path.is_file():
        return None
    return json.loads(path.read_text(encoding="utf-8"))


def run_nuclear_pwr_spec_audit(*, repo: Path | None = None) -> dict[str, Any]:
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

    asset_id = str(spec.get("asset_id") or "")
    grid_units = spec.get("module", {}).get("grid_units")
    whitelist = {str(m) for m in (spec.get("module_whitelist") or [])}
    composition_ids = {
        str(row.get("module_id"))
        for row in (spec.get("composition") or [])
        if row.get("module_id")
    }
    hero = spec.get("hero_module") or {}

    checks = {
        "spec_file_exists": spec_path.is_file(),
        "dmcp_doc_exists": (root / DMCP_DOC_REL).is_file(),
        "asset_id": asset_id == EXPECTED_ASSET_ID,
        "grid_units_6x6": grid_units == EXPECTED_GRID,
        "utility_role_nuclear": spec.get("utility_role") == EXPECTED_UTILITY_ROLE,
        "plant_definition_id": spec.get("plant_definition_id") == EXPECTED_PLANT_ID,
        "hero_module": hero.get("module_id") == HERO_MODULE and hero.get("grid") == HERO_GRID,
        "whitelist_complete": whitelist == MODULE_WHITELIST,
        "composition_covers_whitelist": MODULE_WHITELIST <= composition_ids,
        "spec_only_flag": spec.get("spec_only") is True,
        "bpy_blocked": spec.get("bpy_blocked") is True,
        "development_tier_production": spec.get("development_tier") == "production",
    }

    if not checks["utility_role_nuclear"]:
        errors.append("utility_role must be nuclear")
    if not checks["hero_module"]:
        errors.append(f"hero_module must be {HERO_MODULE} 3×3")
    if not checks["whitelist_complete"]:
        errors.append("module_whitelist mismatch vs massing §3")
    if not checks["spec_only_flag"] or not checks["bpy_blocked"]:
        errors.append("spec wave requires spec_only + bpy_blocked")

    green = all(checks.values()) and not errors
    return {
        "gate": GATE_ID,
        "asset_id": asset_id,
        "spec_file_exists": checks["spec_file_exists"],
        "grid_units": grid_units,
        "utility_role": spec.get("utility_role"),
        "plant_definition_id": spec.get("plant_definition_id"),
        "hero_module": hero,
        "module_whitelist": sorted(whitelist),
        "checks": checks,
        "errors": errors,
        "spec_only": True,
        "bpy_blocked": True,
        "audit_complete": True,
        "green": green,
        "verdict": "PASS" if green else "FAIL",
        "handoff": {
            "coder_mcp": "MCP-PWR-NUCLEAR-BATCH-001",
            "deliverable": SPEC_REL,
            "dmcp_doc": DMCP_DOC_REL,
        },
    }


def refresh_dmcp_nuclear_pwr_spec_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    body = run_nuclear_pwr_spec_audit(repo=root)
    body["_agent_meta"] = {
        "schema": "dmcp_nuclear_pwr_spec_live_v1",
        "written_at_epoch_secs": int(time.time()),
        "profile": "DMCP_NUCLEAR_PWR_SPEC",
        "source_system": "dmcp_nuclear_pwr_spec",
        "relative_path": WITNESS_REL,
        "ritual": "BLANG:WIT-HON→Q✓ DMCP-SPEC-NUCLEAR-PWR-001" if body.get("green") else "BLANG:WIT-HON FAIL",
        "agent": "designer-mcp",
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = WITNESS_REL
    return body
