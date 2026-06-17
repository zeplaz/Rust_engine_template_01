"""APS witness refresh smoke."""

from __future__ import annotations

from rust_engine_mcp.aps_witness_refresh import refresh_aps_witnesses
from rust_engine_mcp.paths import repo_root


def test_refresh_bundle_green() -> None:
    body = refresh_aps_witnesses()
    assert body.get("program_id") == "APS-WITNESS-REFRESH-001"
    assert (repo_root() / "debug_runs/aps_artist_tool_modules_live.json").is_file()
    assert body.get("module_count", 0) >= 10
    assert body.get("aps_imports", {}).get("ok") is True
    assert body.get("green") is True


def test_refresh_refuses_green_when_aps_imports_fail(monkeypatch) -> None:
    from rust_engine_mcp import aps_witness_refresh

    monkeypatch.setattr(
        aps_witness_refresh,
        "_pytest_aps_imports",
        lambda: {"ok": False, "summary": "1 failed", "tests": ["tests/test_aps_imports.py"]},
    )
    monkeypatch.setattr(
        aps_witness_refresh,
        "_pytest_aps_smoke",
        lambda: {"ok": True, "summary": "1 passed", "tests": []},
    )
    body = refresh_aps_witnesses()
    assert body.get("green") is False
    assert body.get("aps_imports", {}).get("ok") is False
