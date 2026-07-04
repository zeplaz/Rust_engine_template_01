"""Tests for APSR-A1-S3-001 — LaneChanged shell cleanup."""

from __future__ import annotations

from rust_engine_mcp.apsr_a1_s3 import (
    APP_REL,
    MAX_APP_LOC,
    WIRING_REL,
    _app_line_count,
    _shell_sync_from_state_calls,
    write_apsr_a1_s3_witness,
)
from rust_engine_mcp.paths import repo_root


def test_app_py_under_loc_cap() -> None:
    loc = _app_line_count(repo_root() / APP_REL)
    assert loc < MAX_APP_LOC


def test_app_py_has_no_sync_from_state_calls() -> None:
    hits = _shell_sync_from_state_calls(repo_root() / APP_REL)
    assert hits == []


def test_shell_wiring_module_exists() -> None:
    assert (repo_root() / WIRING_REL).is_file()


def test_apsr_a1_s3_witness_green() -> None:
    body = write_apsr_a1_s3_witness()
    assert body["task_id"] == "APSR-A1-S3-001"
    assert body["green"] is True
    assert body["shell_sync_from_state_lines"] == []
