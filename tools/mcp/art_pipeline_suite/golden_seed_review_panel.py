"""APSR-Q3 — Golden-seed review panel on Assembly tab."""

from __future__ import annotations

import tkinter as tk
from tkinter import messagebox, ttk
from typing import Any, Callable

from rust_engine_mcp import golden_seed_review
from rust_engine_mcp.aps_grammar_labels import human_label

from .aps_inline_feedback import apply_status_atom, set_inline_status
from .aps_theme import FONT_SMALL, VALIDATION_BANNER_MIN_PX
from .aps_tooltips import bind_aps_tooltip


class GoldenSeedReviewPanel(ttk.LabelFrame):
    """Browse BQ-Q3 golden seeds; approve/reject writes operator rubric rows."""

    def __init__(
        self,
        master: tk.Misc,
        *,
        on_load_snapshot: Callable[[dict[str, Any]], None],
        on_log: Callable[[str], None] | None = None,
    ) -> None:
        super().__init__(master, text="Golden seed review (BQ-Q3)", padding=6)
        self._on_load_snapshot = on_load_snapshot
        self._on_log = on_log or (lambda _m: None)
        self._seeds: list[dict[str, Any]] = golden_seed_review.load_golden_seeds()
        self._status_var = tk.StringVar(value="Select a golden seed to load into Assembly preview.")
        holder = ttk.Frame(self, height=VALIDATION_BANNER_MIN_PX)
        holder.pack(fill=tk.X)
        holder.pack_propagate(False)
        self._status_lbl = ttk.Label(holder, textvariable=self._status_var, font=FONT_SMALL, wraplength=720)
        self._status_lbl.pack(anchor=tk.W, fill=tk.X)

        row = ttk.Frame(self)
        row.pack(fill=tk.X, pady=(4, 0))
        self._list = tk.Listbox(row, height=4, exportselection=False, font=FONT_SMALL)
        scroll = ttk.Scrollbar(row, orient=tk.VERTICAL, command=self._list.yview)
        self._list.configure(yscrollcommand=scroll.set)
        self._list.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        scroll.pack(side=tk.RIGHT, fill=tk.Y)
        self._list.bind("<<ListboxSelect>>", lambda _e: self._on_select())

        actions = ttk.Frame(self)
        actions.pack(anchor=tk.W, pady=(4, 0))
        load_btn = ttk.Button(actions, text="Load seed", command=self._load_selected)
        load_btn.pack(side=tk.LEFT, padx=(0, 4))
        bind_aps_tooltip(load_btn, "asm_load")
        approve_btn = ttk.Button(actions, text="Approve", command=lambda: self._verdict("approve"))
        approve_btn.pack(side=tk.LEFT, padx=2)
        bind_aps_tooltip(approve_btn, "gen_trace_approve")
        reject_btn = ttk.Button(actions, text="Reject", command=lambda: self._verdict("reject"))
        reject_btn.pack(side=tk.LEFT, padx=2)
        bind_aps_tooltip(reject_btn, "asm_validate")

        self._populate_list()

    def _populate_list(self) -> None:
        self._list.delete(0, tk.END)
        for entry in self._seeds:
            label = (
                f"{human_label(str(entry.get('archetype_id')))} · "
                f"{human_label(str(entry.get('district_style')))} · seed {entry.get('seed')}"
            )
            self._list.insert(tk.END, label)

    def _selected_entry(self) -> dict[str, Any] | None:
        sel = self._list.curselection()
        if not sel:
            return None
        idx = int(sel[0])
        if idx >= len(self._seeds):
            return None
        return self._seeds[idx]

    def _on_select(self) -> None:
        entry = self._selected_entry()
        if not entry:
            return
        set_inline_status(
            self._status_lbl,
            self._status_var,
            f"Selected {entry.get('archetype_id')} seed {entry.get('seed')} — Load seed to preview.",
            ok=None,
        )

    def _load_selected(self) -> None:
        entry = self._selected_entry()
        if not entry:
            set_inline_status(self._status_lbl, self._status_var, "Select a golden seed first.", ok=False)
            return
        apply_status_atom(self._status_lbl, self._status_var, "working", detail="Loading golden seed…")
        try:
            snap = golden_seed_review.generate_snapshot_for_seed(entry, write=False)
            self._on_load_snapshot(snap)
            set_inline_status(
                self._status_lbl,
                self._status_var,
                f"Loaded {snap.get('assembly_id')} — review then Approve/Reject.",
                ok=True,
            )
            self._on_log(f"golden seed loaded: {golden_seed_review.seed_key(entry)}")
        except Exception as exc:
            set_inline_status(self._status_lbl, self._status_var, f"Load failed: {exc}", ok=False)

    def _verdict(self, verdict: str) -> None:
        entry = self._selected_entry()
        if not entry:
            set_inline_status(self._status_lbl, self._status_var, "Select a seed before verdict.", ok=False)
            return
        row = golden_seed_review.record_seed_verdict(entry, verdict=verdict)
        ok = verdict == "approve"
        set_inline_status(
            self._status_lbl,
            self._status_var,
            f"{verdict.title()} recorded for seed {entry.get('seed')}.",
            ok=ok,
        )
        self._on_log(f"golden seed {verdict}: {row.get('seed_key')}")
        if verdict == "reject":
            messagebox.showwarning(
                "Golden seed rejected",
                "Reject recorded in operator rubric rows. Fix grammar/kit before re-approving.",
            )
