"""Tests for BQ-C1-CONTRACT-001 — module geometric contract."""

from __future__ import annotations

from rust_engine_mcp import building_quality_bq_c1, module_contract, schemas
from rust_engine_mcp.paths import repo_root


def test_module_contract_json_validates() -> None:
    root = repo_root()
    path = root / module_contract.CONTRACT_JSON_REL
    data = schemas.load_json_file(path)
    schemas.validate_module_contract(data)


def test_python_constants_match_contract() -> None:
    status = building_quality_bq_c1.parity_status()
    assert status["green"] is True


def test_bq_c1_witness_green() -> None:
    root = repo_root()
    body = building_quality_bq_c1.write_bq_c1_witness(repo=root)
    assert body["task_id"] == "BQ-C1-CONTRACT-001"
    assert body["green"] is True
    witness = root / building_quality_bq_c1.WITNESS_REL
    assert witness.is_file()
