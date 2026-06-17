"""MCP-WIT-040 — witness_honesty rule engine + fixture coverage."""

from __future__ import annotations

import json

import pytest

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators import run_validator
from rust_engine_mcp.validators.witness_honesty import (
    FIXTURES_REL,
    _fixture_pairs,
    evaluate_witness_honesty_rules,
    load_witness_integrity_catalog,
    refresh_mcp_witness_honesty_validator_witness,
    validate_witness_honesty_path,
    validate_witness_honesty_scan,
    validate_witness_integrity_catalog,
)

_SCAN_ONLY_RULES = frozenset({"WIT-QUEUE-CONTRADICTION", "WIT-SNAG-DONE"})


def test_catalog_schema_and_rule_count() -> None:
    validate_witness_integrity_catalog()
    catalog = load_witness_integrity_catalog()
    assert len(catalog.get("rules") or []) >= 12


@pytest.mark.parametrize(
    ("fixture", "rule_id"),
    [
        ("good_minimal_live.json", None),
        ("landscape_grammar_lg4_fixture_bad_live.json", "WIT-GREEN-TINT-ZERO"),
        ("bad_operator_lib_fixture_live.json", "WIT-OPERATOR-LIB-FIXTURE"),
        ("bad_art_dishonest_live.json", "WIT-ART-DISHONEST"),
        ("bad_exit_predicate_live.json", "WIT-EXIT-PREDICATE"),
        ("bad_tiny_png_pilot_live.json", "WIT-TINY-PNG-PILOT"),
        ("bad_missing_envelope_live.json", "WIT-MISSING-ENVELOPE"),
    ],
)
def test_fixture_expectations(fixture: str, rule_id: str | None) -> None:
    path = repo_root() / FIXTURES_REL / fixture
    doc = json.loads(path.read_text(encoding="utf-8"))
    meta = doc.get("_fixture") or {}
    expect_pass = meta.get("expect") == "pass"
    report = validate_witness_honesty_path(path, compression_level=3)
    if expect_pass:
        assert report.status == "passed", report.summary
    else:
        assert report.status != "passed", report.summary
        if rule_id:
            assert any(e.symbol == rule_id for e in report.errors), [e.symbol for e in report.errors]


def test_all_disk_fixtures_match_meta() -> None:
    pairs = _fixture_pairs(repo_root())
    assert len(pairs) >= 7
    for rel, rule_id, expect_pass in pairs:
        report = validate_witness_honesty_path(repo_root() / rel, compression_level=3)
        ok = (report.status == "passed") if expect_pass else (report.status != "passed")
        assert ok, f"{rel} expect_pass={expect_pass} status={report.status} rule={rule_id}"


def test_catalog_witness_rules_have_pytest_coverage() -> None:
    catalog = load_witness_integrity_catalog()
    covered = {
        "WIT-GREEN-TINT-ZERO",
        "WIT-OPERATOR-LIB-FIXTURE",
        "WIT-ROLLUP-CHILD-ONLY",
        "WIT-PHASE-CLOSE-WITHOUT-SUB",
        "WIT-GATE-DRIFT-G4",
        "WIT-TINY-PNG-PILOT",
        "WIT-ART-DISHONEST",
        "WIT-ENV-BOOTSTRAP-ONLY",
        "WIT-MISSING-ENVELOPE",
        "WIT-EXIT-PREDICATE",
        "WIT-QUEUE-CONTRADICTION",
        "WIT-SNAG-DONE",
    }
    for rule in catalog.get("rules") or []:
        rid = str(rule.get("rule_id") or "")
        if rid in _SCAN_ONLY_RULES:
            continue
        assert rid in covered, f"missing pytest coverage for {rid}"


def test_env_bootstrap_only_warning() -> None:
    catalog = load_witness_integrity_catalog()
    root = repo_root()
    data = {
        "_agent_meta": {
            "schema": "witness_honesty_fixture_v1",
            "agent_commands": ["cargo test -p proc_A_dine01 --lib stage5 -- --lib"],
        },
        "green": True,
        "live_sim_required": True,
    }
    rel = "debug_runs/veg_runtime_proof_live.json"
    issues = evaluate_witness_honesty_rules(data, witness_rel=rel, catalog=catalog, root=root)
    assert any(i.symbol == "WIT-ENV-BOOTSTRAP-ONLY" and i.severity == "warning" for i in issues)


def test_gate_drift_g4(tmp_path) -> None:
    root = tmp_path
    keyframe = root / "debug_runs/art_pipeline/pilot_production_keyframe_g4_live.json"
    keyframe.parent.mkdir(parents=True, exist_ok=True)
    keyframe.write_text(
        json.dumps({"gates": {"g4_8_proceed_ship": "pass"}, "_agent_meta": {"schema": "x"}}),
        encoding="utf-8",
    )
    tile_rel = "debug_runs/art_pipeline/tile_pilot_production_v1_live.json"
    tile_path = root / tile_rel
    tile_path.parent.mkdir(parents=True, exist_ok=True)
    data = {
        "_agent_meta": {"schema": "witness_honesty_fixture_v1"},
        "batch_id": "tile_pilot_production_v1",
        "gates": {"G4": "planned"},
        "green": False,
    }
    tile_path.write_text(json.dumps(data), encoding="utf-8")
    catalog = load_witness_integrity_catalog()
    issues = evaluate_witness_honesty_rules(data, witness_rel=tile_rel, catalog=catalog, root=root)
    assert any(i.symbol == "WIT-GATE-DRIFT-G4" for i in issues)


def test_rollup_child_only(tmp_path) -> None:
    root = tmp_path
    child_rel = "debug_runs/landscape_grammar_lg4_preview_live.json"
    child = root / child_rel
    child.parent.mkdir(parents=True, exist_ok=True)
    child.write_text(
        json.dumps(
            {
                "_agent_meta": {"schema": "witness_honesty_fixture_v1"},
                "green": True,
                "topology_tint_visible_chunks": 0,
            }
        ),
        encoding="utf-8",
    )
    parent_rel = "debug_runs/vegetation_program_close_live.json"
    parent_data = {
        "_agent_meta": {"schema": "witness_honesty_fixture_v1"},
        "green": True,
        "all_green": True,
    }
    catalog = load_witness_integrity_catalog()
    issues = evaluate_witness_honesty_rules(
        parent_data,
        witness_rel=parent_rel,
        catalog=catalog,
        root=root,
    )
    assert any(i.symbol == "WIT-ROLLUP-CHILD-ONLY" for i in issues)


def test_phase_close_without_sub(tmp_path) -> None:
    root = tmp_path
    child_rel = "debug_runs/landscape_grammar_lg4_preview_live.json"
    child = root / child_rel
    child.parent.mkdir(parents=True, exist_ok=True)
    child.write_text(
        json.dumps(
            {
                "_agent_meta": {"schema": "witness_honesty_fixture_v1"},
                "green": True,
                "topology_tint_visible_chunks": 0,
            }
        ),
        encoding="utf-8",
    )
    parent_rel = "debug_runs/vegetation_program_close_live.json"
    parent_data = {
        "_agent_meta": {"schema": "witness_honesty_fixture_v1"},
        "green": True,
        "phase_a_green": True,
        "phase_b_green": True,
    }
    catalog = load_witness_integrity_catalog()
    issues = evaluate_witness_honesty_rules(
        parent_data,
        witness_rel=parent_rel,
        catalog=catalog,
        root=root,
    )
    assert any(i.symbol == "WIT-PHASE-CLOSE-WITHOUT-SUB" for i in issues)


def test_real_lg4_preview_fails_green_tint_zero() -> None:
    path = repo_root() / "debug_runs/landscape_grammar_lg4_preview_live.json"
    if not path.is_file():
        pytest.skip("lg4 preview witness missing")
    report = validate_witness_honesty_path(path, compression_level=3)
    assert report.status == "failed"
    assert any(i.symbol == "WIT-GREEN-TINT-ZERO" for i in report.errors)


def test_scan_debug_runs_structured_report() -> None:
    report = validate_witness_honesty_scan("debug_runs", compression_level=3)
    assert report.validator == "test"
    assert report.compression_level == 3
    assert "scanned=" in report.summary
    assert report.status in ("failed", "warning", "passed")


def test_cli_run_validator_witness_honesty() -> None:
    path = repo_root() / FIXTURES_REL / "good_minimal_live.json"
    rel = path.relative_to(repo_root()).as_posix()
    report = run_validator("witness_honesty", rel, compression_level=4)
    assert report.status == "passed"
    assert report.compression_level == 4


def test_mcp_witness_honesty_validator_witness_green() -> None:
    body = refresh_mcp_witness_honesty_validator_witness()
    assert body.get("green") is True
    assert len(body.get("fixture_results") or []) >= 7
    witness_path = repo_root() / "debug_runs/mcp_witness_honesty_validator_live.json"
    assert witness_path.is_file()
