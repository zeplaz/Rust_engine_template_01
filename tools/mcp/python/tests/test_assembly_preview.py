"""APS-PREVIEW-002 — assembly preview CLI + placement resolution."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from rust_engine_mcp import assembly_preview
from rust_engine_mcp.paths import repo_root


EXAMPLE_SNAPSHOT = (
    repo_root()
    / "tools"
    / "mcp"
    / "schemas"
    / "examples"
    / "assembly_snapshot_warehouse_industrial_west_production_v1.json"
)


def test_collect_preview_placements_resolves_warehouse_example():
    snap = json.loads(EXAMPLE_SNAPSHOT.read_text(encoding="utf-8"))
    placements, missing = assembly_preview.collect_preview_placements(snap)
    assert len(placements) >= 2
    assert not missing
    profiles = {p.material_profile for p in placements if p.material_profile}
    assert "steel_panel_01" in profiles or "brick_red_01" in profiles


def test_write_preview_job_matches_contract():
    job = assembly_preview.write_preview_job(EXAMPLE_SNAPSHOT)
    data = json.loads(job.read_text(encoding="utf-8"))
    assert data["schema_version"] == 1
    assert data["operation"] == "preview_assembly"
    assert data["camera"]["preset"] == "iso_ne"
    assert data["output"]["width"] == 512


def test_preview_assembly_cli_witness_no_browser():
    if not EXAMPLE_SNAPSHOT.is_file():
        pytest.skip("warehouse example snapshot missing")
    result = assembly_preview.preview_assembly(
        EXAMPLE_SNAPSHOT,
        open_browser=False,
        try_bevy=False,
    )
    assert result["gate_id"] == "APS-PREVIEW-002"
    assert result["modules_loaded"] >= 2
    assert result["mode"] == "browser_threejs"
    assert result.get("preview_job")
    out = assembly_preview.write_aps_preview_002_witness(result)
    assert out.is_file()
    body = json.loads(out.read_text(encoding="utf-8"))
    assert body.get("green") is True
    assert body.get("material_profiles_sample")


def test_bevy_preview_worker_smoke_optional():
    """APS-PREVIEW-004 — run worker when binary exists (GPU required)."""
    root = repo_root()
    exe = root / "target/debug/bevy_preview_worker.exe"
    if not exe.is_file():
        exe = root / "target/debug/bevy_preview_worker"
    if not exe.is_file():
        pytest.skip("bevy_preview_worker not built")
    if not EXAMPLE_SNAPSHOT.is_file():
        pytest.skip("warehouse example snapshot missing")
    result = assembly_preview.preview_assembly(EXAMPLE_SNAPSHOT, open_browser=False, try_bevy=True)
    if result.get("mode") != "bevy_worker":
        pytest.skip(f"bevy worker unavailable: {result.get('bevy_status')}")
    assert result.get("png")
    out = assembly_preview.write_preview_worker_smoke_witness(result)
    body = json.loads(out.read_text(encoding="utf-8"))
    assert body.get("green") is True


def test_assembly_preview_panel_module_exists():
    root = repo_root()
    assert (root / "tools/mcp/art_pipeline_suite/assembly_preview_panel.py").is_file()
    assert (root / "tools/mcp/python/rust_engine_mcp/assembly_preview.py").is_file()
