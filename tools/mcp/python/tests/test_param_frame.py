"""S0 — ParamFrame schema + StudioTask param_frame round-trip."""

from __future__ import annotations

import json

from rust_engine_mcp import studio_task
from rust_engine_mcp.paths import schemas_dir
from rust_engine_mcp.schemas import validate_param_frame, validate_studio_task


def test_example_param_frame_validates() -> None:
    example = schemas_dir() / "examples" / "param_frame_rail_warehouse_v1.json"
    data = json.loads(example.read_text(encoding="utf-8"))
    validate_param_frame(data)
    cite = data.get("ref_books_dual_cite") or {}
    assert "IO-09" in cite.get("new_rows", [])
    assert "shape_grammars" in cite.get("legacy_keywords", [])


def test_studio_task_param_frame_example_validates() -> None:
    example = schemas_dir() / "examples" / "studio_task_param_frame_v1.json"
    data = json.loads(example.read_text(encoding="utf-8"))
    validate_studio_task(data)


def test_param_frame_studio_task_round_trip(tmp_path, monkeypatch) -> None:
    monkeypatch.setattr(studio_task, "studio_tasks_dir", lambda: tmp_path)
    payload_rel = "tools/mcp/schemas/examples/param_frame_rail_warehouse_v1.json"
    task = studio_task.enqueue(
        kind="param_frame",
        seed=1,
        payload_path=payload_rel,
        task_id="param_frame_roundtrip_test",
    )
    assert task["status"] == "queued"
    assert task["kind"] == "param_frame"
    loaded = studio_task.read_task("param_frame_roundtrip_test")
    assert loaded is not None
    validate_studio_task(loaded)
    payload = studio_task.resolve_payload(loaded)
    assert payload is not None and payload.is_file()
    validate_param_frame(json.loads(payload.read_text(encoding="utf-8")))