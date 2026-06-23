"""Tests for MCP-PWR-NUCLEAR manifest + witness gates."""

from __future__ import annotations

import json

from rust_engine_mcp.mcp_pwr_nuclear import (
    KIT_SPEC_REL,
    MANIFEST_REL,
    NUCLEAR_CONSTITUENT_STUBS,
    audit_nuclear_manifest,
    ensure_nuclear_manifest_assets,
    refresh_nuclear_manifest_witness,
)
from rust_engine_mcp.paths import repo_root


def test_ensure_nuclear_manifest_assets_writes_constituents():
    root = repo_root()
    result = ensure_nuclear_manifest_assets(repo=root)
    assert result["constituent_count"] == 6
    for stub in NUCLEAR_CONSTITUENT_STUBS:
        assert (root / stub.spec_rel).is_file()
        assert (root / "tools/mcp/schemas/examples" / f"{stub.job_id}.json").is_file()


def test_nuclear_manifest_audit_green_after_stubs():
    ensure_nuclear_manifest_assets()
    audit = audit_nuclear_manifest()
    assert audit["manifest_modules"] >= 8
    assert audit["kit_spec_valid"] is True
    assert audit["green"] is True


def test_nuclear_manifest_witness_written():
    body = refresh_nuclear_manifest_witness()
    path = repo_root() / body["written"]
    assert path.is_file()
    loaded = json.loads(path.read_text(encoding="utf-8"))
    assert loaded["manifest_modules"] >= 8
    assert loaded["kit_spec_valid"] is True


def test_nuclear_manifest_lists_kit_job():
    manifest = json.loads((repo_root() / MANIFEST_REL).read_text(encoding="utf-8"))
    job_ids = {row["job_id"] for row in manifest["modules"]}
    assert "kit_nuclear_pwr_production_run001" in job_ids
    assert "containment_dome_pwr_production_run001" in job_ids


def test_nuclear_kit_spec_exists():
    root = repo_root()
    assert (root / KIT_SPEC_REL).is_file()
