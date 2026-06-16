"""LG-5 landscape atlas batch — validate spec + registry stamp."""

from __future__ import annotations

from rust_engine_mcp.landscape_atlas_index import landscape_lg5_registry_stamped
from rust_engine_mcp.landscape_lg5_batch import BATCH_JSON, run_landscape_lg5_atlas_batch
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators import run_validator


def test_tile_batch_landscape_lg5_validates():
    report = run_validator("tile_batch", str(BATCH_JSON))
    assert report.status == "passed", report.summary


def test_landscape_lg5_atlas_batch_green():
    result = run_landscape_lg5_atlas_batch(refresh_keyframes=False)
    assert result.get("ok") is True
    assert result.get("witness_green") is True
    assert landscape_lg5_registry_stamped()
    witness = repo_root() / "debug_runs/art_pipeline/tile_tile_landscape_lg5_pilot_v1_live.json"
    assert witness.is_file()
    import json

    body = json.loads(witness.read_text(encoding="utf-8"))
    assert body.get("green") is True
