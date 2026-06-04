"""Tests for variant_set_v1 tools (APS-VAR)."""

from __future__ import annotations

import json
import os
from pathlib import Path

import pytest

from rust_engine_mcp import variant_set
from rust_engine_mcp.schemas import validate_variant_set


@pytest.fixture
def example_variant_set() -> dict:
    path = variant_set.example_variant_set_path()
    return variant_set.load_variant_set(path)


def test_variant_set_validate_example(example_variant_set: dict) -> None:
    assert example_variant_set["variant_set_id"] == "rowhouse_victorian_night_damage"
    assert len(example_variant_set["variants"]) == 2


def test_variant_set_patch_replace_tag(tmp_path: Path, example_variant_set: dict) -> None:
    p = tmp_path / "test_set.json"
    variant_set.save_variant_set(example_variant_set, p)
    result = variant_set.variant_set_patch(
        p,
        [{"op": "add", "path": "/variants/0/tags/-", "value": "user_review"}],
    )
    doc = result["document"]
    assert "user_review" in doc["variants"][0]["tags"]


def test_layers_to_tile_variant(example_variant_set: dict) -> None:
    entry = example_variant_set["variants"][1]
    flat = variant_set.layers_to_tile_variant(entry)
    assert flat["variant_key"] == "damaged_night_on"
    assert flat["lighting"] == "night_on"
    assert flat["damage"] == 0.45


def test_variant_agent_request_writes_debug_run(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr("rust_engine_mcp.paths.repo_root", lambda: tmp_path)
    (tmp_path / "debug_runs" / "art_pipeline").mkdir(parents=True)
    body = {
        "assembly_id": "victorian_4x3_s42_a7cb",
        "variant_key": "damaged_night_on",
        "intent": "add_warm_window_lights",
        "current_layers": {"lighting": {"power": "on"}},
    }
    result = variant_set.variant_agent_request(body, write=True)
    assert result.get("patch")
    out = tmp_path / "debug_runs" / "art_pipeline" / "variant_agent_request.json"
    assert out.is_file()


def test_expand_variant_set_to_tile_batch(example_variant_set: dict) -> None:
    batch = variant_set.expand_variant_set_to_tile_batch(example_variant_set)
    assert batch["batch_id"].startswith("tile_")
    assert len(batch["variants"]) == 2
    assert "assembly_ref" in batch


def test_apply_json_patch() -> None:
    doc = {"variants": [{"variant_key": "a", "layers": {"lighting": {"power": "off"}}}]}
    patched = variant_set.apply_json_patch(
        doc, [{"op": "replace", "path": "/variants/0/layers/lighting/power", "value": "on"}]
    )
    assert patched["variants"][0]["layers"]["lighting"]["power"] == "on"
