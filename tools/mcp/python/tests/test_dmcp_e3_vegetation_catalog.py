"""DMCP-E3 — vegetation variant catalog on disk."""

from __future__ import annotations

from pathlib import Path

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.vegetation_variant_catalog import (
    CATALOG_RON_REL,
    build_catalog_body,
    refresh_dmcp_e3_witness,
    validate_catalog_body,
)


def test_catalog_validates_against_schema() -> None:
    body = build_catalog_body()
    rep = validate_catalog_body(body)
    assert rep["status"] == "passed"
    assert rep["entry_count"] == 35
    assert rep["veg_burn_count"] == 8
    assert rep["topology_count"] == 22
    review = rep["ship_review"]
    assert review["atlas_sparse_v1_complete"] is True
    assert review["engine_veg_byte_parity"] is True


def test_sparse_topology_keys_present() -> None:
    body = build_catalog_body()
    keys = {e["variant_key"] for e in body["entries"]}
    assert "topology_patch_burn_07" in keys
    assert "topology_patch_regrowth_canopy" in keys
    assert "topology_cluster_scar" in keys


def test_catalog_ron_written_by_witness() -> None:
    witness = refresh_dmcp_e3_witness()
    assert witness.get("green") is True
    ron = repo_root() / CATALOG_RON_REL
    assert ron.is_file()
    text = ron.read_text(encoding="utf-8")
    assert "vegetation_variant_catalog_v1" in text
    assert "veg_burn_07" in text
    assert "topology_patch_scar" in text
