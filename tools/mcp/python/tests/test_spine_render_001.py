"""SPINE-RENDER-001 — blender-worker render_variant contract tests."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.spine_render_contract import (
    EXAMPLE_REL,
    classify_render_job,
    contract_errors,
    run_spine_render_audit,
    validate_render_variant_job,
    validate_render_variant_path,
    write_spine_render_001_witness,
)


def test_example_job_validates() -> None:
    path = repo_root() / EXAMPLE_REL
    data = json.loads(path.read_text(encoding="utf-8"))
    validate_render_variant_job(data)
    assert classify_render_job(data)["ok"] is True


def test_ship_ortho_rejected() -> None:
    job = {
        "schema_version": 1,
        "job": "render_variant",
        "asset": "x",
        "variant": "clean",
        "facing": 0,
        "seed": 1,
        "role": "smoke",
        "ship": True,
        "render": {"method": "blender_orthographic_iso", "bake_source": "smoke_ortho_headless"},
    }
    errs = contract_errors(job)
    assert errs
    assert any("ship" in e.lower() or "ortho" in e.lower() for e in errs)
    with pytest.raises(ValueError):
        validate_render_variant_job(job)


def test_validate_report_path_passes_example() -> None:
    report = validate_render_variant_path(EXAMPLE_REL, compression_level=4)
    assert report.status == "passed"


def test_audit_and_witness_green(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    # Write witness into real debug_runs (audit needs repo schemas/examples).
    body = run_spine_render_audit()
    assert body["green"] is True
    assert body["checks"]["ship_ortho_job_rejected"]["ok"] is True
    out = write_spine_render_001_witness()
    assert out.get("green") is True
    assert (repo_root() / "debug_runs/spine_render_001_live.json").is_file()
