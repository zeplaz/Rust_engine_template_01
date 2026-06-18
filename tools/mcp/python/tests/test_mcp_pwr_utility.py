"""Tests for MCP-PWR-UTILITY manifest + witness gates."""

from __future__ import annotations

import json

from rust_engine_mcp.mcp_pwr_utility import (
    CONSTITUENT_STUBS,
    MANIFEST_REL,
    SUBSTATION_SPEC_REL,
    TRANSFORMER_SPEC_REL,
    audit_utility_manifest,
    ensure_utility_manifest_assets,
    refresh_pwr_utility_manifest_witness,
)
from rust_engine_mcp.paths import repo_root


def test_ensure_utility_manifest_assets_writes_constituents():
    root = repo_root()
    result = ensure_utility_manifest_assets(repo=root)
    assert result["constituent_count"] == 6
    for stub in CONSTITUENT_STUBS:
        assert (root / stub.spec_rel).is_file()
        assert (root / "tools/mcp/schemas/examples" / f"{stub.job_id}.json").is_file()


def test_manifest_audit_green_after_stubs():
    ensure_utility_manifest_assets()
    audit = audit_utility_manifest()
    assert audit["manifest_modules"] >= 8
    assert audit["substation_spec_valid"] is True
    assert audit["transformer_spec_valid"] is True
    assert audit["green"] is True


def test_manifest_witness_written():
    body = refresh_pwr_utility_manifest_witness()
    path = repo_root() / body["written"]
    assert path.is_file()
    loaded = json.loads(path.read_text(encoding="utf-8"))
    assert loaded["manifest_modules"] >= 8
    assert loaded["substation_spec_valid"] is True
    assert loaded["transformer_spec_valid"] is True


def test_manifest_lists_transformer_and_substation():
    manifest = json.loads((repo_root() / MANIFEST_REL).read_text(encoding="utf-8"))
    job_ids = {row["job_id"] for row in manifest["modules"]}
    assert "kit_substation_yard_production_run001" in job_ids
    assert "prop_transformer_production_run001" in job_ids


def test_core_specs_exist():
    root = repo_root()
    assert (root / SUBSTATION_SPEC_REL).is_file()
    assert (root / TRANSFORMER_SPEC_REL).is_file()
