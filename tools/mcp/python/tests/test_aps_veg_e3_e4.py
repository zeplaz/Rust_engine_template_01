"""APS-E3/E4 + CDR-B veg catalog and resolver parity tests."""

from __future__ import annotations

import json
import os

import pytest

from rust_engine_mcp.aps_veg_state_axis import (
    APS_VEG_STATE_AXIS_WITNESS,
    refresh_aps_veg_state_axis_witness,
    verify_veg_state_axis,
)
from rust_engine_mcp.landscape_lg5_expanded_batch import (
    EXPANDED_WITNESS_REL,
    refresh_tile_landscape_expanded_witness,
    write_landscape_expanded_keyframes,
)
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.veg_catalog_loader import (
    burn_variant_keys,
    catalog_validator_report,
    state_axis_rows,
)
from rust_engine_mcp.veg_resolver_parity import (
    DOC_REL,
    ENGINE_VEG_RESOLVER_KEYS,
    check_resolver_catalog_parity,
    refresh_veg_resolver_parity_witness,
)


def test_catalog_validator_green() -> None:
    rep = catalog_validator_report()
    assert rep.get("green") is True
    assert rep.get("veg_burn_count") == 8


def test_state_axis_rows_include_burn_and_topology() -> None:
    rows = state_axis_rows()
    axes = {r["axis"] for r in rows}
    assert "burn" in axes
    assert "succession" in axes
    assert "topology_state" in axes
    assert len(burn_variant_keys()) >= 8


def test_resolver_catalog_byte_parity() -> None:
    body = check_resolver_catalog_parity()
    assert body.get("green") is True
    assert body.get("missing_in_catalog") == []
    assert body.get("extra_in_catalog") == []
    assert len(ENGINE_VEG_RESOLVER_KEYS) == 13


def test_veg_resolver_parity_witness_and_doc() -> None:
    body = refresh_veg_resolver_parity_witness()
    assert body.get("green") is True
    doc = repo_root() / DOC_REL
    assert doc.is_file()
    text = doc.read_text(encoding="utf-8")
    assert "veg_burn_00" in text
    witness = repo_root() / body["written"]
    assert witness.is_file()


def test_aps_veg_state_axis_witness() -> None:
    body = refresh_aps_veg_state_axis_witness()
    assert body.get("green") is True
    assert body.get("catalog_validator_green") is True
    assert body.get("burn_variants_ok") is True
    assert body.get("v2_labels_wired") is True
    assert body.get("witness_honesty", {}).get("status") == "passed"
    out = repo_root() / APS_VEG_STATE_AXIS_WITNESS
    assert out.is_file()


def test_verify_veg_state_axis_headless() -> None:
    check = verify_veg_state_axis()
    assert check.get("catalog_validator_green") is True
    assert check.get("states_panel_catalog_wired") is True
    assert check.get("v2_labels_wired") is True


def test_expanded_keyframes_write_sixteen_pngs(tmp_path) -> None:
    paths = write_landscape_expanded_keyframes(tmp_path)
    assert len(paths) == 16
    assert all(p.is_file() for p in paths)


@pytest.fixture
def tile_dry_run(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("RUST_ENGINE_TILE_DRY_RUN", "1")


def test_expanded_atlas_batch_witness(tile_dry_run: None) -> None:
    prev = os.environ.get("RUST_ENGINE_TILE_DRY_RUN")
    body = refresh_tile_landscape_expanded_witness()
    assert body.get("green") is True
    assert body.get("png_count", 0) > 3
    assert body.get("atlas_domain") == "landscape"
    assert body.get("bake_source") == "keyframe_pack"
    rollup = repo_root() / EXPANDED_WITNESS_REL
    assert rollup.is_file()
    data = json.loads(rollup.read_text(encoding="utf-8"))
    assert data.get("variant_count") == 16
    _ = prev
