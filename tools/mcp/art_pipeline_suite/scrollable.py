"""Scrollable container for APS tabs — vertical (+ optional horizontal) scroll."""

from __future__ import annotations

import sys
import tkinter as tk
from tkinter import ttk


class ScrollableFrame(ttk.Frame):
    """Canvas + interior frame; mouse wheel scrolls when pointer is over the widget tree."""

    def __init__(self, master: tk.Misc, *, enable_horizontal: bool = False) -> None:
        super().__init__(master)
        self._enable_horizontal = enable_horizontal
        self._wheel_bound: set[str] = set()
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
        self._bind_wheel_recursive(self)

    def _bind_wheel_recursive(self, widget: tk.Misc) -> None:
        wid = str(widget)
        if wid in self._wheel_bound:
            return
        self._wheel_bound.add(wid)
        widget.bind("<MouseWheel>", self._on_mousewheel, add="+")
        widget.bind("<Shift-MouseWheel>", self._on_shift_mousewheel, add="+")
        widget.bind("<Button-4>", self._on_mousewheel_linux, add="+")
        widget.bind("<Button-5>", self._on_mousewheel_linux, add="+")
        for child in widget.winfo_children():
            self._bind_wheel_recursive(child)

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
        self._bind_wheel_recursive(self.interior)

    def _on_canvas_configure(self, event) -> None:
        if not self._enable_horizontal:
            # Skip redundant width writes (each one re-lays-out the interior).
            if self._canvas.itemcget(self._interior_id, "width") != str(event.width):
                self._canvas.itemconfigure(self._interior_id, width=event.width)

    def _on_mousewheel(self, event) -> None:
        if sys.platform == "darwin":
            delta = event.delta
        else:
            delta = event.delta // 120 if event.delta else 0
        if delta:
            self._canvas.yview_scroll(int(-delta), "units")

    def _on_shift_mousewheel(self, event) -> None:
        if not self._enable_horizontal:
            return
        if sys.platform == "darwin":
            delta = event.delta
        else:
            delta = event.delta // 120 if event.delta else 0
        if delta:
            self._canvas.xview_scroll(int(-delta), "units")

    def _on_mousewheel_linux(self, event) -> None:
        if event.num == 4:
            self._canvas.yview_scroll(-1, "units")
        elif event.num == 5:
            self._canvas.yview_scroll(1, "units")
