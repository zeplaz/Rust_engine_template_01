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
from .schemas import (
    validate_assembly_snapshot,
    validate_geometry_job,
    validate_massing_preview,
    validate_param_frame,
    validate_studio_task,
)

# S4 — ParamFrame escalate hints → StudioTask kinds (IO-09 · LEGACY pipeline_app_layer).
_ESCALATE_HINT_KIND: dict[str, tuple[str, Any]] = {
    "geometry_job_path": ("geometry", validate_geometry_job),
    "massing_preview_path": ("massing_export", validate_massing_preview),
    "assembly_snapshot_path": ("assembly_build", validate_assembly_snapshot),
}


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
    if payload_path:
        p = Path(str(payload_path))
        if not p.is_absolute():
            p = repo_root() / p
        if p.is_file():
            payload_data = json.loads(p.read_text(encoding="utf-8"))
            if kind == "param_frame":
                validate_param_frame(payload_data)
            elif kind == "geometry":
                validate_geometry_job(payload_data)
            elif kind == "massing_export":
                validate_massing_preview(payload_data)
            elif kind == "assembly_build":
                validate_assembly_snapshot(payload_data)
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


def payload_path_for_task(*, path: Path) -> str:
    """Store repo-relative payload paths when under repo root (addon/APS parity)."""
    resolved = path.resolve()
    root = repo_root().resolve()
    try:
        return str(resolved.relative_to(root)).replace("\\", "/")
    except ValueError:
        return str(resolved)


def load_param_frame(
    *,
    task_id: str | None = None,
    param_frame_path: str | None = None,
) -> tuple[dict[str, Any], Path, int]:
    """Load ParamFrame JSON from a StudioTask or direct path."""
    if task_id and param_frame_path:
        raise ValueError("specify only one of task_id or param_frame_path")
    if task_id:
        task = read_task(task_id)
        if task is None:
            raise FileNotFoundError(f"studio task not found: {task_id}")
        if task.get("kind") != "param_frame":
            raise ValueError(f"task {task_id!r} is not kind=param_frame")
        payload = resolve_payload(task)
        if payload is None or not payload.is_file():
            raise FileNotFoundError(f"param_frame payload missing for {task_id}")
        data = json.loads(payload.read_text(encoding="utf-8"))
        seed = int(task.get("seed", data.get("seed", 0)))
        return data, payload, seed
    if param_frame_path:
        p = Path(str(param_frame_path))
        if not p.is_absolute():
            p = repo_root() / p
        p = p.resolve()
        if not p.is_file():
            raise FileNotFoundError(f"param frame not found: {p}")
        data = json.loads(p.read_text(encoding="utf-8"))
        seed = int(data.get("seed", 0))
        return data, p, seed
    raise ValueError("task_id or param_frame_path required")


def escalate_from_param_frame(
    *,
    task_id: str | None = None,
    param_frame_path: str | None = None,
    run: bool = False,
    blender_session: str | None = None,
) -> dict[str, Any]:
    """Enqueue StudioTasks for optional ParamFrame escalate.*_path hints (same schemas as CLI/addon)."""
    frame, frame_path, seed = load_param_frame(
        task_id=task_id, param_frame_path=param_frame_path
    )
    validate_param_frame(frame)
    escalate = frame.get("escalate")
    if not isinstance(escalate, dict) or not escalate:
        raise ValueError("param frame has no escalate hints")
    stem = str(frame.get("frame_id") or frame_path.stem).strip() or "param_frame"
    parent = (task_id or stem).replace("/", "_").replace("\\", "_")
    children: list[dict[str, Any]] = []
    for hint_key, (kind, validator) in _ESCALATE_HINT_KIND.items():
        raw_path = escalate.get(hint_key)
        if not raw_path:
            continue
        payload = Path(str(raw_path))
        if not payload.is_absolute():
            payload = repo_root() / payload
        payload = payload.resolve()
        if not payload.is_file():
            raise FileNotFoundError(f"escalate {hint_key} missing: {payload}")
        payload_data = json.loads(payload.read_text(encoding="utf-8"))
        validator(payload_data)
        child_id = f"{parent}_esc_{kind}"
        rel_payload = payload_path_for_task(path=payload)
        child = enqueue(
            kind=kind,
            seed=seed,
            payload_path=rel_payload,
            task_id=child_id,
            blender_session=blender_session,
        )
        if run:
            child = run_queued(child_id)
        children.append(child)
    if not children:
        raise ValueError("escalate block present but no recognized *_path hints set")
    rel_frame = payload_path_for_task(path=frame_path)
    out: dict[str, Any] = {
        "param_frame_path": rel_frame,
        "frame_id": frame.get("frame_id"),
        "source_task_id": task_id,
        "children": children,
        "ref_books_dual_cite": {
            "new_rows": ["IO-09"],
            "legacy_keywords": ["pipeline_app_layer", "asset_pipeline"],
        },
    }
    return out


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


def run_queued_massing_export(task_id: str) -> dict[str, Any]:
    """Validate massing_preview payload, write Republic QC HTML, mark StudioTask done."""
    task = read_task(task_id)
    if task is None:
        raise FileNotFoundError(f"studio task not found: {task_id}")
    if task["kind"] != "massing_export":
        raise ValueError(
            f"run_queued_massing_export only for kind=massing_export, got {task['kind']}"
        )
    payload = resolve_payload(task)
    if payload is None or not payload.is_file():
        raise FileNotFoundError(f"payload missing for {task_id}: {task.get('payload_path')}")
    mark_status(task_id, "running")
    try:
        data = json.loads(payload.read_text(encoding="utf-8"))
        validate_massing_preview(data)
        from .massing_qc import massing_qc_view
        from .studio_html import write_studio_report

        qc = massing_qc_view(data)
        report_dir = repo_root() / "debug_runs" / "studio_dash"
        report_dir.mkdir(parents=True, exist_ok=True)
        report_path = report_dir / f"{task_id}_massing_qc.html"
        write_studio_report(qc, report_path, title=str(qc.get("title") or "massing_preview QC"))
        rel = str(report_path.relative_to(repo_root())).replace("\\", "/")
        return mark_status(task_id, "done", error=None, report_path=rel)
    except Exception as exc:
        return mark_status(task_id, "failed", error=str(exc))


def run_queued_assembly_build(task_id: str) -> dict[str, Any]:
    """Validate assembly_snapshot payload and run headless assembly_build job."""
    from .tile_pipeline import assembly_build_run

    task = read_task(task_id)
    if task is None:
        raise FileNotFoundError(f"studio task not found: {task_id}")
    if task["kind"] != "assembly_build":
        raise ValueError(
            f"run_queued_assembly_build only for kind=assembly_build, got {task['kind']}"
        )
    payload = resolve_payload(task)
    if payload is None or not payload.is_file():
        raise FileNotFoundError(f"payload missing for {task_id}: {task.get('payload_path')}")
    mark_status(task_id, "running")
    try:
        result = assembly_build_run(payload)
        if result.get("ok"):
            return mark_status(task_id, "done", error=None)
        return mark_status(
            task_id,
            "failed",
            error=str(result.get("error") or result.get("status") or "assembly_build failed"),
        )
    except Exception as exc:
        return mark_status(task_id, "failed", error=str(exc))


def run_queued_param_frame(task_id: str) -> dict[str, Any]:
    """ParamFrame → massing_preview.json + Republic QC HTML (no bpy)."""
    from .massing_from_param_frame import export_massing_preview_from_param_frame
    from .massing_qc import massing_qc_view
    from .studio_html import write_studio_report

    task = read_task(task_id)
    if task is None:
        raise FileNotFoundError(f"studio task not found: {task_id}")
    if task["kind"] != "param_frame":
        raise ValueError(f"run_queued_param_frame only for kind=param_frame, got {task['kind']}")
    payload = resolve_payload(task)
    if payload is None or not payload.is_file():
        raise FileNotFoundError(f"payload missing for {task_id}: {task.get('payload_path')}")
    mark_status(task_id, "running")
    try:
        massing_out = repo_root() / "debug_runs" / "studio_dash" / "massing_preview.json"
        data = export_massing_preview_from_param_frame(massing_out, payload)
        qc = massing_qc_view(data)
        report_dir = repo_root() / "debug_runs" / "studio_dash"
        report_dir.mkdir(parents=True, exist_ok=True)
        report_path = report_dir / f"{task_id}_param_frame_massing_qc.html"
        write_studio_report(qc, report_path, title=str(qc.get("title") or "massing_preview QC"))
        rel = str(report_path.relative_to(repo_root())).replace("\\", "/")
        massing_rel = str(massing_out.relative_to(repo_root())).replace("\\", "/")
        done = mark_status(task_id, "done", error=None, report_path=rel)
        done["massing_preview_path"] = massing_rel
        return done
    except Exception as exc:
        return mark_status(task_id, "failed", error=str(exc))


# CQ-41: kind → runner dispatch (status files kept — Blender addon polls async handoff markers).
_QUEUED_RUNNERS: dict[str, Any] = {
    "geometry": lambda tid: run_queued_geometry(tid),
    "massing_export": lambda tid: run_queued_massing_export(tid),
    "assembly_build": lambda tid: run_queued_assembly_build(tid),
    "param_frame": lambda tid: run_queued_param_frame(tid),
}


def run(kind: str, payload: Path, *, task_id: str | None = None, seed: int = 0) -> dict[str, Any]:
    """Enqueue + run a StudioTask synchronously via the kind dispatch table (CQ-41)."""
    task = enqueue(kind=kind, seed=seed, payload_path=str(payload), task_id=task_id)
    return run_queued(str(task["task_id"]))


def run_queued(task_id: str) -> dict[str, Any]:
    """Dispatch StudioTask by kind (geometry → Blender; massing_export / param_frame → QC)."""
    task = read_task(task_id)
    if task is None:
        raise FileNotFoundError(f"studio task not found: {task_id}")
    kind = str(task.get("kind") or "")
    runner = _QUEUED_RUNNERS.get(kind)
    if runner is None:
        raise ValueError(f"no runner for StudioTask kind={kind!r}")
    return runner(task_id)


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
