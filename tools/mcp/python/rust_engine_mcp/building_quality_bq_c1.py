"""BQ-C1-CONTRACT-001 — module geometric contract schema + Python/Rust parity witness."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from rust_engine_mcp import module_contract
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.schemas import load_json_file, validate_module_contract

TASK_ID = "BQ-C1-CONTRACT-001"
WITNESS_REL = "debug_runs/bq_c1_contract_001_live.json"
CONTRACT_JSON_REL = module_contract.CONTRACT_JSON_REL
CONTRACT_SCHEMA_REL = "tools/mcp/schemas/module_contract_v1.schema.json"
CONTRACT_MD_REL = "tools/mcp/schemas/module_contract_v1.md"


def contract_paths(*, repo: Path | None = None) -> dict[str, Path]:
    root = repo or repo_root()
    return {
        "json": root / CONTRACT_JSON_REL,
        "schema": root / CONTRACT_SCHEMA_REL,
        "md": root / CONTRACT_MD_REL,
    }


def load_contract(*, repo: Path | None = None) -> dict[str, Any]:
    paths = contract_paths(repo=repo)
    data = load_json_file(paths["json"])
    validate_module_contract(data)
    return data


def parity_status(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    paths = contract_paths(repo=root)
    missing = [name for name, path in paths.items() if not path.is_file()]
    contract = load_contract(repo=root)
    grid_ok = contract["grid_unit_m"] == module_contract.GRID_UNIT_M
    floor_ok = contract["floor_height_m"] == module_contract.FLOOR_HEIGHT_M
    pivot_ok = contract["pivot_convention"] == module_contract.PIVOT_CONVENTION
    edge_ok = contract.get("edge_socket_names") == list(module_contract.EDGE_SOCKET_NAMES)
    families = contract.get("module_families") or {}
    wall_ok = (families.get("wall") or {}).get("height_m") == module_contract.FLOOR_HEIGHT_M
    return {
        "missing_files": missing,
        "schema_valid": True,
        "grid_unit_m": grid_ok,
        "floor_height_m": floor_ok,
        "pivot_convention": pivot_ok,
        "edge_socket_names": edge_ok,
        "wall_height_m": wall_ok,
        "green": not missing and grid_ok and floor_ok and pivot_ok and edge_ok and wall_ok,
    }


def write_bq_c1_witness(*, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    root = repo or repo_root()
    status = parity_status(repo=root)
    contract = load_contract(repo=root)
    green = bool(status.get("green"))
    body: dict[str, Any] = {
        "task_id": TASK_ID,
        "gate": TASK_ID,
        "green": green,
        "contract_json": CONTRACT_JSON_REL,
        "contract_schema": CONTRACT_SCHEMA_REL,
        "contract_md": CONTRACT_MD_REL,
        "python": {
            "grid_unit_m": module_contract.GRID_UNIT_M,
            "floor_height_m": module_contract.FLOOR_HEIGHT_M,
            "pivot_convention": module_contract.PIVOT_CONVENTION,
            "edge_socket_names": list(module_contract.EDGE_SOCKET_NAMES),
        },
        "contract": contract,
        "parity": status,
        "plan_ref": "src/dev/plan_building_quality_v1.md#BQ-C1",
    }
    return write_aps_live_witness(
        body,
        WITNESS_REL,
        schema="building_quality_bq_c1_live_v1",
        profile="BQ_C1_CONTRACT",
        source_system="building_quality_bq_c1",
        ritual=f"BLANG:WIT-HON {TASK_ID}" if green else None,
        repo=root,
    )
