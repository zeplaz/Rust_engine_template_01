"""BQ-SMOKE-AUDIT-001 — smoke/lod0 tier inventory witness tests."""

from __future__ import annotations

from rust_engine_mcp import bq_smoke_tier_audit
from rust_engine_mcp.paths import repo_root


def test_count_index_tiers_shape() -> None:
    counts = bq_smoke_tier_audit.count_index_tiers()
    assert counts["total_entries"] > 0
    assert "production" in counts
    assert "lod0" in counts
    assert "smoke" in counts
    assert counts["production"] >= 1
    assert counts["lod0"] >= 1
    # Smoke may be 0 after BQ-PROD-DEFER-PACKS-001 retire.
    assert counts["smoke"] >= 0


def test_audit_smoke_tier_exit_fields() -> None:
    body = bq_smoke_tier_audit.audit_smoke_tier()
    assert body["gate"] == "BQ-SMOKE-AUDIT-001"
    assert body["counts"]["production"] >= 1
    assert body["counts"]["lod0"] >= 1
    assert isinstance(body["selectable_smoke_ids"], list)
    assert isinstance(body["per_style_pack"], list)
    assert body["per_style_pack"], "expected style packs"
    for pack in body["per_style_pack"]:
        assert "selectable_smoke_ids" in pack
        assert "selectable_lod0_ids" in pack
    lists = body["promote_block_lists"]
    assert "promote" in lists
    assert "block" in lists
    if body["any_style_pack_picks_smoke_or_lod0_standard_slot"]:
        assert body["green"] is False
        assert body["verdict"] == "FAIL"
    else:
        # Cleared CORE + smoke index → audit green; residual = spine render.
        assert body["green"] is True
        assert body["counts"]["smoke"] == 0
    assert body["next_residual"] == bq_smoke_tier_audit.NEXT_SLICE


def test_write_bq_smoke_tier_audit_witness() -> None:
    body = bq_smoke_tier_audit.write_bq_smoke_tier_audit_witness()
    assert body["written"] == bq_smoke_tier_audit.WITNESS_REL
    path = repo_root() / bq_smoke_tier_audit.WITNESS_REL
    assert path.is_file()
    assert body["_agent_meta"]["task_id"] == "BQ-SMOKE-AUDIT-001"
    assert body["next_residual"] == bq_smoke_tier_audit.NEXT_SLICE
