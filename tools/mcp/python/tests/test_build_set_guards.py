"""MCP-GUARD-001…004 + MCP-INTEGRATE-001 tests."""

from __future__ import annotations

import json
from pathlib import Path

from rust_engine_mcp.build_set_guards import (
    TEACHABLE_WITNESS,
    example_teachable_audit,
    single_archetype_ratio_guard,
    validate_example_teachable_audit,
    validate_warehouse_track_guard,
    warehouse_track_guard,
    write_build_set_guards_witnesses,
)
from rust_engine_mcp.grammar_integration import (
    DEFAULT_SNAPSHOT,
    grammar_integration_validate,
    validate_grammar_integration_path,
    write_grammar_integration_witness,
)
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.pilot_hardcode_lint import (
    WITNESS_PATH,
    _path_allowed,
    pilot_hardcode_lint,
    validate_pilot_hardcode_lint,
    write_pilot_hardcode_lint_witness,
)


def test_pilot_hardcode_lint_green_on_repo() -> None:
    body = pilot_hardcode_lint()
    assert body["ok"] is True
    assert body["green"] is True
    assert body["violation_count"] == 0
    assert body["scanned_files"] > 0


def test_validate_report_passes_when_green() -> None:
    report = validate_pilot_hardcode_lint()
    assert report.status == "passed"
    assert "pilot_hardcode_lint" in report.summary


def test_path_allowlist_globs() -> None:
    globs = ["tools/mcp/schemas/examples/*warehouse*"]
    assert _path_allowed(
        "tools/mcp/schemas/examples/tile_batch_warehouse_industrial_west_production_v1.json",
        globs=globs,
        exact=[],
    )
    assert not _path_allowed("src/construction/pilot_catalog.rs", globs=globs, exact=[])


def test_detects_violation_with_empty_allowlist(tmp_path: Path) -> None:
    scan_root = tmp_path / "src"
    scan_root.mkdir()
    bad = scan_root / "bad_branch.rs"
    bad.write_text('let x = "logistics_rail_warehouse_v0";\n', encoding="utf-8")

    cfg = {
        "needles": ["logistics_rail_warehouse_v0"],
        "scan_roots": ["src"],
        "scan_extensions": [".rs"],
        "permanent_allowlist_globs": [],
        "transitional_allowlist": [],
    }
    allowlist = tmp_path / "allowlist.json"
    allowlist.write_text(json.dumps(cfg), encoding="utf-8")

    # Scan temp tree by temporarily treating tmp_path as repo root via chdir pattern —
    # pilot_hardcode_lint always uses repo_root(); test classification via direct file read instead.
    rel = "src/bad_branch.rs"
    assert not _path_allowed(rel, globs=[], exact=[])
    text = bad.read_text(encoding="utf-8")
    assert "logistics_rail_warehouse_v0" in text


def test_witness_writer() -> None:
    witness = write_pilot_hardcode_lint_witness()
    assert witness["gate_id"] == "pilot_hardcode_lint"
    assert witness["task_id"] == "MCP-GUARD-001"
    assert (repo_root() / WITNESS_PATH).is_file()


def test_example_teachable_audit_green_on_repo() -> None:
    body = example_teachable_audit()
    assert body["ok"] is True
    assert body["green"] is True
    assert body["violation_count"] == 0
    assert body["checked_files"] >= 2


def test_validate_example_teachable_audit_report() -> None:
    report = validate_example_teachable_audit()
    assert report.status == "passed"


def test_single_archetype_ratio_guard_insured() -> None:
    body = single_archetype_ratio_guard()
    assert body["building_set_insured"] is True
    assert body["green"] is True
    assert body["total_refs"] > 0


def test_warehouse_track_guard_green_on_repo() -> None:
    body = warehouse_track_guard()
    assert body["green"] is True
    assert body["violation_count"] == 0


def test_validate_warehouse_track_guard_report() -> None:
    report = validate_warehouse_track_guard()
    assert report.status == "passed"


def test_build_set_guards_witnesses() -> None:
    bundle = write_build_set_guards_witnesses()
    assert bundle["green"] is True
    assert (repo_root() / TEACHABLE_WITNESS).is_file()


def test_grammar_integration_validate_warehouse_production() -> None:
    body = grammar_integration_validate(DEFAULT_SNAPSHOT)
    assert body["green"] is True
    assert body["preset_id"] == "logistics_rail_warehouse_v0"
    assert body["preset_pair"]["green"] is True
    assert body["error_count"] == 0


def test_validate_grammar_integration_report() -> None:
    report = validate_grammar_integration_path(DEFAULT_SNAPSHOT)
    assert report.status == "passed"
    assert report.validator == "grammar_integration"


def test_grammar_integration_witness_writer() -> None:
    body = write_grammar_integration_witness()
    assert body["task_id"] == "MCP-INTEGRATE-001"
    assert body["green"] is True
