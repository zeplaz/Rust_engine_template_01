"""PLAN-REF-BOOKS-UPGRADE-v3 S2 — ParamFrame → massing_preview (no bpy)."""

from __future__ import annotations

import json
from pathlib import Path

from rust_engine_mcp.massing_from_param_frame import (
    build_massing_preview_from_param_frame,
    export_massing_preview_from_param_frame,
)
from rust_engine_mcp.paths import schemas_dir
from rust_engine_mcp.schemas import validate_massing_preview

PARAM_FRAME = schemas_dir() / "examples" / "param_frame_rail_warehouse_v1.json"


def test_param_frame_example_exports_massing_preview() -> None:
    frame = json.loads(PARAM_FRAME.read_text(encoding="utf-8"))
    data = build_massing_preview_from_param_frame(frame)
    validate_massing_preview(data)
    assert data["param_frame_id"] == "rail_warehouse_param_frame_v1"
    assert data["pilot_id"] == "logistics_rail_warehouse_v0"
    assert data["presentation_only"] is True
    assert data["sim_occupation"] is False
    assert len(data["massing_columns"]) > 0
    assert "ref_books_dual_cite" not in data  # book bookkeeping stays out of payloads (CQ-79)
    for col in data["massing_columns"]:
        assert 1 <= col["floors"] <= 2


def test_export_writes_valid_json(tmp_path: Path) -> None:
    out = tmp_path / "massing_preview.json"
    written = export_massing_preview_from_param_frame(out, PARAM_FRAME)
    validate_massing_preview(written)
    loaded = json.loads(out.read_text(encoding="utf-8"))
    assert loaded["param_frame_id"] == written["param_frame_id"]


def test_studio_task_param_frame_run(monkeypatch, tmp_path) -> None:
    from rust_engine_mcp import studio_task

    jobs = tmp_path / "jobs"
    jobs.mkdir(parents=True)
    monkeypatch.setattr(studio_task, "jobs_root", lambda: jobs)

    task = studio_task.enqueue(
        kind="param_frame",
        seed=1,
        payload_path=str(PARAM_FRAME),
        task_id="param_frame_massing_s2_test",
    )
    done = studio_task.run_queued_param_frame(task["task_id"])
    assert done["status"] == "done"
    assert done.get("report_path")
    assert done.get("massing_preview_path")