"""MCP-P2-KIT002-G3 — validate_asset_report tier pass on kit_production_002 manifest."""

from __future__ import annotations

import json

from rust_engine_mcp import kit_production_002
from rust_engine_mcp.paths import repo_root


def test_g3_batch_all_six_promoted_glbs_pass() -> None:
    batch = kit_production_002.validate_kit_production_002_g3_batch()
    assert batch.get("module_count") == 6
    assert batch.get("passed_count") == 6
    assert batch.get("all_modules_passed") is True
    assert batch.get("green") is True
    for row in batch.get("modules") or []:
        rep = row.get("validate_asset_report") or {}
        assert row.get("passed") is True, row
        assert rep.get("status") in ("passed", "warning"), rep
        assert rep.get("compression_level") == kit_production_002.G3_ASSET_COMPRESSION


def test_g3_honest_bake_on_paired_tile_batch() -> None:
    batch = kit_production_002.validate_kit_production_002_g3_batch()
    honest = batch.get("tile_promotion_honest") or {}
    assert honest.get("ship") is False
    assert honest.get("ok") is True
    assert honest.get("status") in ("passed", "warning")
    assert honest.get("bake_source") == "keyframe_pack"


def test_g3_witness_green_and_manifest_gate() -> None:
    manifest_before = kit_production_002.load_manifest()
    body = kit_production_002.refresh_kit_production_002_g3_witness()
    assert body.get("green") is True
    assert body.get("verdict") == "PASS"
    assert body.get("validate_asset_report", {}).get("all_passed") is True
    assert body.get("proceed_tile_ship") is False
    assert "MCP-P2-KIT002-G4" in (body.get("unblocks") or [])

    manifest_after = kit_production_002.load_manifest()
    assert manifest_after.get("gate") == "G3"
    assert manifest_before.get("gate") in ("G2", "G3")

    witness_path = repo_root() / kit_production_002.G3_WITNESS_REL
    assert witness_path.is_file()
    on_disk = json.loads(witness_path.read_text(encoding="utf-8"))
    assert on_disk.get("green") is True

    batch_witness = repo_root() / kit_production_002.BATCH_WITNESS_REL
    batch_body = json.loads(batch_witness.read_text(encoding="utf-8"))
    assert batch_body.get("gate") == "G3"
    assert batch_body.get("green") is True
