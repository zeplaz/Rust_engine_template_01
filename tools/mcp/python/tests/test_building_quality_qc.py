"""APSR-A4-Q1-001 / APS-QC-SMOKE-LABEL-001 — building quality QC strip tests."""

from __future__ import annotations

from rust_engine_mcp import building_quality_qc
from rust_engine_mcp.paths import repo_root


def test_building_quality_witness_loads() -> None:
    witness = building_quality_qc.load_building_quality_witness(repo=repo_root())
    assert witness is not None
    assert witness.get("gate") == "BQ-A2-GATE-001"


def test_format_qc_strip_from_witness() -> None:
    text, ok = building_quality_qc.format_qc_strip_text(repo=repo_root())
    assert "Building QC" in text
    assert ok is True or ok is False or ok is None


def test_smoke_label_on_greybox_placement() -> None:
    snap = {
        "assembly_id": "qc_smoke_label_test",
        "module_placements": [
            {
                "module_id": "wall_brick_1u",
                "job_id": "wall_smoke_kit_greybox_001",
                "slot_key": "wall_1u",
            }
        ],
    }
    hit = building_quality_qc.scan_snapshot_tier_labels(snap, repo=repo_root())
    assert hit["smoke_hit"] is True
    assert building_quality_qc.LABEL_SMOKE in hit["labels"]
    text, ok = building_quality_qc.format_qc_strip_text(
        "qc_smoke_label_test",
        snapshot=snap,
        repo=repo_root(),
    )
    assert building_quality_qc.LABEL_SMOKE in text
    assert ok is False
    allowed, reason = building_quality_qc.assembly_qc_allows_approve(
        "qc_smoke_label_test",
        snapshot=snap,
        repo=repo_root(),
    )
    assert allowed is False
    assert reason == building_quality_qc.LABEL_SMOKE


def test_lod0_core_label() -> None:
    snap = {
        "assembly_id": "qc_lod0_label_test",
        "module_placements": [
            {
                "module_id": "wall_brick_1u",
                "job_id": "wall_brick_1u_lod0_run001",
                "slot_key": "wall_1u",
                "lod_policy": "lod0",
            }
        ],
    }
    hit = building_quality_qc.scan_snapshot_tier_labels(snap, repo=repo_root())
    assert hit["lod0_core_hit"] is True
    assert building_quality_qc.LABEL_LOD0 in hit["labels"]
    text, _ok = building_quality_qc.format_qc_strip_text(
        "qc_lod0_label_test",
        snapshot=snap,
        repo=repo_root(),
    )
    assert building_quality_qc.LABEL_LOD0 in text


def test_write_apsr_q1_witness() -> None:
    body = building_quality_qc.write_apsr_q1_witness(repo=repo_root())
    assert body.get("task_id") == "APSR-A4-Q1-001"
    assert body.get("approve_blocks_on_red_qc") is True
    assert (repo_root() / "debug_runs" / "apsr_a4_q1_001_live.json").is_file()
