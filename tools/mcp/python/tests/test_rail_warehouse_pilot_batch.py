"""BUILD-READ-VISUAL-002 — rail warehouse pilot keyframe batch."""

from __future__ import annotations

import json

from rust_engine_mcp import rail_warehouse_pilot_batch
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators import run_validator


def test_staging_spec_has_four_variants():
    spec = rail_warehouse_pilot_batch.load_staging_spec()
    keys = rail_warehouse_pilot_batch._variant_keys_from_spec(spec)
    assert keys == [
        "clean_day",
        "clean_night_off",
        "clean_night_on",
        "under_construction_01",
    ]
    assert spec["bake_source"] == "keyframe_pack"
    assert spec["seed"] == 440013


def test_write_batch_artifacts():
    written = rail_warehouse_pilot_batch.write_rail_warehouse_pilot_batch_artifacts()
    assert (repo_root() / written["tile_batch"]).is_file()
    assert (repo_root() / written["variant_set"]).is_file()
    assert (repo_root() / written["building_definition"]).is_file()
    assert (repo_root() / written["assembly_snapshot"]).is_file()
    batch = json.loads((repo_root() / written["tile_batch"]).read_text(encoding="utf-8"))
    assert batch["batch_id"] == "tile_rail_warehouse_pilot_v1"
    assert batch["ship"] is False
    assert len(batch["variants"]) == 4


def test_validate_tile_batch():
    rail_warehouse_pilot_batch.write_rail_warehouse_pilot_batch_artifacts()
    rel = "tools/mcp/schemas/examples/tile_batch_rail_warehouse_pilot_v1.json"
    rep = run_validator("tile_batch", rel, compression_level=4)
    assert rep.status == "passed"


def test_batch_witness():
    body = rail_warehouse_pilot_batch.refresh_rail_warehouse_pilot_batch_witness()
    assert body["gate_id"] == "BUILD-READ-VISUAL-002-BATCH"
    assert body["variant_keys"] == [
        "clean_day",
        "clean_night_off",
        "clean_night_on",
        "under_construction_01",
    ]
    assert (repo_root() / rail_warehouse_pilot_batch.WITNESS_PATH).is_file()
    design = json.loads(
        (repo_root() / "debug_runs/design_tile_rail_warehouse_pilot_live.json").read_text(
            encoding="utf-8"
        )
    )
    assert design.get("impl_wired") is True


def test_g4_signoff_and_runtime_register():
    staging = repo_root() / "assets/staging/tiles/tile_rail_warehouse_pilot_v1"
    if not (staging / "clean_day.png").is_file():
        pytest.skip("keyframe stills not baked")
    g4 = rail_warehouse_pilot_batch.apply_rail_warehouse_pilot_g4_signoff()
    assert g4["ok"] is True
    assert g4["proceed_ship"] is True
    signoff = repo_root() / "debug_runs/art_pipeline/rail_warehouse_pilot_production_signoff.yaml"
    text = signoff.read_text(encoding="utf-8")
    assert "proceed_ship: yes" in text
    assert "g4_3_keyframe_minimum_stills_review: pass" in text
    reg = rail_warehouse_pilot_batch.register_rail_warehouse_pilot_for_runtime()
    assert reg["ok"] is True
    entry = reg["entry"]
    assert entry["ship_allowed"] is True
    assert entry["development_tier"] == "production"
