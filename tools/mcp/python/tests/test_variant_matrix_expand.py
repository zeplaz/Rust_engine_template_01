"""PT-3 — variant_matrix_expand + sim_tags."""

from __future__ import annotations

from pathlib import Path

import pytest

from rust_engine_mcp.variant_matrix_expand import (
    SIM_TAGS,
    expanded_variant_keys,
    load_variant_matrix,
    variant_matrix_expand,
)
from rust_engine_mcp.paths import repo_root


MATRIX = repo_root() / "debug_runs/art_pipeline/variant_matrix_rowhouse_v1.yaml"


def test_load_variant_matrix_rowhouse():
    matrix = load_variant_matrix(MATRIX)
    assert matrix["archetype"]
    assert matrix["variant_keys"]
    assert "clean_day" in matrix["variant_keys"]


def test_expanded_keys_includes_fire_row():
    matrix = load_variant_matrix(MATRIX)
    keys = expanded_variant_keys(matrix, include_fire_row=True, minimum_only=False)
    assert len(keys) >= 6
    assert "clean_day" in keys
    assert "clean_night_on" in keys
    assert "burning_00" in keys
    assert "burning_07" in keys


def test_sim_tags_on_burning_frame():
    assert "sim_fire" in SIM_TAGS["burning_00"]
    assert "sim_fire_frame_0" in SIM_TAGS["burning_00"]


def test_variant_matrix_expand_returns_sim_tags():
    result = variant_matrix_expand(MATRIX, write_batch=False)
    assert result["ok"]
    assert result["variant_count"] >= 6
    tags = result["sim_tags_by_key"]
    assert tags["clean_day"]
    assert tags["burning_00"]


def test_ship_minimum_only():
    matrix = load_variant_matrix(MATRIX)
    keys = expanded_variant_keys(matrix, minimum_only=True)
    assert "clean_day" in keys
    assert "burning_00" in keys


@pytest.mark.parametrize(
    "matrix_name",
    [
        "variant_matrix_rowhouse_v1.yaml",
        "variant_matrix_warehouse_v1.yaml",
        "variant_matrix_shopfront_v1.yaml",
        "variant_matrix_bunker_v1.yaml",
    ],
)
def test_all_pilot_matrices_expand(matrix_name: str):
    path = repo_root() / "debug_runs/art_pipeline" / matrix_name
    result = variant_matrix_expand(path, write_batch=False)
    assert result["variant_count"] >= 6
