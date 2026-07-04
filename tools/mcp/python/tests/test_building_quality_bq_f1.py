"""Tests for BQ-F1-BAKE-001 — roof flush + wall sill rebake witness."""

from __future__ import annotations

from pathlib import Path

from rust_engine_mcp import building_quality_bq_f1
from rust_engine_mcp.paths import repo_root


def test_audit_bpy_sources_green() -> None:
    audit = building_quality_bq_f1.audit_bpy_sources()
    assert audit["green"] is True
    assert audit["issues"] == []


def test_discover_bq_f1_job_ids_nonempty() -> None:
    jobs = building_quality_bq_f1.discover_bq_f1_job_ids()
    assert len(jobs) >= 10
    assert any(j.startswith("roof_") for j in jobs)
    assert any(j.startswith("wall_") for j in jobs)


def test_roof_flat_glb_bounds_helper() -> None:
    glb = repo_root() / "assets/models/modules/roof_flat_2u_run001/model.glb"
    if not glb.is_file():
        return
    bounds = building_quality_bq_f1.glb_position_bounds(glb)
    assert bounds is not None
    assert "min" in bounds


def test_write_bq_f1_witness_shape() -> None:
    body = building_quality_bq_f1.write_bq_f1_witness()
    assert body["task_id"] == "BQ-F1-BAKE-001"
    assert "green" in body
    assert body.get("bpy_source_audit_green") is True
    assert (repo_root() / "debug_runs/building_quality_bq_f1_live.json").is_file()


def test_bq_f1_status_rows_cover_operations() -> None:
    status = building_quality_bq_f1.bq_f1_status()
    ops = {r["operation"] for r in status["rows"]}
    assert "module_roof" in ops
    assert "module_wall" in ops
