"""S4 — ParamFrame escalate → StudioTask children (IO-09 · pipeline_app_layer)."""

from __future__ import annotations

import json

from rust_engine_mcp import studio_task
from rust_engine_mcp.paths import schemas_dir
from rust_engine_mcp.schemas import (
    validate_assembly_snapshot,
    validate_geometry_job,
    validate_massing_preview,
    validate_param_frame,
    validate_studio_task,
)


def test_escalate_hint_payloads_validate_against_job_schemas() -> None:
    from rust_engine_mcp.paths import repo_root

    frame_path = schemas_dir() / "examples" / "param_frame_rail_warehouse_v1.json"
    frame = json.loads(frame_path.read_text(encoding="utf-8"))
    validate_param_frame(frame)
    escalate = frame["escalate"]
    massing = repo_root() / escalate["massing_preview_path"]
    assembly = repo_root() / escalate["assembly_snapshot_path"]
    validate_massing_preview(json.loads(massing.read_text(encoding="utf-8")))
    validate_assembly_snapshot(json.loads(assembly.read_text(encoding="utf-8")))
    assert "ref_books_dual_cite" not in frame


def test_escalate_from_param_frame_enqueues_children(tmp_path, monkeypatch) -> None:
    monkeypatch.setattr(studio_task, "studio_tasks_dir", lambda: tmp_path)
    rel = "tools/mcp/schemas/examples/param_frame_rail_warehouse_v1.json"
    parent = studio_task.enqueue(
        kind="param_frame",
        seed=1,
        payload_path=rel,
        task_id="pf_escalate_parent",
    )
    assert parent["status"] == "queued"
    result = studio_task.escalate_from_param_frame(task_id="pf_escalate_parent")
    assert result["source_task_id"] == "pf_escalate_parent"
    assert "ref_books_dual_cite" not in result
    kinds = {c["kind"] for c in result["children"]}
    assert kinds == {"massing_export", "assembly_build"}
    for child in result["children"]:
        validate_studio_task(child)
        loaded = studio_task.read_task(child["task_id"])
        assert loaded is not None
        assert loaded["status"] == "queued"


def test_escalate_from_frame_path_without_task(tmp_path, monkeypatch) -> None:
    monkeypatch.setattr(studio_task, "studio_tasks_dir", lambda: tmp_path)
    rel = "tools/mcp/schemas/examples/param_frame_rail_warehouse_v1.json"
    result = studio_task.escalate_from_param_frame(param_frame_path=rel)
    assert len(result["children"]) == 2


def test_enqueue_geometry_validates_job_schema(tmp_path, monkeypatch) -> None:
    monkeypatch.setattr(studio_task, "studio_tasks_dir", lambda: tmp_path)
    job_rel = "tools/mcp/schemas/examples/wall_steel_1u_lod0_job.json"
    task = studio_task.enqueue(kind="geometry", seed=3, payload_path=job_rel, task_id="geo_schema")
    validate_studio_task(task)
    payload = studio_task.resolve_payload(task)
    assert payload is not None
    validate_geometry_job(json.loads(payload.read_text(encoding="utf-8")))