"""Optional headless keyframe export gate (DESIGN-TILE-SPINE-001)."""

from __future__ import annotations

import json
import os
from pathlib import Path

import pytest

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.tile_pipeline import (
    tile_keyframe_export,
    tile_keyframe_headless_enabled,
)


def test_keyframe_headless_disabled_by_default(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("RUST_ENGINE_TILE_KEYFRAME_HEADLESS", raising=False)
    assert not tile_keyframe_headless_enabled()
    batch = repo_root() / "tools/mcp/schemas/examples/tile_batch_rowhouse_victorian_production_v1.json"
    result = tile_keyframe_export(batch)
    assert not result["ok"]
    assert result["status"] == "keyframe_headless_disabled"


def test_keyframe_headless_dry_run_one_variant(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("RUST_ENGINE_TILE_KEYFRAME_HEADLESS", "1")
    monkeypatch.setenv("RUST_ENGINE_TILE_DRY_RUN", "1")
    batch_path = repo_root() / "tools/mcp/schemas/examples/tile_batch_rowhouse_victorian_production_v1.json"
    batch = json.loads(batch_path.read_text(encoding="utf-8"))
    batch["variants"] = batch["variants"][:1]
    tmp = repo_root() / "tools/mcp/schemas/examples/.test_keyframe_headless_one.json"
    tmp.write_text(json.dumps(batch, indent=2) + "\n", encoding="utf-8")
    try:
        result = tile_keyframe_export(tmp)
        assert result["ok"], result
        assert result.get("export_mode") == "headless_light_rig"
        png = result["png_paths"][0]
        assert Path(png).is_file()
    finally:
        tmp.unlink(missing_ok=True)
        batch_id = batch["batch_id"]
        staging = repo_root() / "assets/staging/tiles" / batch_id
        for p in staging.glob("clean_day.png"):
            p.unlink(missing_ok=True)
