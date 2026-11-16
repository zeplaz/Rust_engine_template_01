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


def test_run_queued_massing_export(tmp_path, monkeypatch):
    from rust_engine_mcp.paths import repo_root, schemas_dir

    monkeypatch.setattr(studio_task, "studio_tasks_dir", lambda: tmp_path)
    example = schemas_dir() / "examples" / "massing_preview_pilot_rail_warehouse_v1.json"
    payload = tmp_path / "massing_preview.json"
    payload.write_text(example.read_text(encoding="utf-8"), encoding="utf-8")
    # payload_path relative to repo for resolve_payload
    rel = str(payload)  # absolute OK via resolve
    task = studio_task.enqueue(
        kind="massing_export",
        seed=1,
        payload_path=str(payload),
        task_id="massing_export_test",
    )
    assert task["status"] == "queued"
    done = studio_task.run_queued_massing_export("massing_export_test")
    assert done["status"] == "done", done.get("error")
    assert done.get("report_path")
    report = repo_root() / done["report_path"]
    assert report.is_file()
    html = report.read_text(encoding="utf-8")
    assert "Grammar rule chain" in html or "seed" in html
    assert "Massing columns" in html
