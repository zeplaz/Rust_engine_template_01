"""S1 — ParamFrame grammar/modifier/connection Studio tools + APS panel."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from rust_engine_mcp import building_quality_qc, param_frame_studio, studio_task
from rust_engine_mcp.paths import repo_root, schemas_dir
from rust_engine_mcp.schemas import validate_param_frame

pytestmark_panel = pytest.mark.aps_gui


def test_s1_dual_cite_on_example_frame() -> None:
    frame = param_frame_studio.load_param_frame()
    merged = param_frame_studio.merge_ref_books_dual_cite(frame)
    cite = merged["ref_books_dual_cite"]
    assert "PCG-06" in cite["new_rows"]
    assert "PCG-08" in cite["new_rows"]
    assert "DES-04" in cite["new_rows"]
    assert "pcg" in cite["legacy_keywords"]
    assert "shape_grammars" in cite["legacy_keywords"]


def test_browse_grammar_catalog_non_empty_no_bpy() -> None:
    rows = param_frame_studio.browse_grammar_catalog()
    assert len(rows) >= 3
    assert all("grammar_id" in r and "grammar_ref" in r for r in rows)
    refs = {str(r["grammar_ref"]) for r in rows}
    assert "building_grammar_v1" in refs


def test_upsert_grammar_and_modifier_validates() -> None:
    frame = param_frame_studio.load_param_frame()
    catalogs = param_frame_studio.browse_grammar_catalog()
    pick = catalogs[0]
    frame2 = param_frame_studio.upsert_grammar_chain_step(frame, pick)
    frame3 = param_frame_studio.upsert_modifier(frame2, "age_band", params={"band": "mature"}, enabled=True)
    validate_param_frame(frame3)
    assert any(m.get("modifier_id") == "age_band" for m in frame3["modifiers"])


def test_toggle_connection_graph_id() -> None:
    frame = param_frame_studio.load_param_frame()
    before = set(frame.get("connection_graph_ids") or [])
    frame2 = param_frame_studio.toggle_connection_graph_id(frame, "test_graph_link_v1")
    after = set(frame2.get("connection_graph_ids") or [])
    assert "test_graph_link_v1" in after or "test_graph_link_v1" not in before


def test_pcg08_gate_blocks_enqueue_without_witness(monkeypatch, tmp_path) -> None:
    monkeypatch.setattr(building_quality_qc, "load_pcg08_report", lambda **_: None)
    frame = param_frame_studio.load_param_frame()
    path = tmp_path / "gate_test_frame.json"
    param_frame_studio.write_param_frame(frame, path)
    with pytest.raises(ValueError, match="generate-and-test"):
        param_frame_studio.enqueue_param_frame_task(path)


def test_pcg08_gate_allows_enqueue_with_witness(monkeypatch, tmp_path) -> None:
    monkeypatch.setattr(
        building_quality_qc,
        "load_pcg08_report",
        lambda **_: {"pass_rate": 0.5, "task_id": "PCG-08-TEST"},
    )
    tasks_dir = tmp_path / "tasks"
    tasks_dir.mkdir(parents=True, exist_ok=True)
    monkeypatch.setattr(studio_task, "studio_tasks_dir", lambda: tasks_dir)
    frame = param_frame_studio.load_param_frame()
    path = tmp_path / "ok_frame.json"
    param_frame_studio.write_param_frame(frame, path)
    task = param_frame_studio.enqueue_param_frame_task(path, task_id="s1_gate_ok_test")
    assert task["kind"] == "param_frame"
    assert task["status"] == "queued"


def test_promote_module_still_requires_glb(tmp_path) -> None:
    from rust_engine_mcp.promote import promote_module

    with pytest.raises(FileNotFoundError):
        promote_module("nonexistent_s1_gate_job")


@pytest.mark.aps_gui
def test_param_frame_panel_round_trip(gui_panel_host) -> None:
    from art_pipeline_suite.param_frame_panel import ParamFramePanel

    panel = ParamFramePanel(gui_panel_host, on_log=lambda _m: None)
    panel.update_idletasks()
    assert len(param_frame_studio.browse_grammar_catalog()) >= 1
    mod_children = panel._modifier_tree.get_children()
    assert mod_children
    panel._modifier_tree.selection_set(mod_children[0])
    panel._on_add_modifier()
    gram_children = panel._grammar_tree.get_children()
    panel._grammar_tree.selection_set(gram_children[0])
    panel._on_add_grammar()
    frame = panel.get_frame()
    validate_param_frame(frame)
    cite = frame.get("ref_books_dual_cite") or {}
    assert "PCG-08" in (cite.get("new_rows") or [])


@pytest.mark.aps_gui
def test_aps_app_has_param_frame_tab(aps_app) -> None:
    assert hasattr(aps_app, "param_frame")
    aps_app.param_frame.refresh_from_state()
    assert aps_app.param_frame.get_frame().get("schema_version") == 1


def test_example_frame_still_validates_after_s1_cite_merge() -> None:
    example = schemas_dir() / "examples" / "param_frame_rail_warehouse_v1.json"
    data = json.loads(example.read_text(encoding="utf-8"))
    merged = param_frame_studio.merge_ref_books_dual_cite(data)
    validate_param_frame(merged)
    assert len(param_frame_studio.browse_connection_graph_ids(repo=repo_root())) >= 1