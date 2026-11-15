"""P0 — StudioTask bus enqueue / status round-trip (no Blender required)."""

from __future__ import annotations

import json

from rust_engine_mcp import studio_task
from rust_engine_mcp.schemas import validate_studio_task


def test_enqueue_and_status_round_trip(tmp_path, monkeypatch):
    monkeypatch.setattr(studio_task, "studio_tasks_dir", lambda: tmp_path)
    task = studio_task.enqueue(
        kind="geometry",
        seed=7,
        payload_path="tools/mcp/schemas/examples/wall_steel_1u_lod0_job.json",
        task_id="studio_test_roundtrip",
    )
    assert task["status"] == "queued"
    assert task["seed"] == 7
    loaded = studio_task.read_task("studio_test_roundtrip")
    assert loaded is not None
    validate_studio_task(loaded)
    updated = studio_task.mark_status("studio_test_roundtrip", "done", report_path="debug_runs/x.html")
    assert updated["status"] == "done"
    assert updated["report_path"].endswith("x.html")


def test_example_schema_validates():
    from rust_engine_mcp.paths import schemas_dir

    example = schemas_dir() / "examples" / "studio_task_geometry_v1.json"
    data = json.loads(example.read_text(encoding="utf-8"))
    validate_studio_task(data)


def test_handoff_marker(tmp_path, monkeypatch):
    monkeypatch.setattr(studio_task, "studio_tasks_dir", lambda: tmp_path)
    studio_task.enqueue(kind="open_staging", seed=1, task_id="handoff_1")
    marker = studio_task.blender_handoff_marker("handoff_1")
    assert marker.is_file()
    task = studio_task.read_task("handoff_1")
    assert task is not None
    assert "blender_session" in task
