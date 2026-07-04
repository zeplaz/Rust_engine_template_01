"""Tests for APSR-A2-P2/P3."""

from __future__ import annotations

from rust_engine_mcp.apsr_a2_p2 import write_apsr_a2_p2_witness
from rust_engine_mcp.apsr_a2_p3 import write_apsr_a2_p3_witness
from rust_engine_mcp.paths import repo_root


def test_preview_state_display_module_exists() -> None:
    assert (repo_root() / "tools/mcp/art_pipeline_suite/preview_state_display.py").is_file()


def test_apsr_a2_p2_witness_green() -> None:
    body = write_apsr_a2_p2_witness()
    assert body["green"] is True


def test_apsr_a2_p3_witness_green() -> None:
    body = write_apsr_a2_p3_witness()
    assert body["green"] is True
