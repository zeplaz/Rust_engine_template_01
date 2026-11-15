"""MATH-03 S5 — Halton scatter engine parity via shared fixture."""

from __future__ import annotations

import pytest

from rust_engine_mcp import landscape_grammar_presets
from rust_engine_mcp.halton_scatter import (
    halton_scatter_uv,
    load_math03_halton_fixture,
    math03_halton_fixture_path,
    scatter_jitter_in_cell,
)


def test_math03_halton_fixture_matches_python_halton() -> None:
    doc = load_math03_halton_fixture()
    assert doc.get("schema") == "math03_halton_uv_fixture_v1"
    assert doc.get("seed") == 1
    points = doc.get("points") or []
    assert len(points) == 16
    for row in points:
        i = int(row["site_index"])
        u, v = halton_scatter_uv(1, i)
        assert u == pytest.approx(float(row["u"]), rel=0, abs=1e-5)
        assert v == pytest.approx(float(row["v"]), rel=0, abs=1e-5)


def test_landscape_preset_scatter_preview_uses_halton() -> None:
    body = landscape_grammar_presets.landscape_preset_scatter_preview(
        3, 5, site_count=4, site_ix=2, site_iy=4
    )
    assert body.get("schema") == "landscape_preset_scatter_preview_v1"
    seed = body["halton_seed"]
    sites = body["uv_sites"]
    assert len(sites) == 4
    for row in sites:
        i = int(row["site_index"])
        u, v = halton_scatter_uv(seed, i)
        assert row["u"] == pytest.approx(u, rel=0, abs=1e-7)
        assert row["v"] == pytest.approx(v, rel=0, abs=1e-7)
    ju, jv = scatter_jitter_in_cell(3, 5, 2, 4)
    ex = body["example_cell_jitter"]
    assert ex["u"] == pytest.approx(ju, rel=0, abs=1e-7)
    assert ex["v"] == pytest.approx(jv, rel=0, abs=1e-7)


def test_math03_halton_fixture_on_disk() -> None:
    path = math03_halton_fixture_path()
    assert path.is_file(), f"missing shared fixture: {path}"