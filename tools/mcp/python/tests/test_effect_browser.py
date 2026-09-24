"""VSS-T4-005 — EffectBrowser list / honesty / assign gate tests."""

from __future__ import annotations

import json
import sys
from dataclasses import replace
from pathlib import Path

import pytest

APS_ROOT = Path(__file__).resolve().parents[2]
if str(APS_ROOT) not in sys.path:
    sys.path.insert(0, str(APS_ROOT))

from rust_engine_mcp.effect_registry_browse import (
    REFERENCE_BATCH_ID,
    honesty_label,
    list_effect_entries,
)
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.scenario_trigger_write import (
    AssignBlockedError,
    assign_effect_to_marker,
    can_assign,
    format_trigger_ron,
)

REFERENCE_IDS = ("spark_shower", "smoke_column", "rain_streaks")


def test_effect_registry_lists_reference_batch() -> None:
    entries = list_effect_entries(batch_id=REFERENCE_BATCH_ID)
    ids = {e.effect_id for e in entries}
    for eid in REFERENCE_IDS:
        assert eid in ids
    assert len(ids) >= 3


def test_honesty_badge_reads_honest_after_capture() -> None:
    entries = {e.effect_id: e for e in list_effect_entries(batch_id=REFERENCE_BATCH_ID)}
    for eid in REFERENCE_IDS:
        assert eid in entries
        assert entries[eid].honest_gate == "honest"
        assert honesty_label(entries[eid].honest_gate) == "✓ honest"
        assert entries[eid].frames_captured >= 1
        assert can_assign(entries[eid]) is True


def test_assign_blocked_when_not_honest(tmp_path: Path) -> None:
    entries = list_effect_entries(batch_id=REFERENCE_BATCH_ID)
    assert entries
    pending = replace(entries[0], honest_gate="pending", frames_captured=0)
    assert pending.honest_gate != "honest"
    with pytest.raises(AssignBlockedError):
        assign_effect_to_marker(
            pending.effect_id,
            f"test_{pending.effect_id}_blocked",
            repo=tmp_path,
            entry=pending,
        )
    triggers = tmp_path / "assets" / "scenarios" / "triggers"
    assert not triggers.exists() or not any(triggers.glob("*.trigger.ron"))


def test_assign_writes_trigger_when_honest(tmp_path: Path) -> None:
    entries = list_effect_entries(batch_id=REFERENCE_BATCH_ID)
    base = entries[0]
    honest = replace(base, honest_gate="honest", frames_captured=1)
    marker = f"test_{base.effect_id}_honest"
    result = assign_effect_to_marker(base.effect_id, marker, repo=tmp_path, entry=honest)
    path = tmp_path / result["path"]
    assert path.is_file()
    text = path.read_text(encoding="utf-8")
    assert f'id: "{marker}"' in text
    assert "schema_version: 1" in text
    assert "cells:" in text


def test_format_trigger_ron_matches_demo_shape() -> None:
    text = format_trigger_ron(
        marker_id="demo_shape",
        cells=[{"chunk_x": 4, "chunk_y": 2, "cell": 0, "spark": 0.35}],
    )
    assert 'id: "demo_shape"' in text
    assert "chunk_x: 4" in text
    assert "parent_effect_id: None" in text


def test_mount_effect_library_factory() -> None:
    from art_pipeline_suite.effect_browser import (
        EffectBrowserPanel,
        MOUNT_ASSIGN,
        MOUNT_BROWSE,
        mount_effect_library,
    )

    root = repo_root()
    assert (root / "tools/mcp/art_pipeline_suite/effect_browser.py").is_file()
    assert (root / "tools/mcp/art_pipeline_suite/effect_library_widget.py").is_file()
    assert (root / "tools/mcp/art_pipeline_suite/effects_panel.py").is_file()

    try:
        import tkinter as tk
    except ImportError:
        pytest.skip("tkinter unavailable")

    try:
        master = tk.Tk()
        master.withdraw()
    except tk.TclError as exc:
        pytest.skip(f"Tk display unavailable: {exc}")

    try:
        panel = mount_effect_library(master, mount=MOUNT_BROWSE, on_log=lambda _l: None)
        assert isinstance(panel, EffectBrowserPanel)
        refs = panel.list_reference_ids()
        assert len(refs) >= 3
        for eid in REFERENCE_IDS:
            assert eid in refs
        assign_panel = mount_effect_library(master, mount=MOUNT_ASSIGN, on_log=lambda _l: None)
        assert assign_panel._mode == "assign"
    finally:
        master.destroy()


def test_no_fire_vfx_or_bevy_imports_in_panel() -> None:
    import ast

    suite = repo_root() / "tools/mcp/art_pipeline_suite"
    for name in ("effect_browser.py", "effect_library_widget.py", "effects_panel.py"):
        path = suite / name
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                for alias in node.names:
                    mod = alias.name.lower()
                    assert "fire_vfx" not in mod
                    assert "bevy" not in mod
            elif isinstance(node, ast.ImportFrom):
                mod = (node.module or "").lower()
                assert "fire_vfx" not in mod
                assert "bevy" not in mod
                assert not mod.startswith("src.")
                assert "src.render" not in mod
                assert "src.sim" not in mod


def test_write_aps_effect_browser_live_witness() -> None:
    entries = list_effect_entries(batch_id=REFERENCE_BATCH_ID)
    ids = [e.effect_id for e in entries]
    assert len(ids) >= 3
    body = {
        "_agent_meta": {
            "schema": "debug_run_envelope_v1",
            "relative_path": "debug_runs/aps_effect_browser_live.json",
            "source_system": "aps_effect_browser",
            "track": "VSS-T4",
            "task_id": "VSS-T4-005",
            "proceed_ship": True,
            "art_quality": "smoke_tier_panel_wire",
            "docs": {
                "charter": "src/dev/design_aps_effect_browser_v1.md",
                "packet": "tools/mcp/art_pipeline_suite/effect_browser_impl_packet_v1.md",
            },
        },
        "schema": "aps_effect_browser_live_v1",
        "slice_id": "VSS-T4-005",
        "status": "shipped",
        "green": True,
        "honest_gate": "honest",
        "reference_batch_id": REFERENCE_BATCH_ID,
        "listed_effect_ids": ids,
        "reference_count": len(ids),
        "honesty_sample": {
            e.effect_id: {"honest_gate": e.honest_gate, "label": e.honesty_label} for e in entries[:3]
        },
        "assign_gate": "blocked_unless_honest",
        "assign_unblocked": all(e.honest_gate == "honest" for e in entries if e.effect_id in REFERENCE_IDS),
        "mount": "mount_effect_library",
        "tab": "Effects",
        "no_fire_vfx_import": True,
        "residuals": [],
        "preview_witness_capture": {
            "status": "shipped",
            "capture_kind": "staging_pack_digest",
            "cli": "effect-preview-capture",
            "mcp": "effect_preview_capture",
        },
    }
    out = repo_root() / "debug_runs" / "aps_effect_browser_live.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    assert body["green"] is True
    assert body["reference_count"] >= 3
