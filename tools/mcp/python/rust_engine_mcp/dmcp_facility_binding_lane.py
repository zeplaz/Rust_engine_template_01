"""DMCP facility binding lane — schema + FactoryCluster binding + concrete site pilot."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.schemas import load_json_file, validate_building_grammar

WITNESS_REL = "debug_runs/art_pipeline/dmcp_facility_binding_lane_live.json"

POWER_TIER_BANDS: tuple[tuple[int, int, str], ...] = (
    (0, 30, "light"),
    (31, 80, "medium"),
    (81, 200, "heavy"),
)

LANE_SLICES: tuple[dict[str, str], ...] = (
    {
        "id": "DES-FACILITY-BINDING-001",
        "deliverable": "src/dev/design_facility_binding_schema_v1.md",
        "schema": "tools/mcp/schemas/facility_binding_v1.schema.json",
    },
    {
        "id": "DMCP-GRAM-FACTORY-CLUSTER-001",
        "grammar_ron": "assets/configs/buildings/grammars/factory_cluster_v1.ron",
        "grammar_json": "tools/mcp/schemas/examples/building_grammar_factory_cluster_v1.json",
    },
    {
        "id": "DMCP-PILOT-CONCRETE-SITE-001",
        "pilot": "assets/configs/buildings/pilots/concrete_portland_chain_pilot_v1.json",
    },
)

CONCRETE_SITE_PATHS: tuple[str, ...] = (
    "assets/configs/buildings/pilots/concrete_aggregate_mine_site_v0.json",
    "assets/configs/buildings/pilots/concrete_cement_kiln_site_v0.json",
    "assets/configs/buildings/pilots/concrete_mixer_plant_site_v0.json",
)

CONCRETE_STEP_ROLES: tuple[str, ...] = (
    "aggregate_mine",
    "cement_kiln",
    "concrete_mixer",
)

GRAMMAR_BINDING_EXPECTATIONS: dict[str, dict[str, str]] = {
    "factory_cluster_v1": {
        "catalog_id": "concrete_mixer_plant",
        "chain_id": "concrete_portland",
        "supply_chain_role": "concrete_mixer",
        "site_template_id": "concrete_mixer_plant_site_v0",
    },
    "rail_edge_v1": {
        "catalog_id": "logistics_rail_warehouse",
        "chain_id": "logistics_storage",
        "supply_chain_role": "rail_warehouse",
        "site_template_id": "logistics_rail_warehouse_site_v0",
    },
    "industrial_warehouse_v1": {
        "catalog_id": "logistics_storage_warehouse",
        "chain_id": "logistics_storage",
        "supply_chain_role": "storage_warehouse",
        "site_template_id": "logistics_storage_warehouse_site_v0",
    },
}


def power_tier_for_units(units: int) -> str:
    for lo, hi, label in POWER_TIER_BANDS:
        if lo <= units <= hi:
            return label
    return "grid"


def _load_chain_step(root: Path, chain_id: str, role: str) -> dict[str, Any] | None:
    chains_path = root / "assets/configs/industrial_supply_chains.json"
    if not chains_path.is_file():
        return None
    body = json.loads(chains_path.read_text(encoding="utf-8"))
    chain = (body.get("chains") or {}).get(chain_id) or {}
    for step in chain.get("steps") or []:
        if str(step.get("role")) == role:
            return step
    return None


def _check_site_grid(root: Path, rel: str) -> dict[str, Any]:
    path = root / rel
    if not path.is_file():
        return {"green": False, "checks": {"on_disk": False}}
    body = json.loads(path.read_text(encoding="utf-8"))
    width = int(body.get("width") or 0)
    depth = int(body.get("depth") or 0)
    cells = body.get("cells") or []
    expected = width * depth
    checks = {
        "on_disk": True,
        "schema_version": body.get("schema_version") == "site_zone_grid_v1",
        "cell_count": len(cells) == expected and expected > 0,
        "chain_id": body.get("chain_id") == "concrete_portland",
    }
    return {"green": all(checks.values()), "checks": checks, "site_id": body.get("site_id")}


def _check_grammar_binding(root: Path, grammar_id: str) -> dict[str, Any]:
    expected = GRAMMAR_BINDING_EXPECTATIONS.get(grammar_id)
    if expected is None:
        return {"green": False, "checks": {"known_grammar": False}}
    json_path = root / f"tools/mcp/schemas/examples/building_grammar_{grammar_id}.json"
    ron_path = root / f"assets/configs/buildings/grammars/{grammar_id}.ron"
    if not json_path.is_file() or not ron_path.is_file():
        return {"green": False, "checks": {"files_on_disk": False}}
    grammar = load_json_file(json_path)
    validate_building_grammar(grammar)
    binding = grammar.get("facility_binding") or {}
    catalog_id = str(binding.get("catalog_id") or "")
    catalog_path = root / f"assets/configs/buildings/{catalog_id}.json"
    catalog = json.loads(catalog_path.read_text(encoding="utf-8")) if catalog_path.is_file() else {}
    power_units = int(catalog.get("power_consumption") or 0)
    expected_tier = power_tier_for_units(power_units)
    chain_row = _load_chain_step(
        root,
        str(binding.get("chain_id") or ""),
        str(binding.get("supply_chain_role") or ""),
    )
    ron_text = ron_path.read_text(encoding="utf-8")
    checks = {
        "files_on_disk": True,
        "schema_valid": True,
        "catalog_id": catalog_id == expected["catalog_id"],
        "chain_id": binding.get("chain_id") == expected["chain_id"],
        "role_match": binding.get("supply_chain_role") == catalog.get("supply_chain_role"),
        "chain_step_match": chain_row is not None
        and str(chain_row.get("catalog_id")) == catalog_id,
        "power_tier_derived": binding.get("power_tier") == expected_tier,
        "site_template_id": binding.get("site_template_id") == expected["site_template_id"],
        "ron_has_binding": "facility_binding:" in ron_text,
    }
    return {
        "green": all(checks.values()),
        "checks": checks,
        "grammar_id": grammar_id,
        "catalog_power_units": power_units,
        "expected_power_tier": expected_tier,
    }


def _check_all_grammar_bindings(root: Path) -> dict[str, Any]:
    rows = [_check_grammar_binding(root, gid) for gid in GRAMMAR_BINDING_EXPECTATIONS]
    green_count = sum(1 for r in rows if r.get("green"))
    return {
        "green": green_count == len(rows),
        "binding_count": len(rows),
        "green_binding_count": green_count,
        "grammars": rows,
    }


def _check_factory_binding(root: Path) -> dict[str, Any]:
    return _check_grammar_binding(root, "factory_cluster_v1")


def _check_concrete_pilot(root: Path) -> dict[str, Any]:
    pilot_path = root / "assets/configs/buildings/pilots/concrete_portland_chain_pilot_v1.json"
    if not pilot_path.is_file():
        return {"green": False, "checks": {"pilot_on_disk": False}}
    pilot = json.loads(pilot_path.read_text(encoding="utf-8"))
    steps = pilot.get("steps") or []
    step_checks: list[dict[str, Any]] = []
    all_green = True
    for role in CONCRETE_STEP_ROLES:
        step = next((s for s in steps if str(s.get("role")) == role), None)
        if step is None:
            step_checks.append({"role": role, "green": False, "error": "missing_step"})
            all_green = False
            continue
        chain_row = _load_chain_step(root, "concrete_portland", role)
        catalog_id = str(step.get("catalog_id") or "")
        catalog_path = root / f"assets/configs/buildings/{catalog_id}.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8")) if catalog_path.is_file() else {}
        ref_power = int(step.get("power_consumption_ref") or 0)
        catalog_power = int(catalog.get("power_consumption") or 0)
        site_rel = str(step.get("site_plan_json") or "")
        site_audit = _check_site_grid(root, site_rel)
        checks = {
            "catalog_on_disk": catalog_path.is_file(),
            "catalog_id_chain_match": chain_row is not None
            and str(chain_row.get("catalog_id")) == catalog_id,
            "power_ref_match": ref_power == catalog_power,
            "power_tier_match": step.get("power_tier") == power_tier_for_units(catalog_power),
            "site_grid": site_audit.get("green") is True,
        }
        green = all(checks.values())
        if not green:
            all_green = False
        step_checks.append({"role": role, "green": green, "checks": checks})
    checks = {
        "pilot_on_disk": True,
        "schema_version": pilot.get("schema_version") == "facility_chain_pilot_v1",
        "step_count_3": len(steps) == 3,
        "chain_id": pilot.get("chain_id") == "concrete_portland",
        "grammar_hint": pilot.get("grammar_archetype_hint") == "FactoryCluster",
    }
    return {
        "green": all_green and all(checks.values()),
        "checks": checks,
        "steps": step_checks,
    }


def _check_binding_schema(root: Path) -> dict[str, Any]:
    doc = root / "src/dev/design_facility_binding_schema_v1.md"
    schema = root / "tools/mcp/schemas/facility_binding_v1.schema.json"
    grammar_schema_path = root / "tools/mcp/schemas/building_grammar_v1.schema.json"
    if not doc.is_file() or not schema.is_file() or not grammar_schema_path.is_file():
        return {"green": False, "checks": {"artifacts_on_disk": False}}
    grammar_schema = json.loads(grammar_schema_path.read_text(encoding="utf-8"))
    props = grammar_schema.get("properties") or {}
    checks = {
        "artifacts_on_disk": True,
        "standalone_schema": schema.is_file(),
        "grammar_schema_has_binding": "facility_binding" in props,
    }
    return {"green": all(checks.values()), "checks": checks}


def run_facility_binding_lane_audit(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    rows: list[dict[str, Any]] = []
    for spec in LANE_SLICES:
        gate = spec["id"]
        row: dict[str, Any] = {"id": gate, "status": "fail"}
        if gate == "DES-FACILITY-BINDING-001":
            row["deliverable"] = spec["deliverable"]
            row["schema_audit"] = _check_binding_schema(root)
            row["deliverable_exists"] = (root / spec["deliverable"]).is_file()
            row["status"] = (
                "done"
                if row["schema_audit"].get("green") and row["deliverable_exists"]
                else "fail"
            )
            row["verdict"] = "PASS"
        elif gate == "DMCP-GRAM-FACTORY-CLUSTER-001":
            row["grammar_audit"] = _check_all_grammar_bindings(root)
            row["status"] = "done" if row["grammar_audit"].get("green") else "fail"
            row["verdict"] = "PASS"
        elif gate == "DMCP-PILOT-CONCRETE-SITE-001":
            row["pilot_audit"] = _check_concrete_pilot(root)
            row["status"] = "done" if row["pilot_audit"].get("green") else "fail"
            row["verdict"] = "PASS"
        rows.append(row)

    done = sum(1 for r in rows if r["status"] == "done")
    grammar_bindings = _check_all_grammar_bindings(root)
    green = done == len(rows) and grammar_bindings.get("green") is True
    return {
        "gate": "DMCP-FACILITY-BINDING-LANE-001",
        "lanes": [
            "facility_binding schema",
            "FactoryCluster + binding",
            "concrete 3-step site pilot",
            "all G1 grammars bound",
        ],
        "slice_count": len(rows),
        "done_count": done,
        "rows": rows,
        "grammar_bindings": grammar_bindings,
        "audit_complete": True,
        "green": green,
        "verdict": "PASS" if green else "FAIL",
        "handoff": {
            "designer": "DES-APS-FACILITY-NEEDS-001 sign-off on schema",
            "coder_mcp": "CMCP-GRAMMAR-FACILITY-BRIEF-001",
            "coder": "COD-FACILITY-BINDING-READ-001",
        },
    }


def refresh_dmcp_facility_binding_lane_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    body = run_facility_binding_lane_audit(repo=root)
    body["_agent_meta"] = {
        "schema": "dmcp_facility_binding_lane_live_v1",
        "written_at_epoch_secs": int(time.time()),
        "profile": "DMCP_FACILITY_BINDING",
        "source_system": "dmcp_facility_binding_lane",
        "relative_path": WITNESS_REL,
        "ritual": "BLANG:WIT-HON→Q✓ DMCP-FACILITY-BINDING" if body.get("green") else "BLANG:WIT-HON FAIL",
        "agent": "designer-mcp",
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = WITNESS_REL
    return body
