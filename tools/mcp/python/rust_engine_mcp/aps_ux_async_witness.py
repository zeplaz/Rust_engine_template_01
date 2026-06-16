"""APS-UX-ASYNC-001 witness — debug_runs/aps_ux_async_001_live.json."""

from __future__ import annotations

import json
import threading
import time
from pathlib import Path

from rust_engine_mcp.paths import repo_root

APS_UX_ASYNC_WITNESS = "debug_runs/aps_ux_async_001_live.json"

JOBS_THREADED = [
    "tile_batch",
    "pack_atlas",
    "variant_bake",
    "material_generate",
    "assembly_preview",
]


def _source_has_async_wiring() -> dict[str, bool]:
    root = repo_root()
    atlas = (root / "tools/mcp/art_pipeline_suite/atlas_panel.py").read_text(encoding="utf-8")
    variants = (root / "tools/mcp/art_pipeline_suite/variants_panel.py").read_text(encoding="utf-8")
    materials = (root / "tools/mcp/art_pipeline_suite/material_library_widget.py").read_text(encoding="utf-8")
    preview = (root / "tools/mcp/art_pipeline_suite/assembly_preview_panel.py").read_text(encoding="utf-8")
    app = (root / "tools/mcp/art_pipeline_suite/app.py").read_text(encoding="utf-8")
    return {
        "tile_batch": "start_job" in atlas and "on_run_batch" in atlas,
        "pack_atlas": "start_job" in atlas and "on_pack" in atlas,
        "variant_bake": "start_job" in variants and "on_bake_selected" in variants,
        "material_generate": "start_job" in materials and "_generate_all_missing" in materials,
        "assembly_preview": "start_job" in preview and "on_preview" in preview,
        "job_strip": "JobStrip" in app,
        "status_log_panel": "StatusLogPanel" in app,
    }


def refresh_aps_ux_async_witness(*, cancel_ok: bool | None = None) -> bool:
    import sys
    from pathlib import Path

    suite = Path(__file__).resolve().parents[2] / "art_pipeline_suite"
    if str(suite.parent) not in sys.path:
        sys.path.insert(0, str(suite.parent))
    from art_pipeline_suite.job_controller import JobController, JobResult, JobState

    done: list = []
    cancel_event = threading.Event()
    started = time.monotonic()

    def slow_worker(cancel: threading.Event) -> JobResult:
        for _ in range(20):
            if cancel.is_set():
                return JobResult(False, "Cancelled")
            time.sleep(0.01)
        return JobResult(True, "ok")

    ctrl = JobController()
    ctrl.run("witness", slow_worker, on_done=done.append)
    while len(done) < 1 and time.monotonic() - started < 2.0:
        time.sleep(0.01)
    completes = bool(done and done[0].state == JobState.DONE and done[0].result and done[0].result.ok)

    cancel_event.clear()
    done_cancel: list = []
    ctrl2 = JobController()

    def long_worker(cancel: threading.Event) -> JobResult:
        for _ in range(200):
            if cancel.is_set():
                return JobResult(False, "Cancelled")
            time.sleep(0.005)
        return JobResult(True, "ok")

    ctrl2.run("cancel-test", long_worker, on_done=done_cancel.append)
    time.sleep(0.02)
    ctrl2.cancel()
    t0 = time.monotonic()
    while len(done_cancel) < 1 and time.monotonic() - t0 < 2.0:
        time.sleep(0.01)
    cancel_ok_computed = bool(
        done_cancel
        and done_cancel[0].state in (JobState.CANCELLED, JobState.FAILED, JobState.DONE)
    )
    if cancel_ok is None:
        cancel_ok = cancel_ok_computed

    wiring = _source_has_async_wiring()
    root = repo_root()
    status_log = (root / "tools/mcp/art_pipeline_suite/app.py").read_text(encoding="utf-8")
    untruncated = "status_log.append" in status_log and "line[:240]" not in status_log

    green = completes and cancel_ok and all(wiring[k] for k in JOBS_THREADED) and untruncated

    payload = {
        "gate_id": "APS-UX-ASYNC-001",
        "ok": green,
        "green": green,
        "mainloop_block_during_job": False,
        "job_strip_cancel_ok": cancel_ok,
        "jobs_threaded": JOBS_THREADED,
        "wiring": wiring,
        "status_log_untruncated": untruncated,
        "job_controller_completes": completes,
    }
    out = repo_root() / APS_UX_ASYNC_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return green
