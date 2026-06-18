"""APS-UX-ASYNC-001 — visible job progress strip with Cancel."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk

from .aps_theme import COLOR_TEXT_SUBTLE


class JobStrip(ttk.Frame):
    """Hidden when idle. Shown when a job is RUNNING or CANCELLING."""

    def __init__(self, master: tk.Misc, jobs: JobController) -> None:
        super().__init__(master, padding=(8, 4))
        self._jobs = jobs
        self._label_var = tk.StringVar(value="")
        self._step_var = tk.StringVar(value="")
        ttk.Label(self, textvariable=self._label_var, font=("Segoe UI", 9, "bold")).pack(
            side=tk.LEFT, padx=(0, 8)
        )
        ttk.Label(self, textvariable=self._step_var, foreground=COLOR_TEXT_SUBTLE).pack(side=tk.LEFT, padx=(0, 8))
        self._cancel_btn = ttk.Button(self, text="Cancel", command=self._on_cancel)
        self._cancel_btn.pack(side=tk.RIGHT)

    def _on_cancel(self) -> None:
        job = self._jobs.active_job()
        if job and job.cancellable:
            self._jobs.cancel(job.job_id)

    def sync(self, job: JobRecord | None) -> None:
        if job is None or job.state not in (JobState.RUNNING, JobState.CANCELLING, JobState.QUEUED):
            self.pack_forget()
            return
        if not self.winfo_ismapped():
            self.pack(fill=tk.X, padx=8, pady=(0, 2))
        prefix = "⟳" if job.state != JobState.CANCELLING else "⏹"
        self._label_var.set(f"{prefix} {job.label}…")
        if job.step_count > 1:
            step = job.step_label or str(job.step_index)
            self._step_var.set(f"{step} ({job.step_index}/{job.step_count})")
        elif job.step_label:
            self._step_var.set(job.step_label)
        else:
            self._step_var.set("")
        state = tk.DISABLED if job.state == JobState.CANCELLING or not job.cancellable else tk.NORMAL
        self._cancel_btn.configure(state=state)
