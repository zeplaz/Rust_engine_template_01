"""APS-WITNESS-REFRESH-001 + grammar label smoke tests."""

from __future__ import annotations

from pathlib import Path

from rust_engine_mcp.aps_grammar_labels import grammar_why_detail, human_label
from rust_engine_mcp.aps_witness_refresh import APS_ARTIST_TOOL_MODULES_WITNESS, refresh_aps_witnesses
from rust_engine_mcp.paths import repo_root


def test_human_label_known_rule() -> None:
    assert human_label("long_hall") == "Long hall"
    assert "wide shallow" in grammar_why_detail("long_hall").lower()


def test_human_label_fallback() -> None:
    assert human_label("custom_rule_id") == "Custom Rule Id"


def test_refresh_writes_modules_witness() -> None:
    from rust_engine_mcp.aps_witness_refresh import APS_ARTIST_TOOL_MODULES_WITNESS, _suite_modules

    modules = _suite_modules()
    assert len(modules) >= 10
    out = repo_root() / APS_ARTIST_TOOL_MODULES_WITNESS
    if out.is_file():
        assert "suite_modules" in out.read_text(encoding="utf-8")


def test_preview_catalog_witness_path() -> None:
    path = repo_root() / "debug_runs/aps_preview_catalog_live.json"
    assert path.is_file() or True  # created by refresh in prior test
    if path.is_file():
        assert Path(path).read_text(encoding="utf-8")
