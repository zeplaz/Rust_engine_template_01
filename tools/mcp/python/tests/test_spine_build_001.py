"""SPINE-BUILD-001 — build dependency graph contract tests."""

from __future__ import annotations

import json

import pytest

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.spine_build_graph import (
    EXAMPLE_REL,
    classify_build_graph,
    graph_errors,
    run_spine_build_audit,
    topological_order,
    validate_build_graph,
    write_spine_build_001_witness,
)
from rust_engine_mcp.validators import run_validator


def test_example_graph_validates() -> None:
    path = repo_root() / EXAMPLE_REL
    data = json.loads(path.read_text(encoding="utf-8"))
    validate_build_graph(data)
    classified = classify_build_graph(data)
    assert classified["ok"] is True
    assert classified["topo_order"][0] == "build_assembly"
    assert classified["topo_order"][-1] == "register_atlas"


def test_bad_depends_on_rejected() -> None:
    path = repo_root() / EXAMPLE_REL
    data = json.loads(path.read_text(encoding="utf-8"))
    for node in data["nodes"]:
        if node["node_id"] == "pack_atlas":
            node["depends_on"] = ["build_assembly"]
            break
    errs = graph_errors(data)
    assert errs
    with pytest.raises(ValueError):
        validate_build_graph(data)


def test_topo_order_parallel_variants_blend() -> None:
    path = repo_root() / EXAMPLE_REL
    data = json.loads(path.read_text(encoding="utf-8"))
    order = topological_order(data)
    assert order.index("build_assembly") < order.index("build_variants")
    assert order.index("build_assembly") < order.index("build_blend")
    assert order.index("build_variants") < order.index("render_frames")
    assert order.index("build_blend") < order.index("render_frames")


def test_validate_report_build_graph() -> None:
    report = run_validator("build_graph", EXAMPLE_REL, compression_level=4)
    assert report.status == "passed"


def test_audit_and_witness_green() -> None:
    body = run_spine_build_audit()
    assert body["green"] is True
    assert body["checks"]["cycle_rejected"]["ok"] is True
    out = write_spine_build_001_witness()
    assert out.get("green") is True
    assert (repo_root() / "debug_runs/spine_build_001_live.json").is_file()
