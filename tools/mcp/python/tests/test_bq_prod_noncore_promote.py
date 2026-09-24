"""BQ-PROD-NONCORE-PROMOTE-001 — deferred-pack non-core lod0 promote."""

from __future__ import annotations

from rust_engine_mcp import bq_prod_noncore_promote
from rust_engine_mcp.paths import repo_root


def test_noncore_catalog_has_twelve() -> None:
    assert len(bq_prod_noncore_promote.PROMOTE_JOBS) == 12
    assert len(bq_prod_noncore_promote.job_ids()) == 12
    assert "door_gate_industrial_production_run001" in bq_prod_noncore_promote.job_ids()
    assert "prop_transformer_production_run002" in bq_prod_noncore_promote.job_ids()
    assert "wall_wood_2u_production_run001" in bq_prod_noncore_promote.job_ids()


def test_ensure_production_artifacts() -> None:
    written = bq_prod_noncore_promote.ensure_production_artifacts()
    assert len(written) == 12
    root = repo_root()
    for row in written:
        assert (root / row["spec"]).is_file()
        assert (root / row["job"]).is_file()


def test_run_prod_noncore_promote_no_rebake_green() -> None:
    body = bq_prod_noncore_promote.run_prod_noncore_promote(try_rebake=False, register=True)
    assert body["job_count"] == 12
    assert body["promoted_count"] == 12
    assert body["rules_check"]["passed"] is True
    assert body["exit_check"]["ok"] is True
    assert body["operator_pass"] is False
    assert body["green"] is True
    assert body["next_residual"] == bq_prod_noncore_promote.NEXT_RESIDUAL


def test_write_witness() -> None:
    body = bq_prod_noncore_promote.write_bq_prod_noncore_promote_witness(try_rebake=False)
    assert body["written"] == bq_prod_noncore_promote.WITNESS_REL
    path = repo_root() / bq_prod_noncore_promote.WITNESS_REL
    assert path.is_file()
    assert body["_agent_meta"]["task_id"] == "BQ-PROD-NONCORE-PROMOTE-001"
    assert body["_agent_meta"]["operator_pass"] is False
