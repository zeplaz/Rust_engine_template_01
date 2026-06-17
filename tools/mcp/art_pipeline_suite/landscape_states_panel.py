"""APS-E3 — Landscape States tab (v2 labels · catalog axes · not Variants reuse)."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk
from typing import Any

from rust_engine_mcp.veg_catalog_loader import (
    catalog_axis_summary,
    catalog_validator_report,
    load_vegetation_variant_catalog,
)

from .aps_inline_feedback import set_inline_status
from .aps_theme import FONT_HINT, FONT_SMALL, FONT_UI
from .aps_tooltips import bind_aps_tooltip
from .landscape_extract_parity_panel import LandscapeExtractParityPanel
from .landscape_state_labels import (
    REGROWTH_MACRO_ROWS,
    SUCCESSION_STAGE_ROWS,
    burn_preview_rows,
    combobox_display_values,
    enum_from_ui_label,
    inline_hint,
    resolver_plain_label,
    atlas_slot_label,
    status_display,
    status_foreground,
    ui_label_for_enum,
)
from .metadata_flow_panel import MetadataFlowPanel
from .state import SuiteState


class _LabeledEnumCombobox:
    """Readonly combobox — shows UI labels; get_enum/set_enum use schema enums."""

    def __init__(
        self,
        master: tk.Misc,
        rows: tuple[Any, ...],
        *,
        width: int = 18,
    ) -> None:
        self._rows = rows
        self._label_by_enum = {r.enum: r.ui_label for r in rows}
        self._enum_by_label = {r.ui_label: r.enum for r in rows}
        self.widget = ttk.Combobox(
            master,
            values=combobox_display_values(rows),
            state="readonly",
            width=width,
            font=FONT_UI,
        )

    def grid(self, **kwargs: Any) -> None:
        self.widget.grid(**kwargs)

    def get_enum(self) -> str:
        raw = self.widget.get().strip()
        return self._enum_by_label.get(raw) or enum_from_ui_label(raw, rows=self._rows) or raw

    def set_enum(self, enum: str) -> None:
        self.widget.set(self._label_by_enum.get(enum, ui_label_for_enum(enum)))


class LandscapeStatesPanel(ttk.Frame):
    """State matrix — succession + burn + topology rows from vegetation catalog."""

    def __init__(self, master: tk.Misc, state: SuiteState, *, on_log) -> None:
        super().__init__(master, padding=4)
        self.state = state
        self._on_log = on_log
        self._validator_lbl: ttk.Label | None = None
        self._axis_var = tk.StringVar(value="")
        self._succession_combo: _LabeledEnumCombobox | None = None
        self._regrowth_combo: _LabeledEnumCombobox | None = None
        self._burn_preview_combo: ttk.Combobox | None = None
        self._burn_preview_enums: list[str] = []
        self._burn_preview_displays: list[str] = []
        self._build()
        self.refresh_from_catalog()

    def _build(self) -> None:
        self.metadata_flow = MetadataFlowPanel(self, context="landscape_states")
        self.metadata_flow.pack(fill=tk.X, pady=(0, 6))
        head = ttk.Frame(self)
        head.pack(fill=tk.X)
        ttk.Label(
            head,
            text="States — growth stages & fire",
            font=("Segoe UI", 9, "bold"),
        ).pack(side=tk.LEFT)
        ttk.Button(head, text="Validate catalog", command=self._validate_catalog).pack(
            side=tk.RIGHT, padx=4
        )
        ttk.Label(
            self,
            textvariable=self._axis_var,
            font=FONT_HINT,
            foreground="#555",
            wraplength=720,
        ).pack(anchor=tk.W, pady=(0, 4))

        axis_row = ttk.Frame(self)
        axis_row.pack(fill=tk.X, pady=(0, 6))
        ttk.Label(axis_row, text="Succession stage", font=FONT_UI).grid(row=0, column=0, sticky=tk.W, padx=(0, 6))
        self._succession_combo = _LabeledEnumCombobox(axis_row, SUCCESSION_STAGE_ROWS, width=16)
        self._succession_combo.grid(row=0, column=1, sticky=tk.W, padx=(0, 12))
        bind_aps_tooltip(self._succession_combo.widget, "state_succession_axis")

        ttk.Label(axis_row, text="Regrowth macro", font=FONT_UI).grid(row=0, column=2, sticky=tk.W, padx=(0, 6))
        self._regrowth_combo = _LabeledEnumCombobox(axis_row, REGROWTH_MACRO_ROWS, width=16)
        self._regrowth_combo.grid(row=0, column=3, sticky=tk.W, padx=(0, 12))
        bind_aps_tooltip(self._regrowth_combo.widget, "state_regrowth_axis")

        ttk.Label(axis_row, text="Preview frame", font=FONT_UI).grid(row=0, column=4, sticky=tk.W, padx=(0, 6))
        self._burn_preview_combo = ttk.Combobox(axis_row, state="readonly", width=22, font=FONT_UI)
        self._burn_preview_combo.grid(row=0, column=5, sticky=tk.W)
        bind_aps_tooltip(self._burn_preview_combo, "state_burn_frames")

        cols = ("state_key", "label", "status", "atlas_slot")
        self._tree = ttk.Treeview(self, columns=cols, show="headings", height=12)
        for col, heading, width in (
            ("state_key", "State key", 200),
            ("label", "Label", 280),
            ("status", "Status", 140),
            ("atlas_slot", "Atlas slot", 180),
        ):
            self._tree.heading(col, text=heading)
            self._tree.column(col, width=width)
        self._tree.pack(fill=tk.BOTH, expand=True, pady=4)
        self._tree.tag_configure("status_pass", foreground=status_foreground("valid"))
        self._tree.tag_configure("status_fail", foreground=status_foreground("catalog_fail"))
        self._tree.tag_configure("status_warn", foreground=status_foreground("await_grammar"))
        self._tree.tag_configure("status_muted", foreground=status_foreground("blocked"))

        self._hint_var = tk.StringVar(value="")
        self._hint_lbl = ttk.Label(self, textvariable=self._hint_var, font=FONT_SMALL)
        self._hint_lbl.pack(anchor=tk.W)
        self._validation = tk.StringVar(value="")
        self._validator_lbl = ttk.Label(self, textvariable=self._validation, font=FONT_SMALL)
        self._validator_lbl.pack(anchor=tk.W, pady=(4, 0))

        self.extract_parity = LandscapeExtractParityPanel(self, on_log=self._on_log)
        self.extract_parity.pack(fill=tk.X, pady=(8, 0))

    def _sync_axis_comboboxes(self, axis: dict[str, Any]) -> None:
        burn_count = int(axis.get("burn_frame_count") or 8)
        preview = burn_preview_rows(burn_count)
        self._burn_preview_enums = [enum for enum, _label in preview]
        self._burn_preview_displays = [_label for _enum, _label in preview]
        if self._burn_preview_combo is not None:
            self._burn_preview_combo.configure(values=self._burn_preview_displays)
            if self._burn_preview_displays:
                mid = min(3, len(self._burn_preview_displays) - 1)
                self._burn_preview_combo.set(self._burn_preview_displays[mid])

        stages = list(axis.get("succession_stages") or [])
        phases = list(axis.get("regrowth_macro_phases") or [])
        if self._succession_combo is not None and stages:
            self._succession_combo.set_enum(str(stages[0]))
        if self._regrowth_combo is not None and phases:
            self._regrowth_combo.set_enum(str(phases[0]))

    def _row_internal_status(
        self,
        *,
        has_preset: bool,
        grammar_ok: bool,
        catalog_ok: bool | None,
    ) -> str:
        if not has_preset:
            return "blocked"
        if not grammar_ok:
            return "await_grammar"
        if catalog_ok is False:
            return "catalog_fail"
        if catalog_ok is True:
            return "catalog_ok"
        return "validate"

    def _validate_catalog(self) -> None:
        report = catalog_validator_report()
        ok = bool(report.get("green"))
        self.state.landscape_catalog_validate_ok = ok
        if ok:
            msg = (
                f"Catalog PASS — {report.get('entry_count')} entries · "
                f"{report.get('veg_burn_count')} burn frames"
            )
        else:
            msg = f"Catalog FAIL — {report.get('error') or report.get('status')}"
        set_inline_status(self._validator_lbl, self._validation, msg, ok=ok if ok else False)
        self._on_log(f"vegetation catalog validate · {report.get('status')}")
        self.refresh_from_catalog()

    def refresh_from_catalog(self) -> None:
        self.refresh_from_state()

    def refresh_from_state(self) -> None:
        axis = catalog_axis_summary()
        self._axis_var.set(
            f"Burn frames: {axis.get('burn_frame_count')} · "
            f"Succession: {len(axis.get('succession_stages') or [])} · "
            f"Regrowth: {len(axis.get('regrowth_macro_phases') or [])} · "
            f"Catalog rows: {axis.get('entry_count')}"
        )
        self._sync_axis_comboboxes(axis)

        catalog = load_vegetation_variant_catalog()
        entries = [e for e in (catalog.get("entries") or []) if isinstance(e, dict)]

        self._tree.delete(*self._tree.get_children())
        has_preset = bool(self.state.selected_landscape_preset_id)
        grammar_ok = self.state.landscape_grammar_saved or self.state.landscape_preset_validate_ok
        catalog_ok = self.state.landscape_catalog_validate_ok
        internal = self._row_internal_status(
            has_preset=has_preset,
            grammar_ok=grammar_ok,
            catalog_ok=catalog_ok,
        )
        status_text, _ok = status_display(internal, catalog_ok=catalog_ok)
        if internal == "catalog_ok":
            tag = "status_pass"
        elif internal == "catalog_fail":
            tag = "status_fail"
        elif internal in ("await_grammar", "scaffold"):
            tag = "status_warn"
        else:
            tag = "status_muted"

        for entry in sorted(entries, key=lambda e: str(e.get("variant_key") or "")):
            key = str(entry.get("variant_key") or "")
            self._tree.insert(
                "",
                tk.END,
                values=(
                    key,
                    resolver_plain_label(entry),
                    status_text,
                    atlas_slot_label(entry),
                ),
                tags=(tag,),
            )

        self._hint_var.set(
            inline_hint(
                has_preset=has_preset,
                grammar_ok=grammar_ok,
                catalog_ok=catalog_ok,
            )
        )
        if hasattr(self, "extract_parity"):
            self.extract_parity.refresh_parity()

    def mark_states_ready(self) -> None:
        report = catalog_validator_report()
        if not report.get("green"):
            set_inline_status(
                self._validator_lbl,
                self._validation,
                "Catalog FAIL — validate before bake",
                ok=False,
            )
            self._on_log("landscape states · bake blocked (catalog not valid)")
            return
        self.state.landscape_catalog_validate_ok = True
        self.state.landscape_states_ready = True
        self.refresh_from_catalog()
        self._on_log("landscape states · bake ready (catalog validated)")

    def selected_burn_preview_enum(self) -> str | None:
        if self._burn_preview_combo is None or not self._burn_preview_enums:
            return None
        label = self._burn_preview_combo.get().strip()
        for enum, display in zip(self._burn_preview_enums, self._burn_preview_displays, strict=True):
            if label == display:
                return enum
        idx = self._burn_preview_combo.current()
        if 0 <= idx < len(self._burn_preview_enums):
            return self._burn_preview_enums[idx]
        return None
