"""APS P0-A — Variants tab live preview (4-state strip + debounced assembly preview)."""

from __future__ import annotations

import copy
import threading
import tkinter as tk
from tkinter import ttk
from typing import Any, Callable

try:
    from PIL import Image, ImageTk
except ImportError:  # pragma: no cover
    Image = None  # type: ignore[misc, assignment]
    ImageTk = None  # type: ignore[misc, assignment]

from rust_engine_mcp import assembly_preview
from rust_engine_mcp.paths import repo_root

from rust_engine_mcp.reaction_territory import preview_visual_states_for_entry

from .aps_inline_feedback import apply_status_atom
from .preview_state_display import (
    apply_preview_photo,
    configure_preview_label,
    load_png_thumbnail,
    set_preview_status,
)
from .aps_preview_variant_state import (
    VARIANT_STATES,
    VariantVisualState,
    merge_variant_entry_layers,
    variant_entry_to_visual_state,
    variant_state_label,
)
from .aps_reaction_context import reaction_preview_status_line
from .aps_theme import COLOR_MUTED, FONT_SMALL, PREVIEW_THUMB_MD

DEBOUNCE_MS = 300


class VariantsPreviewPanel(ttk.LabelFrame):
    def __init__(
        self,
        master: tk.Misc,
        *,
        on_log: Callable[[str], None] | None = None,
        get_snapshot: Callable[[], dict[str, Any] | None] | None = None,
        get_variant_entry: Callable[[], dict[str, Any] | None] | None = None,
        get_reaction_event_id: Callable[[], str | None] | None = None,
        get_context_line: Callable[[], str] | None = None,
        get_draft_dirty: Callable[[], bool] | None = None,
    ) -> None:
        super().__init__(master, text="Variant preview", padding=6)
        self._on_log = on_log or (lambda _line: None)
        self._get_snapshot = get_snapshot or (lambda: None)
        self._get_variant_entry = get_variant_entry or (lambda: None)
        self._get_reaction_event_id = get_reaction_event_id or (lambda: None)
        self._get_context_line = get_context_line or (lambda: "")
        self._get_draft_dirty = get_draft_dirty or (lambda: False)
        self._visual_state: VariantVisualState = "clean"
        self._active_states: tuple[VariantVisualState, ...] = VARIANT_STATES
        self._debounce_after: str | None = None
        self._preview_cancel: threading.Event | None = None
        self._preview_thread: threading.Thread | None = None
        self._thumb_photo: ImageTk.PhotoImage | None = None
        self._build()

    def _build(self) -> None:
        ttk.Label(
            self,
            text="Select a variant row — preview uses assembly snapshot + your layer controls (live draft).",
            wraplength=420,
            font=FONT_SMALL,
            foreground=COLOR_MUTED,
        ).pack(anchor=tk.W, pady=(0, 4))

        self._context_var = tk.StringVar(value="")
        self._context_lbl = ttk.Label(
            self,
            textvariable=self._context_var,
            wraplength=420,
            font=FONT_SMALL,
            justify=tk.LEFT,
        )
        self._context_lbl.pack(anchor=tk.W, pady=(0, 2))

        self._draft_var = tk.StringVar(value="")
        ttk.Label(
            self,
            textvariable=self._draft_var,
            wraplength=420,
            font=FONT_SMALL,
            foreground=COLOR_MUTED,
        ).pack(anchor=tk.W, pady=(0, 4))

        chip_row = ttk.Frame(self)
        chip_row.pack(anchor=tk.W, pady=2)
        self._chip_row = chip_row
        self._chip_vars: dict[VariantVisualState, tk.StringVar] = {}
        self._chip_labels: dict[VariantVisualState, ttk.Label] = {}
        for state in VARIANT_STATES:
            var = tk.StringVar(value=variant_state_label(state))
            self._chip_vars[state] = var
            chip = ttk.Label(chip_row, textvariable=var, relief=tk.GROOVE, padding=(6, 2))
            chip.pack(side=tk.LEFT, padx=2)
            chip.bind("<Button-1>", lambda _e, s=state: self._on_chip_click(s))
            self._chip_labels[state] = chip

        body = ttk.Frame(self)
        body.pack(fill=tk.X, pady=4)
        self._thumb_label = tk.Label(body, relief=tk.SUNKEN)
        self._thumb_label.pack(side=tk.LEFT, padx=(0, 8))
        configure_preview_label(
            self._thumb_label,
            "empty",
            detail="Select a variant",
            width=PREVIEW_THUMB_MD,
            height=PREVIEW_THUMB_MD,
        )

        meta = ttk.Frame(body)
        meta.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        self._status_var = tk.StringVar(value="")
        self._status_lbl = ttk.Label(meta, textvariable=self._status_var, wraplength=280, justify=tk.LEFT)
        self._status_lbl.pack(anchor=tk.W)
        self._variant_var = tk.StringVar(value="")
        ttk.Label(meta, textvariable=self._variant_var, font=FONT_SMALL, foreground=COLOR_MUTED).pack(anchor=tk.W)

    def _set_status(self, text: str, *, ok: bool | None = None) -> None:
        set_preview_status(self._status_lbl, self._status_var, text, ok=ok)

    def _apply_active_states(self, states: list[str]) -> None:
        allowed = tuple(s for s in VARIANT_STATES if s in states) or VARIANT_STATES
        self._active_states = allowed
        for state in VARIANT_STATES:
            chip = self._chip_labels.get(state)
            if chip is None:
                continue
            if state in allowed:
                chip.state(["!disabled"])
                chip.pack(side=tk.LEFT, padx=2)
            else:
                chip.pack_forget()
        if self._visual_state not in allowed:
            self._visual_state = allowed[0]
        self._highlight_chip(self._visual_state)

    def _highlight_chip(self, state: VariantVisualState) -> None:
        for key, var in self._chip_vars.items():
            label = variant_state_label(key)
            if key not in self._active_states:
                continue
            if key == state:
                var.set(f"▸ {label}")
            else:
                var.set(label)

    def _on_chip_click(self, state: VariantVisualState) -> None:
        if state not in self._active_states:
            return
        self._visual_state = state
        self._highlight_chip(state)
        self.queue_preview()

    def sync_visual_state_from_entry(self, entry: dict[str, Any] | None) -> None:
        if not entry:
            return
        allowed = preview_visual_states_for_entry(entry)
        self._apply_active_states(allowed)
        state = variant_entry_to_visual_state(entry)
        if state not in self._active_states:
            state = self._active_states[0]
        self._visual_state = state
        self._highlight_chip(state)
        key = str(entry.get("variant_key") or "—")
        event = entry.get("reaction_event_id")
        if event:
            self._variant_var.set(f"variant: {key} · event: {event}")
        else:
            self._variant_var.set(f"variant: {key}")

    def queue_preview(self, *, force: bool = False) -> None:
        if self._debounce_after is not None:
            try:
                self.after_cancel(self._debounce_after)
            except tk.TclError:
                pass
            self._debounce_after = None
        delay = 0 if force else DEBOUNCE_MS
        self._debounce_after = self.after(delay, self._start_preview_job)

    def run_preview_now(self) -> None:
        self.queue_preview(force=True)

    def _cancel_in_flight(self) -> None:
        if self._preview_cancel is not None:
            self._preview_cancel.set()

    def _start_preview_job(self) -> None:
        self._debounce_after = None
        snapshot = self._get_snapshot()
        entry = self._get_variant_entry()
        if not snapshot:
            configure_preview_label(
                self._thumb_label,
                "empty",
                detail="No assembly snapshot",
                hint="Generate on Assembly tab",
                width=PREVIEW_THUMB_MD,
                height=PREVIEW_THUMB_MD,
            )
            self._set_status("Generate an assembly snapshot first (Assembly tab).", ok=None)
            return
        if not entry:
            configure_preview_label(
                self._thumb_label,
                "empty",
                detail="Select a variant",
                width=PREVIEW_THUMB_MD,
                height=PREVIEW_THUMB_MD,
            )
            self._set_status("Select a variant row to preview.", ok=None)
            self._context_var.set("")
            self._draft_var.set("")
            return

        context = self._get_context_line()
        self._context_var.set(context)
        if self._get_draft_dirty():
            self._draft_var.set("Draft — not saved on row. Apply layers to commit.")
        else:
            self._draft_var.set("")

        self.sync_visual_state_from_entry(entry)
        self._cancel_in_flight()
        cancel = threading.Event()
        self._preview_cancel = cancel
        event_id = str(entry.get("reaction_event_id") or self._get_reaction_event_id() or "base_session")
        cell_x, cell_y = 0, 0
        anchor = entry.get("tag_anchor") or {}
        if "cell_x" in anchor and "cell_y" in anchor:
            cell_x, cell_y = int(anchor["cell_x"]), int(anchor["cell_y"])
        else:
            from rust_engine_mcp.reaction_territory import reaction_preview_cell

            cell_x, cell_y = reaction_preview_cell(snapshot, entry)
        status_line = reaction_preview_status_line(event_id, cell_x, cell_y)
        apply_status_atom(
            self._status_lbl,
            self._status_var,
            "working",
            detail=status_line,
        )
        configure_preview_label(
            self._thumb_label,
            "loading",
            detail=status_line,
            width=PREVIEW_THUMB_MD,
            height=PREVIEW_THUMB_MD,
        )

        payload = copy.deepcopy(snapshot)
        merged = merge_variant_entry_layers(payload, entry, self._visual_state)
        variant_key = str(entry.get("variant_key") or "?")

        def worker() -> None:
            if cancel.is_set():
                return
            try:
                result = assembly_preview.preview_assembly_from_dict(
                    merged,
                    open_browser=False,
                    serve_seconds=0.0,
                )
            except Exception as exc:  # noqa: BLE001
                self.after(0, lambda: self._on_preview_failed(str(exc), cancel))
                return
            if cancel.is_set():
                return
            self.after(0, lambda: self._on_preview_done(result, variant_key, cancel))

        thread = threading.Thread(target=worker, daemon=True, name="aps-variant-preview")
        self._preview_thread = thread
        thread.start()

    def _on_preview_failed(self, message: str, cancel: threading.Event) -> None:
        if self._preview_cancel is not cancel:
            return
        configure_preview_label(
            self._thumb_label,
            "error",
            detail="Preview failed",
            hint=message[:48],
            width=PREVIEW_THUMB_MD,
            height=PREVIEW_THUMB_MD,
        )
        self._set_status(f"Preview failed: {message}", ok=False)
        self._on_log(f"variant preview error: {message}")

    def _on_preview_done(self, result: dict[str, Any], variant_key: str, cancel: threading.Event) -> None:
        if self._preview_cancel is not cancel:
            return
        mode = str(result.get("mode") or "?")
        loaded = int(result.get("modules_loaded") or 0)
        missing = result.get("missing_glb") or []
        self._load_thumbnail(str(result.get("png") or ""))
        if missing:
            self._set_status(
                f"{variant_key} · {mode} · {loaded} modules · {len(missing)} missing GLB",
                ok=None,
            )
        else:
            self._set_status(
                f"{variant_key} · {mode} · {loaded} modules · preview ready",
                ok=True,
            )
        self._on_log(f"variant preview {variant_key} · {mode} · modules={loaded}")

    def _load_thumbnail(self, png_rel: str) -> None:
        photo = load_png_thumbnail(
            self._thumb_label,
            png_rel,
            width=PREVIEW_THUMB_MD,
            height=PREVIEW_THUMB_MD,
            empty_detail="No thumbnail",
            error_detail="Thumbnail missing",
            near_black_detail="Preview blank",
            near_black_hint="check GLB paths",
        )
        self._thumb_photo = photo
