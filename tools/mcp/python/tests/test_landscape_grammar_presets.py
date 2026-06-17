"""MCP-LG-VALID-PRESET-001 — batch landscape grammar preset validation."""

from __future__ import annotations

from rust_engine_mcp import landscape_grammar_presets
from rust_engine_mcp.validators import run_validator


def test_landscape_grammar_presets_batch_green() -> None:
    body = landscape_grammar_presets.landscape_grammar_presets_batch()
    assert body.get("green") is True
    validation = body.get("preset_validation") or {}
    assert validation.get("failed") == 0
    assert validation.get("total") == 10


def test_validate_report_landscape_grammar_presets() -> None:
    report = run_validator("landscape_grammar_presets", compression_level=3)
    assert report.status == "passed"


def test_write_landscape_grammar_presets_witness() -> None:
    body = landscape_grammar_presets.write_landscape_grammar_presets_witness()
    assert body.get("gate") == "MCP-LG-VALID-PRESET-001"
    assert body.get("green") is True
    assert body.get("written") == landscape_grammar_presets.BATCH_WITNESS_REL


def test_refresh_sign_witness_from_batch() -> None:
    body = landscape_grammar_presets.refresh_mcp_landscape_grammar_sign_witness()
    assert body.get("signed") is True
    assert body.get("green") is True
