"""BQ-PROD-DEFER-PACKS-001 — deferred CORE promote + smoke retire."""

from __future__ import annotations

from rust_engine_mcp import bq_prod_defer_packs
from rust_engine_mcp.paths import repo_root


def test_defer_catalog_has_six() -> None:
    assert len(bq_prod_defer_packs.PROMOTE_JOBS) == 6
    assert len(bq_prod_defer_packs.job_ids()) == 6
    assert "wall_wood_1u_production_run001" in bq_prod_defer_packs.job_ids()
    assert "door_factory_production_run001" in bq_prod_defer_packs.job_ids()


def test_ensure_production_artifacts() -> None:
    written = bq_prod_defer_packs.ensure_production_artifacts()
    assert len(written) == 6
    root = repo_root()
    for row in written:
        assert (root / row["spec"]).is_file()
        assert (root / row["job"]).is_file()


def test_run_prod_defer_packs_no_rebake_green() -> None:
    body = bq_prod_defer_packs.run_prod_defer_packs(try_rebake=False, register=True, retire_smoke=True)
    assert body["job_count"] == 6
    assert body["promoted_count"] == 6
    assert body["rules_check"]["passed"] is True
    assert body["exit_check"]["deferred_core_production_only"] is True
    assert body["exit_check"]["smoke_index_count"] == 0
    # First run clears orphans (≥14); re-runs may clear 0 if already archived.
    archive = repo_root() / bq_prod_defer_packs.ARCHIVE_REL / "modules"
    archived = len([p for p in archive.iterdir()]) if archive.is_dir() else 0
    assert body["cleared_smoke_count"] + archived >= 14
    assert body["green"] is True
    assert body["next_residual"] == bq_prod_defer_packs.NEXT_RESIDUAL


def test_write_witness() -> None:
    body = bq_prod_defer_packs.write_bq_prod_defer_packs_witness(try_rebake=False)
    assert body["written"] == bq_prod_defer_packs.WITNESS_REL
    path = repo_root() / bq_prod_defer_packs.WITNESS_REL
    assert path.is_file()
    assert body["_agent_meta"]["task_id"] == "BQ-PROD-DEFER-PACKS-001"
