"""DES-APS-GEN-STEP-EXPOSURE-001 — read-only generation trace + artist approve strip."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk
from typing import Any, Callable

from rust_engine_mcp.aps_grammar_labels import human_label

from .aps_theme import COLOR_MUTED, FONT_SMALL
from .aps_tooltips import bind_aps_tooltip
from .state import SuiteState


class GenerationTraceStrip(ttk.LabelFrame):
    """Snapshot lineage: archetype · district · seed · grammar steps · approve."""

    def __init__(
        self,
        master: tk.Misc,
        state: SuiteState,
        *,
        get_snapshot: Callable[[], dict[str, Any] | None],
        on_go_assembly: Callable[[], None] | None = None,
    ) -> None:
        super().__init__(master, text="Generation trace", padding=6)
        self.state = state
        self._get_snapshot = get_snapshot
        self._on_go_assembly = on_go_assembly

        self._summary_var = tk.StringVar(value="No assembly snapshot yet.")
        ttk.Label(self, textvariable=self._summary_var, wraplength=520, justify=tk.LEFT).pack(
            anchor=tk.W, fill=tk.X
        )

        steps = ttk.Frame(self)
        steps.pack(anchor=tk.W, fill=tk.X, pady=(4, 0))
        self._step_vars: list[tk.StringVar] = []
        for _ in range(4):
            var = tk.StringVar(value="")
            self._step_vars.append(var)
            ttk.Label(steps, textvariable=var, font=FONT_SMALL, foreground=COLOR_MUTED).pack(anchor=tk.W)

        row = ttk.Frame(self)
        row.pack(anchor=tk.W, pady=(6, 0))
        self._approved_var = tk.BooleanVar(value=False)
        approve = ttk.Checkbutton(
            row,
            text="Approve snapshot for variant / bake parent",
            variable=self._approved_var,
            command=self._on_approve_toggle,
        )
        approve.pack(side=tk.LEFT, padx=(0, 8))
        bind_aps_tooltip(approve, "gen_trace_approve")
        go_btn = ttk.Button(row, text="Edit on Assembly", command=self._go_assembly)
        go_btn.pack(side=tk.LEFT)
        bind_aps_tooltip(go_btn, "gen_trace_edit_assembly")

    def _go_assembly(self) -> None:
        if self._on_go_assembly:
            self._on_go_assembly()

    def _on_approve_toggle(self) -> None:
        self.state.assembly_generation_approved = bool(self._approved_var.get())

    def refresh(self) -> None:
        snap = self._get_snapshot()
        if not snap:
            self._summary_var.set("No assembly snapshot — Generate on Assembly tab first.")
            for var in self._step_vars:
                var.set("")
            self._approved_var.set(False)
            self.state.assembly_generation_approved = False
            return

        archetype = str(
            snap.get("archetype_id")
            or (snap.get("grammar") or {}).get("archetype")
            or snap.get("building_type")
            or "—"
        )
        district = str(snap.get("district_style") or (snap.get("grammar") or {}).get("district_style") or "—")
        seed = snap.get("seed")
        assembly_id = str(snap.get("assembly_id") or self.state.assembly_id or "—")
        self._summary_var.set(
            f"{human_label(archetype)} · {human_label(district)} · seed {seed} · {assembly_id}"
        )

        grammar = snap.get("grammar") or {}
        chain = snap.get("semantic_tags") or []
        chain_hits = sum(1 for t in chain if str(t).startswith("chain:"))
        placements = snap.get("module_placements") or snap.get("placements") or []
        p0 = self.state.assembly_p0_passed

        self._step_vars[0].set(
            f"{'✓' if grammar or archetype != '—' else '○'} Footprint / grammar massing"
        )
        self._step_vars[1].set(
            f"{'✓' if placements else '○'} Module resolve ({len(placements)} placements)"
        )
        self._step_vars[2].set(
            f"{'✓' if chain_hits else '○'} Rule chain ({chain_hits} chain tags)"
        )
        if p0 is True:
            self._step_vars[3].set("✓ P0 ship check passed")
        elif p0 is False:
            self._step_vars[3].set("✗ P0 ship check failed — fix on Assembly")
        else:
            self._step_vars[3].set("○ P0 ship check not run")

        approved = bool(getattr(self.state, "assembly_generation_approved", False))
        self._approved_var.set(approved)

    def reset_approval(self) -> None:
        self._approved_var.set(False)
        self.state.assembly_generation_approved = False
