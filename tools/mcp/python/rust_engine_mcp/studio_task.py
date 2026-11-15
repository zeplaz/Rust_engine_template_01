"""StudioTask bus — shared enqueue/status for APS, CLI, and Blender addon.

Tasks live under ``jobs_root()/studio_tasks/<task_id>.json``.
Underlying blender jobs still use ``<job_id>.status.json`` via blender_runner.
"""

from __future__ import annotations

import json
import time
import uuid
from pathlib import Path
from typing import Any

from . import blender_runner
from .paths import jobs_root, repo_root, resolve_under_root
from .schemas import validate_studio_task


def studio_tasks_dir() -> Path:
    p = jobs_root() / "studio_tasks"
    p.mkdir(parents=True, exist_ok=True)
    return p


def _task_path(task_id: str) -> Path:
    name = str(task_id or "").strip()
    if (
        not name
        or name in {".", ".."}
        or "/" in name
        or "\\" in name
        or ":" in name
        or Path(name).is_absolute()
    ):
        raise ValueError(f"task_id must be a single folder name: {task_id!r}")
    return resolve_under_root(studio_tasks_dir(), f"{name}.json", label="studio task")


def write_task(task: dict[str, Any]) -> Path:
    validate_studio_task(task)
    task = dict(task)
    task["updated_at"] = time.time()
    if "created_at" not in task:
        task["created_at"] = task["updated_at"]
    path = _task_path(str(task["task_id"]))
    path.write_text(json.dumps(task, indent=2), encoding="utf-8")
    return path


def read_task(task_id: str) -> dict[str, Any] | None:
    path = _task_path(task_id)
    if not path.is_file():
        return None
    data = json.loads(path.read_text(encoding="utf-8"))
    validate_studio_task(data)
    return data


def list_tasks(*, limit: int = 50) -> list[dict[str, Any]]:
    rows: list[tuple[float, dict[str, Any]]] = []
    for path in studio_tasks_dir().glob("*.json"):
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
            validate_studio_task(data)
            rows.append((float(data.get("updated_at") or 0), data))
        except (OSError, json.JSONDecodeError, Exception):
            continue
    rows.sort(key=lambda t: t[0], reverse=True)
    return [d for _, d in rows[: max(1, limit)]]


def enqueue(
    *,
    kind: str,
    seed: int,
    payload_path: str | None = None,
    task_id: str | None = None,
    blender_session: str | None = None,
) -> dict[str, Any]:
    if not isinstance(seed, int) or isinstance(seed, bool) or seed < 0:
        raise ValueError("seed must be a non-negative int")
    tid = (task_id or f"studio_{uuid.uuid4().hex[:12]}").strip()
    task: dict[str, Any] = {
        "schema_version": 1,
        "task_id": tid,
        "kind": kind,
        "seed": seed,
        "status": "queued",
    }
    if payload_path:
        task["payload_path"] = str(payload_path)
    if blender_session:
        task["blender_session"] = blender_session
    write_task(task)
    return task


def mark_status(
    task_id: str,
    status: str,
    *,
    error: str | None = None,
    report_path: str | None = None,
) -> dict[str, Any]:
    task = read_task(task_id)
    if task is None:
        raise FileNotFoundError(f"studio task not found: {task_id}")
    task["status"] = status
    if error:
        task["error"] = error
    elif "error" in task and status == "done":
        task.pop("error", None)
    if report_path:
        task["report_path"] = report_path
    write_task(task)
    return task


def resolve_payload(task: dict[str, Any]) -> Path | None:
    raw = task.get("payload_path")
    if not raw:
        return None
    p = Path(str(raw))
    if not p.is_absolute():
        p = repo_root() / p
    return p.resolve()


def run_queued_geometry(task_id: str) -> dict[str, Any]:
    """Run underlying geometry job for a queued geometry StudioTask (headless)."""
    task = read_task(task_id)
    if task is None:
        raise FileNotFoundError(f"studio task not found: {task_id}")
    if task["kind"] != "geometry":
        raise ValueError(f"run_queued_geometry only for kind=geometry, got {task['kind']}")
    payload = resolve_payload(task)
    if payload is None or not payload.is_file():
        raise FileNotFoundError(f"payload missing for {task_id}: {task.get('payload_path')}")
    mark_status(task_id, "running")
    result = blender_runner.run_geometry_job(payload)
    if result.status == "done":
        return mark_status(task_id, "done", error=None)
    return mark_status(task_id, "failed", error=result.error or result.status)


def blender_handoff_marker(task_id: str) -> Path:
    """Write a marker file Blender addon / APS can poll for interactive session."""
    task = read_task(task_id)
    if task is None:
        raise FileNotFoundError(task_id)
    marker = studio_tasks_dir() / f"{task_id}.blender_handoff"
    marker.write_text(
        json.dumps(
            {
                "task_id": task_id,
                "payload_path": task.get("payload_path"),
                "seed": task.get("seed"),
                "kind": task.get("kind"),
                "written_at": time.time(),
            },
            indent=2,
        ),
        encoding="utf-8",
    )
    mark_status(task_id, task.get("status") or "queued")
    task = read_task(task_id) or task
    task["blender_session"] = str(marker)
    write_task(task)
    return marker
