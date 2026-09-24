"""BQ-PROD-PROMOTE-BATCH-001 — production rebake/promote of focus lod0 slots."""

from __future__ import annotations

from rust_engine_mcp import bq_prod_promote_batch
from rust_engine_mcp.paths import repo_root


def test_promote_catalog_has_six() -> None:
    assert len(bq_prod_promote_batch.PROMOTE_JOBS) == 6
    assert len(bq_prod_promote_batch.job_ids()) == 6
    assert "win_double_1u_production_run001" in bq_prod_promote_batch.job_ids()


def test_ensure_production_artifacts() -> None:
    written = bq_prod_promote_batch.ensure_production_artifacts()
    assert len(written) == 6
    root = repo_root()
    for row in written:
        assert (root / row["spec"]).is_file()
        assert (root / row["job"]).is_file()


def test_run_prod_promote_batch_no_rebake_green() -> None:
    """Donor-seed path (no Blender) must still promote + clear focus lod0 targets."""
    body = bq_prod_promote_batch.run_prod_promote_batch(try_rebake=False, register=True)
    assert body["job_count"] == 6
    assert body["promoted_count"] == 6
    assert body["rules_check"]["passed"] is True
    assert body["exit_check"]["industrial_west_standard_production_only"] is True
    assert body["exit_check"]["victorian_standard_production_only"] is True
    assert not body["exit_check"]["promoted_targets_still_lod0"]
    assert body["green"] is True
    assert body["next_residual"] == bq_prod_promote_batch.NEXT_RESIDUAL


def test_write_witness() -> None:
    body = bq_prod_promote_batch.write_bq_prod_promote_batch_witness(try_rebake=False)
    assert body["written"] == bq_prod_promote_batch.WITNESS_REL
    path = repo_root() / bq_prod_promote_batch.WITNESS_REL
    assert path.is_file()
    assert body["_agent_meta"]["task_id"] == "BQ-PROD-PROMOTE-BATCH-001"
