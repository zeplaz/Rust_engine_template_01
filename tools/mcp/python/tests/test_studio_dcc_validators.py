"""PLAN-REF-BOOKS-UPGRADE-v3 S5 — param_frame / massing / building_grammar validate_report."""

from __future__ import annotations

from pathlib import Path

from rust_engine_mcp.paths import repo_root, schemas_dir
from rust_engine_mcp.validators import run_validator
from rust_engine_mcp.validators.studio_dcc import (
    validate_building_grammar_path,
    validate_massing_preview_path,
    validate_param_frame_path,
)

PARAM_FRAME = schemas_dir() / "examples" / "param_frame_rail_warehouse_v1.json"
MASSING_PILOT = schemas_dir() / "examples" / "massing_preview_pilot_rail_warehouse_v1.json"
BUILDING_GRAMMAR = schemas_dir() / "examples" / "building_grammar_industrial_warehouse_v1.json"


def test_param_frame_validate_report_passes() -> None:
    report = validate_param_frame_path(PARAM_FRAME, compression_level=3)
    assert report.status == "passed"
    assert report.validator == "param_frame"
    d = report.to_dict()
    assert d["status"] == "passed"
    assert "frame_id" in report.summary or "rail" in report.summary.lower()


def test_massing_preview_validate_report_passes() -> None:
    report = validate_massing_preview_path(MASSING_PILOT, compression_level=3)
    assert report.status == "passed"
    assert report.validator == "massing_preview"


def test_building_grammar_validate_report_passes() -> None:
    report = validate_building_grammar_path(BUILDING_GRAMMAR, compression_level=3)
    assert report.status == "passed"
    assert report.validator == "building_grammar"


def test_run_validator_aliases_and_paths() -> None:
    rel_pf = "tools/mcp/schemas/examples/param_frame_rail_warehouse_v1.json"
    rel_mp = "tools/mcp/schemas/examples/massing_preview_pilot_rail_warehouse_v1.json"
    rel_bg = "tools/mcp/schemas/examples/building_grammar_industrial_warehouse_v1.json"
    assert run_validator("param_frame", rel_pf).status == "passed"
    assert run_validator("massing_preview", rel_mp).status == "passed"
    assert run_validator("massing", rel_mp).status == "passed"
    assert run_validator("building_grammar", rel_bg).status == "passed"


def test_param_frame_schema_fail(tmp_path: Path) -> None:
    bad = tmp_path / "bad_frame.json"
    bad.write_text("{}", encoding="utf-8")
    report = validate_param_frame_path(bad)
    assert report.status == "failed"
    assert report.error_count >= 1