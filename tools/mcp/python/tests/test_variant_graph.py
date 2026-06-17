"""ARCH-002 — variant_graph_v1 schema validation."""

from __future__ import annotations

from pathlib import Path

import pytest

from rust_engine_mcp.schemas import load_json_file, validate_variant_graph


def test_variant_graph_warehouse_example_validates() -> None:
    path = (
        Path(__file__).resolve().parents[2]
        / "schemas/examples/variant_graph_warehouse_industrial_west_v1.json"
    )
    data = load_json_file(path)
    validate_variant_graph(data)
    assert data["variant_graph_id"] == "variant_graph_warehouse_industrial_west_v1"
    assert len(data["variants"]) >= 3
