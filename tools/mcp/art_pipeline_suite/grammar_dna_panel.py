"""BUILD-READ-GRAMMAR-v0-002 — ARCH-DNA preset + β sliders for Assembly tab."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk
from typing import Any, Callable

from rust_engine_mcp import arch_build_grammar
from rust_engine_mcp.aps_grammar_labels import human_label

from .aps_theme import FONT_SMALL
from .aps_tooltips import bind_aps_tooltip


class GrammarDnaPanel(ttk.LabelFrame):
    def __init__(
        self,
        master: tk.Misc,
        *,
        on_change: Callable[[], None] | None = None,
    ) -> None:
        super().__init__(master, text="Massing pressure (advanced)", padding=6)
        self._on_change = on_change
        self._beta_vars: dict[str, tk.DoubleVar] = {}
        self._dna_value_vars: dict[str, tk.StringVar] = {}
        self._building = False
        self._build()

    def _build(self) -> None:
        top = ttk.Frame(self)
        top.pack(fill=tk.X, pady=2)
        self.enabled_var = tk.BooleanVar(value=True)
        enable_cb = ttk.Checkbutton(
            top,
            text="Store ARCH-DNA + β in snapshot",
            variable=self.enabled_var,
            command=self._notify_change,
        )
        enable_cb.pack(side=tk.LEFT)
        bind_aps_tooltip(enable_cb, "asm_grammar_dna_enable")

        preset_row = ttk.Frame(self)
        preset_row.pack(fill=tk.X, pady=4)
        ttk.Label(preset_row, text="DNA preset").pack(side=tk.LEFT)
        presets = arch_build_grammar.list_preset_ids()
        default = arch_build_grammar.default_preset_id()
        self.preset_var = tk.StringVar(value=default)
        self.preset_combo = ttk.Combobox(
            preset_row,
            textvariable=self.preset_var,
            width=34,
            values=presets,
            state="readonly",
        )
        self.preset_combo.pack(side=tk.LEFT, padx=6)
        self.preset_combo.bind("<<ComboboxSelected>>", self._on_preset_selected)
        bind_aps_tooltip(self.preset_combo, "asm_grammar_dna_preset")

        dna_frame = ttk.LabelFrame(self, text="ARCH-DNA (read-only from preset)", padding=4)
        dna_frame.pack(fill=tk.X, pady=4)
        for i, key in enumerate(arch_build_grammar.DNA_KEYS):
            row = i // 5
            col = i % 5
            cell = ttk.Frame(dna_frame)
            cell.grid(row=row, column=col, sticky=tk.W, padx=4, pady=2)
            ttk.Label(cell, text=f"{key}:", width=2, font=(FONT_SMALL[0], FONT_SMALL[1], "bold")).pack(side=tk.LEFT)
            var = tk.StringVar(value="—")
            self._dna_value_vars[key] = var
            lbl = ttk.Label(cell, textvariable=var, width=14, font=FONT_SMALL)
            lbl.pack(side=tk.LEFT)
            bind_aps_tooltip(lbl, f"asm_grammar_dna_{key.lower()}")

        beta_frame = ttk.LabelFrame(self, text="Pressure field β (0–1)", padding=4)
        beta_frame.pack(fill=tk.X, pady=4)
        for i, key in enumerate(arch_build_grammar.BETA_KEYS):
            row = ttk.Frame(beta_frame)
            row.pack(fill=tk.X, pady=1)
            label = arch_build_grammar.BETA_LABELS.get(key, key)
            ttk.Label(row, text=label, width=28).pack(side=tk.LEFT)
            var = tk.DoubleVar(value=0.5)
            self._beta_vars[key] = var
            scale = ttk.Scale(
                row,
                from_=0.0,
                to=1.0,
                orient=tk.HORIZONTAL,
                variable=var,
                command=lambda _v, k=key: self._on_beta_moved(k),
            )
            scale.pack(side=tk.LEFT, fill=tk.X, expand=True, padx=4)
            readout = ttk.Label(row, width=5, font=("Consolas", 9))
            readout.pack(side=tk.LEFT)
            var.trace_add("write", lambda *_a, v=var, lbl=readout: lbl.configure(text=f"{v.get():.2f}"))
            bind_aps_tooltip(scale, f"asm_grammar_beta_{key.removeprefix('beta_')}")

        self.set_preset(self.preset_var.get())

    def is_enabled(self) -> bool:
        return bool(self.enabled_var.get())

    def _notify_change(self) -> None:
        if self._building or not self._on_change:
            return
        self._on_change()

    def _on_beta_moved(self, _key: str) -> None:
        self._notify_change()

    def _on_preset_selected(self, _event: tk.Event | None = None) -> None:
        self.set_preset(self.preset_var.get())

    def set_preset(self, preset_id: str) -> None:
        self._building = True
        try:
            preset = arch_build_grammar.load_preset(preset_id)
            self.preset_var.set(str(preset["preset_id"]))
            arch_dna = preset.get("arch_dna") if isinstance(preset.get("arch_dna"), dict) else {}
            for key, var in self._dna_value_vars.items():
                raw = arch_dna.get(key, "")
                var.set(human_label(str(raw)) if raw else "—")
            pressure = arch_build_grammar.normalize_pressure_field(
                preset.get("pressure_field") if isinstance(preset.get("pressure_field"), dict) else None
            )
            for key, var in self._beta_vars.items():
                var.set(pressure.get(key, 0.0))
        finally:
            self._building = False
        self._notify_change()

    def set_from_snapshot(self, snapshot: dict[str, Any] | None) -> None:
        snap = snapshot if isinstance(snapshot, dict) else {}
        extracted = arch_build_grammar.extract_from_snapshot(snap)
        self._building = True
        try:
            has_dna = bool(extracted.get("arch_dna"))
            self.enabled_var.set(has_dna or self.is_enabled())
            preset_id = str(extracted.get("preset_id") or arch_build_grammar.default_preset_id())
            self.preset_var.set(preset_id)
            arch_dna = extracted.get("arch_dna") if isinstance(extracted.get("arch_dna"), dict) else {}
            if arch_dna:
                for key, var in self._dna_value_vars.items():
                    raw = arch_dna.get(key, "")
                    var.set(human_label(str(raw)) if raw else "—")
            else:
                preset = arch_build_grammar.load_preset(preset_id)
                arch_dna = preset.get("arch_dna") if isinstance(preset.get("arch_dna"), dict) else {}
                for key, var in self._dna_value_vars.items():
                    raw = arch_dna.get(key, "")
                    var.set(human_label(str(raw)) if raw else "—")
            pressure = extracted.get("pressure_field") if isinstance(extracted.get("pressure_field"), dict) else {}
            if pressure and any(pressure.values()):
                for key, var in self._beta_vars.items():
                    var.set(float(pressure.get(key, var.get())))
            elif not has_dna:
                self.set_preset(preset_id)
        finally:
            self._building = False

    def get_state(self) -> dict[str, Any]:
        return {
            "preset_id": self.preset_var.get().strip(),
            "pressure_field": arch_build_grammar.normalize_pressure_field(
                {key: var.get() for key, var in self._beta_vars.items()}
            ),
            "include": self.is_enabled(),
        }
