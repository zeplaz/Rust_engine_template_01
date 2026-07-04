"""Tests for APSR-A2-P1-001 — assembly_panel ≤400 LOC split."""

from __future__ import annotations

from rust_engine_mcp.apsr_a2_p1 import MAX_PANEL_LOC, PANEL_REL, write_apsr_a2_p1_witness
from rust_engine_mcp.paths import repo_root


def test_assembly_panel_loc_cap() -> None:
    loc = len((repo_root() / PANEL_REL).read_text(encoding="utf-8").splitlines())
    assert loc <= MAX_PANEL_LOC


def test_apsr_a2_p1_witness_green() -> None:
    body = write_apsr_a2_p1_witness()
    assert body["task_id"] == "APSR-A2-P1-001"
    assert body["green"] is True
