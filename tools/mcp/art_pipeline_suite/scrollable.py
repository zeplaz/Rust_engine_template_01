"""Scrollable container for APS tabs — vertical (+ optional horizontal) scroll."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk

from .aps_scroll import attach_wheel_area, canvas_xscroll, canvas_yscroll


class ScrollableFrame(ttk.Frame):
    """Canvas + interior frame; mouse wheel scrolls when pointer is over the widget tree."""

    def __init__(self, master: tk.Misc, *, enable_horizontal: bool = False) -> None:
        super().__init__(master)
        self._enable_horizontal = enable_horizontal
        # B4 — coalesce scrollregion recomputes to one idle pass per burst of
        # <Configure> events so a tall tab does not retear on every child resize.
        self._scrollregion_job: str | None = None
        self._last_scrollregion: tuple[int, int, int, int] | None = None

        self._canvas = tk.Canvas(self, highlightthickness=0, borderwidth=0)
        self._vscroll = ttk.Scrollbar(self, orient=tk.VERTICAL, command=self._canvas.yview)
        self._hscroll: ttk.Scrollbar | None = None
        if enable_horizontal:
            self._hscroll = ttk.Scrollbar(self, orient=tk.HORIZONTAL, command=self._canvas.xview)

        self.interior = ttk.Frame(self._canvas)
        self._interior_id = self._canvas.create_window((0, 0), window=self.interior, anchor=tk.NW)

        self._canvas.configure(yscrollcommand=self._vscroll.set)
        self._vscroll.pack(side=tk.RIGHT, fill=tk.Y)
        if self._hscroll is not None:
            self._canvas.configure(xscrollcommand=self._hscroll.set)
            self._hscroll.pack(side=tk.BOTTOM, fill=tk.X)
        self._canvas.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)

        self.interior.bind("<Configure>", self._on_interior_configure)
        self._canvas.bind("<Configure>", self._on_canvas_configure)
        # P7 Wave-2 S — one wheel owner via aps_scroll (no per-widget recursive rebind).
        attach_wheel_area(
            self._canvas,
            self.interior,
            on_scroll_y=canvas_yscroll(self._canvas),
            on_scroll_x=canvas_xscroll(self._canvas) if enable_horizontal else None,
            area_id=f"aps-scrollable-{id(self)}",
        )

    def _on_interior_configure(self, _event=None) -> None:
        # Debounce: schedule one scrollregion update + wheel rebind at idle rather
        # than recomputing on every nested <Configure>, which is the source of the
        # scroll tearing/flicker on Windows Tk.
        if self._scrollregion_job is not None:
            return
        self._scrollregion_job = self.after_idle(self._apply_scrollregion)

    def _apply_scrollregion(self) -> None:
        self._scrollregion_job = None
        try:
            bbox = self._canvas.bbox("all")
        except tk.TclError:
            return
        if bbox is None:
            return
        # Only reconfigure when the region actually changed — a no-op
        # scrollregion write still forces the canvas to redraw and tear.
        if bbox != self._last_scrollregion:
            self._last_scrollregion = bbox
            self._canvas.configure(scrollregion=bbox)

    def _on_canvas_configure(self, event) -> None:
        if not self._enable_horizontal:
            # Skip redundant width writes (each one re-lays-out the interior).
            if self._canvas.itemcget(self._interior_id, "width") != str(event.width):
                self._canvas.itemconfigure(self._interior_id, width=event.width)

