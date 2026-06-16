"""PG-MODULE-AUDIT-002 production gap batch tests."""

from __future__ import annotations

from pathlib import Path

import pytest

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.pg_module_audit_002 import (
    BATCH_ID,
    GAP_JOBS,
    write_gap_artifacts,
)


def test_gap_jobs_include_p3_stack_and_dock():
    p3 = [g for g in GAP_JOBS if g.priority == "P3"]
    ids = {g.module_id for g in p3}
    assert ids == {"stack_chimney_1u", "platform_dock_2u"}


def test_write_gap_artifacts_p3_specs():
    written = write_gap_artifacts(priorities=("P3",))
    assert len(written) == 2
    for row in written:
        assert Path(repo_root() / row["spec"]).is_file()
        assert Path(repo_root() / row["job"]).is_file()


def test_gap_jobs_include_p0_corner_fork():
    p0 = [g for g in GAP_JOBS if g.priority == "P0"]
    ids = {g.module_id for g in p0}
    assert "corner_L" in ids
    assert "door_warehouse" in ids
    corner = next(g for g in p0 if g.module_id == "corner_L")
    assert corner.job_id == "corner_L_industrial_west_production_run001"
    assert corner.job_id != "corner_L_production_run001"


def test_write_gap_artifacts_creates_specs_and_jobs():
    written = write_gap_artifacts(priorities=("P0",))
    assert len(written) == 2
    for row in written:
        spec = repo_root() / row["spec"]
        job = repo_root() / row["job"]
        assert spec.is_file(), row["spec"]
        assert job.is_file(), row["job"]


def test_promoted_p0_modules_exist_on_disk():
    for gap in GAP_JOBS:
        if gap.priority != "P0":
            continue
        glb = repo_root() / "assets" / "models" / "modules" / gap.job_id / "model.glb"
        if not glb.is_file():
            pytest.skip(f"promoted GLB missing: {gap.job_id}")


def test_manifest_lists_audit_batch():
    manifest = repo_root() / "tools/mcp/schemas/examples/batch_kit_industrial_west_production_001.manifest.json"
    if not manifest.is_file():
        pytest.skip("manifest not written yet")
    text = manifest.read_text(encoding="utf-8")
    assert BATCH_ID in text
    assert "PG-MODULE-AUDIT-002" in text
