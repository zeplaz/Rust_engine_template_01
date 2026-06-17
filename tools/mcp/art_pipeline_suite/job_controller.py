"""APS-UX-ASYNC-001 — background job runner for Tk APS (non-blocking mainloop)."""

from __future__ import annotations

import threading
import uuid
from dataclasses import dataclass, field
from enum import Enum, auto
from typing import Callable

JobWorker = Callable[[threading.Event], "JobResult"]
DoneCallback = Callable[["JobRecord"], None]
UiScheduler = Callable[[Callable[[], None]], None]


class JobState(Enum):
    QUEUED = auto()
    RUNNING = auto()
    CANCELLING = auto()
    CANCELLED = auto()
    DONE = auto()
    FAILED = auto()


@dataclass
class JobResult:
    ok: bool
    summary: str
    detail: str | None = None
    data: dict | None = None


@dataclass
class JobRecord:
    job_id: str
    label: str
    state: JobState = JobState.QUEUED
    result: JobResult | None = None
    cancellable: bool = True
    step_count: int = 1
    step_index: int = 0
    step_label: str | None = None
    _cancel: threading.Event = field(default_factory=threading.Event, repr=False)


class JobController:
    """Single-flight job runner; optional Tk `after(0, …)` scheduler for on_done."""

    def __init__(self, ui_scheduler: UiScheduler | None = None) -> None:
        self._ui_scheduler = ui_scheduler or (lambda fn: fn())
        self._lock = threading.Lock()
        self._active: JobRecord | None = None

    def is_busy(self) -> bool:
        with self._lock:
            return self._busy_locked()

    def _busy_locked(self) -> bool:
        return self._active is not None and self._active.state in (
            JobState.QUEUED,
            JobState.RUNNING,
            JobState.CANCELLING,
        )

    def active_job(self) -> JobRecord | None:
        with self._lock:
            return self._active

    def run(
        self,
        label: str,
        worker: JobWorker,
        on_done: DoneCallback | None = None,
    ) -> str:
        with self._lock:
            if self._busy_locked():
                raise RuntimeError("Another job is running")
            job_id = uuid.uuid4().hex[:12]
            record = JobRecord(job_id=job_id, label=label, state=JobState.RUNNING)
            self._active = record

        def _thread_main() -> None:
            try:
                result = worker(record._cancel)
            except Exception as exc:  # noqa: BLE001
                result = JobResult(False, f"Job failed: {exc}", detail=str(exc))
            with self._lock:
                if record._cancel.is_set():
                    record.state = JobState.CANCELLED
                    if result.summary == "Cancelled" or not result.ok:
                        record.result = result
                    else:
                        record.result = JobResult(False, "Cancelled")
                elif result.ok:
                    record.state = JobState.DONE
                    record.result = result
                else:
                    record.state = JobState.FAILED
                    record.result = result
                self._active = None

            if on_done:

                def _cb() -> None:
                    on_done(record)

                self._ui_scheduler(_cb)

        threading.Thread(target=_thread_main, daemon=True, name=f"aps-job-{label}").start()
        return job_id

    def cancel(self, job_id: str | None = None) -> None:
        with self._lock:
            job = self._active
            if job is None:
                return
            if job_id and job.job_id != job_id:
                return
            job.state = JobState.CANCELLING
            job._cancel.set()
