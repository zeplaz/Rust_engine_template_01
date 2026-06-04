"""PT-0 — variant_catalog_v1 schema + example (PLAN-PROC-TILE-PROD-001)."""

from __future__ import annotations

from pathlib import Path

import pytest

from rust_engine_mcp.schemas import load_json_file, validate_variant_catalog


def _example_path() -> Path:
    return (
        Path(__file__).resolve().parents[2]
        / "schemas"
        / "examples"
        / "variant_catalog_v1.example.json"
    )


def test_variant_catalog_example_validates() -> None:
    data = load_json_file(_example_path())
    validate_variant_catalog(data)
    assert data["program_id"] == "PLAN-PROC-TILE-PROD-001"
    assert "clean_day" in data["canonical_variant_keys"]
    assert len(data["ship_minimum_keys"]) >= 6


def test_ship_minimum_subset_of_canonical() -> None:
    data = load_json_file(_example_path())
    canonical = set(data["canonical_variant_keys"])
    for key in data["ship_minimum_keys"]:
        assert key in canonical, f"ship key {key!r} not in canonical_variant_keys"


def test_fire_frame_keys_present() -> None:
    data = load_json_file(_example_path())
    canonical = set(data["canonical_variant_keys"])
    for i in range(data["fire"]["frame_count"]):
        assert f"burning_{i:02d}" in canonical
