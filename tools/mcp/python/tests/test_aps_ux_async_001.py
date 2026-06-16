"""APS-UX-ASYNC-001 — JobController headless tests."""

from __future__ import annotations

import sys
import threading
import time
from pathlib import Path

import pytest

APS_SUITE = Path(__file__).resolve().parents[2] / "art_pipeline_suite"
if str(APS_SUITE.parent) not in sys.path:
    sys.path.insert(0, str(APS_SUITE.parent))

from art_pipeline_suite.job_controller import (  # noqa: E402
    JobController,
    JobRecord,
    JobResult,
    JobState,
)


def _wait_done(done: list[JobRecord], timeout: float = 3.0) -> JobRecord:
    deadline = time.monotonic() + timeout
    while len(done) < 1 and time.monotonic() < deadline:
        time.sleep(0.01)
    assert done, "on_done never fired"
    return done[0]


def test_job_controller_runs_and_completes():
    ctrl = JobController()
    done: list[JobRecord] = []
    ctrl.run("test", lambda _ev: JobResult(True, "ok"), on_done=done.append)
    record = _wait_done(done)
    assert record.state == JobState.DONE
    assert record.result is not None
    assert record.result.ok
    assert not ctrl.is_busy()


def test_job_controller_cancel():
    ctrl = JobController()
    done: list[JobRecord] = []

    def slow_worker(cancel: threading.Event) -> JobResult:
        for _ in range(400):
            if cancel.is_set():
                return JobResult(False, "Cancelled")
            time.sleep(0.005)
        return JobResult(True, "ok")

    job_id = ctrl.run("cancel-test", slow_worker, on_done=done.append)
    time.sleep(0.03)
    ctrl.cancel(job_id)
    record = _wait_done(done)
    assert record.state in (JobState.CANCELLED, JobState.FAILED, JobState.DONE)
    assert not ctrl.is_busy()


def test_job_controller_rejects_double_start():
    ctrl = JobController()
    gate = threading.Event()

    def block(_cancel: threading.Event) -> JobResult:
        gate.wait(timeout=2.0)
        return JobResult(True, "ok")

    ctrl.run("first", block)
    time.sleep(0.05)
    with pytest.raises(RuntimeError, match="Another job is running"):
        ctrl.run("second", lambda _ev: JobResult(True, "ok"))
    gate.set()


def test_witness_refresh():
    from rust_engine_mcp.aps_ux_async_witness import refresh_aps_ux_async_witness

    assert refresh_aps_ux_async_witness()
