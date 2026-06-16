# APS-UX-ASYNC-001 — job strip + threaded MCP `v1`

| Field | Value |
|:---|:---|
| **Slice ID** | **APS-UX-ASYNC-001** |
| **Phase** | 1 of [`plan_aps_ux_polish_program_v1.md`](plan_aps_ux_polish_program_v1.md) |
| **Rules** | [`aps_ux_professional_polish_rules_v1.md`](aps_ux_professional_polish_rules_v1.md) |
| **Owner** | `@coder-mcp` |
| **Est** | 1–1.5 days |
| **Status** | **READY** |

---

## Problem (on disk today)

| File | Issue |
|:---|:---|
| [`atlas_panel.py`](../../tools/mcp/art_pipeline_suite/atlas_panel.py) | `on_run_batch` / `on_pack` call `run_tile_batch` / `pack_tile_folder` **synchronously** on UI thread |
| [`variants_panel.py`](../../tools/mcp/art_pipeline_suite/variants_panel.py) | Bake path blocks until MCP returns |
| [`material_library_widget.py`](../../tools/mcp/art_pipeline_suite/material_library_widget.py) | Generate profiles blocks UI |
| [`assembly_preview_panel.py`](../../tools/mcp/art_pipeline_suite/assembly_preview_panel.py) | Preview worker call blocks |
| [`app.py`](../../tools/mcp/art_pipeline_suite/app.py) | `on_pack_atlas` / `on_bake_variants` chain blocking handlers |
| [`app.py`](../../tools/mcp/art_pipeline_suite/app.py) `_log` | Truncates to 240 chars — hides job progress |

**Symptom:** Window freezes 10s–minutes during Pack / Tile batch / Bake / Generate. Artist cannot switch tabs or cancel.

---

## Deliverables

| # | File | Purpose |
|:---:|:---|:---|
| 1 | `tools/mcp/art_pipeline_suite/job_controller.py` | Thread pool + cancel + progress callbacks |
| 2 | `tools/mcp/art_pipeline_suite/job_strip.py` | UI: `⟳ {label}… {step}` + Cancel |
| 3 | `tools/mcp/art_pipeline_suite/status_log_panel.py` | Persistent scrollable log (replaces 240-char strip) |
| 4 | Wire `app.py` | Mount job strip + status log; inject `JobController` |
| 5 | Migrate handlers | Atlas, Variants bake, Material generate, Assembly preview |
| 6 | `tools/mcp/python/tests/test_aps_ux_async_001.py` | JobController unit tests (no Tk headless) |
| 7 | `debug_runs/aps_ux_async_001_live.json` | Witness |

---

## API shapes

### `job_controller.py`

```python
from __future__ import annotations

import threading
import time
import uuid
from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Callable


class JobState(str, Enum):
    QUEUED = "queued"
    RUNNING = "running"
    CANCELLING = "cancelling"
    DONE = "done"
    CANCELLED = "cancelled"
    FAILED = "failed"


@dataclass(frozen=True)
class JobResult:
    ok: bool
    summary: str
    detail: str = ""
    data: dict[str, Any] | None = None


@dataclass
class JobRecord:
    job_id: str
    label: str
    state: JobState = JobState.QUEUED
    step_index: int = 0
    step_count: int = 1
    cancellable: bool = True
    started_at: float = field(default_factory=time.monotonic)
    finished_at: float | None = None
    result: JobResult | None = None
    error: str | None = None


ProgressCallback = Callable[[JobRecord], None]
DoneCallback = Callable[[JobRecord], None]

# Worker runs off UI thread; receives cancel event.
JobWorker = Callable[[threading.Event], JobResult]


class JobController:
    """Single active job (APS v1). Queue later if needed."""

    def __init__(self) -> None:
        self._lock = threading.Lock()
        self._active: JobRecord | None = None
        self._cancel = threading.Event()
        self._thread: threading.Thread | None = None

    def is_busy(self) -> bool:
        with self._lock:
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
        *,
        step_count: int = 1,
        cancellable: bool = True,
        on_progress: ProgressCallback | None = None,
        on_done: DoneCallback | None = None,
    ) -> str:
        """Start job; returns job_id. Raises RuntimeError if busy."""
        ...

    def set_step(self, job_id: str, step_index: int, step_label: str | None = None) -> None:
        """Worker thread calls via `after(0, …)` wrapper only for UI; or thread-safe queue."""
        ...

    def cancel(self, job_id: str | None = None) -> None:
        """Signal cooperative cancel; subprocess jobs terminate child."""
        ...

    def _finish(self, record: JobRecord) -> None:
        ...
```

**Threading rule:** Worker runs in `threading.Thread(daemon=True)`. All Tk mutations via `root.after(0, fn)`.

### UI poll loop (`app.py`)

```python
def _poll_jobs(self) -> None:
    job = self.jobs.active_job()
    self.job_strip.sync(job)
    if job and job.state in (JobState.RUNNING, JobState.CANCELLING):
        self.after(100, self._poll_jobs)  # 100ms — meets inline tier
```

### `job_strip.py`

```python
class JobStrip(ttk.Frame):
    """Hidden when idle. Shown when JobState.RUNNING."""

    def sync(self, job: JobRecord | None) -> None:
        # visible = job and job.state in (RUNNING, CANCELLING)
        # label: f"⟳ {job.label}…"
        # step: f"{job.step_index}/{job.step_count}" if step_count > 1
        # cancel_btn: command=controller.cancel
```

Mount **between** `PipelineStatusBar` and notebook:

```python
self.job_strip = JobStrip(self, self.jobs)
self.job_strip.pack(fill=tk.X, padx=8)
# pack_forget() when idle
```

### Button busy states

```python
def _run_job_button(self, btn: ttk.Button, label: str, worker: JobWorker, command_fn):
    if self.jobs.is_busy():
        self._log("Another job is running — wait or Cancel")
        return
    btn.configure(state=tk.DISABLED, text=f"⟳ {label}…")
    def done(record: JobRecord):
        btn.configure(state=tk.NORMAL, text=label)
        ...
    self.jobs.run(label, worker, on_done=lambda r: self.after(0, lambda: done(r)))
```

---

## Jobs to thread (Phase 1 scope)

| Job label | Entry point | Worker wraps | Steps | Cancel |
|:---|:---|:---|:---:|:---:|
| **Tile batch** | `AtlasPanel.on_run_batch` | `run_tile_batch(path)` | 1 | Yes¹ |
| **Pack atlas** | `AtlasPanel.on_pack` | `pack_tile_folder(folder)` | 1 | Yes¹ |
| **Bake variant** | `VariantsPanel` bake | existing bake MCP | 1 | Yes¹ |
| **Generate materials** | `material_library_widget` | profile generate MCP | 1 | Event |
| **Preview assembly** | `assembly_preview_panel` | preview worker | 1 | Event |
| **Flow: Pack atlas** | `app.on_pack_atlas` | delegates to atlas `on_pack` async | 1 | Yes¹ |

¹ **Cancel:** set `threading.Event`; if worker spawns subprocess, store `Popen` handle on `JobRecord` and `terminate()` on cancel. Document partial-write folders in log.

**Out of scope Phase 1:** lod0 batch, keyframe addon (rare); grammar iterate (fast JSON).

---

## Status log panel

Replace `_build_status_log` one-liner with:

```python
class StatusLogPanel(ttk.Frame):
    def append(self, line: str) -> None:
        # Text widget, autoscroll, NO truncation
        # Optional prefix: time.strftime("%H:%M:%S")
```

`ArtPipelineSuiteApp._log`:

```python
def _log(self, line: str) -> None:
    self.state.append_log(line)
    self.status_log.append(line)
    self.status_summary_var.set(line[:80])  # summary only
    self.pipeline_status.refresh()
```

---

## Migration pattern (per handler)

**Before:**

```python
def on_pack(self) -> None:
    code, log = pack_tile_folder(Path(folder))
    self._log(log)
    messagebox.showinfo("Pack", ...)
```

**After:**

```python
def on_pack(self) -> None:
    folder = self.folder_var.get().strip()
    if not folder:
        self._inline_hint("Choose PNG folder first.")
        return

    def worker(cancel: threading.Event) -> JobResult:
        if cancel.is_set():
            return JobResult(False, "Cancelled")
        code, log = pack_tile_folder(Path(folder))
        return JobResult(code == 0, "Pack OK" if code == 0 else "Pack failed", detail=log)

    self._jobs.run("Pack atlas", worker, on_done=self._on_pack_done)
```

`_on_pack_done` runs on UI thread: refresh preview, set inline QC, append summary to log — **no modal**.

---

## Acceptance

| # | Criterion |
|:---:|:---|
| 1 | During tile batch or pack, user can **switch notebook tabs** without freeze |
| 2 | Job strip visible within **100 ms** of click; shows **Cancel** |
| 3 | Cancel stops cooperative job; UI returns to idle ≤ 2 s |
| 4 | Status log retains **full** CLI output (not 240-char truncate) |
| 5 | `pytest tools/mcp/python/tests/test_aps_ux_async_001.py` green |
| 6 | Witness `aps_ux_async_001_live.json`: `mainloop_block_during_job: false`, `job_strip_cancel_ok: true` |

---

## Witness schema (`debug_runs/aps_ux_async_001_live.json`)

```json
{
  "gate_id": "APS-UX-ASYNC-001",
  "ok": true,
  "green": true,
  "mainloop_block_during_job": false,
  "job_strip_cancel_ok": true,
  "jobs_threaded": [
    "tile_batch",
    "pack_atlas",
    "variant_bake",
    "material_generate",
    "assembly_preview"
  ],
  "status_log_untruncated": true
}
```

Populate via headless `JobController` test + one scripted smoke note in witness writer (optional Tk smoke manual).

---

## Test sketch (`test_aps_ux_async_001.py`)

```python
def test_job_controller_runs_and_completes():
    ctrl = JobController()
    done: list[JobRecord] = []
    ctrl.run("test", lambda ev: JobResult(True, "ok"), on_done=done.append)
    # join with timeout
    assert done[0].result.ok

def test_job_controller_cancel():
    ...

def test_job_controller_rejects_double_start():
    ...
```

---

## Orchestrator handoff

```text
@coder-mcp APS-UX-ASYNC-001

Implement per docs/archive/2026-06-src-dev/plans/plan_aps_ux_async_001_exec_v1.md
Rules: docs/archive/2026-06-src-dev/plans/aps_ux_professional_polish_rules_v1.md

Files: job_controller.py, job_strip.py, status_log_panel.py; wire app.py + atlas_panel + variants_panel + material_library_widget + assembly_preview_panel

Witness: debug_runs/aps_ux_async_001_live.json
Tests: test_aps_ux_async_001.py

Next slice after green: APS-UX-NONBLOCK-001 (modal migration)
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | Phase 1 exec brief with JobController API |
