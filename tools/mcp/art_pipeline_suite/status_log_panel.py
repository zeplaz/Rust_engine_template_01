"""APS-UX-ASYNC-001 — persistent scrollable status log (no 240-char truncate)."""

from __future__ import annotations

import time
import tkinter as tk
from tkinter import ttk

from .aps_scroll import attach_wheel_area, text_yscroll
from .aps_theme import COLOR_INPUT_BG, FONT_MONO


class StatusLogPanel(ttk.Frame):
    def __init__(self, master: tk.Misc, *, height: int = 6) -> None:
        super().__init__(master)
        self._text = tk.Text(
            self,
            height=height,
            wrap=tk.WORD,
            font=FONT_MONO,
            bg=COLOR_INPUT_BG,
            relief=tk.FLAT,
            borderwidth=1,
            highlightthickness=1,
            highlightbackground="#c8ccd4",
        )
        scroll = ttk.Scrollbar(self, orient=tk.VERTICAL, command=self._text.yview)
        self._text.configure(yscrollcommand=scroll.set, state=tk.NORMAL)
        self._text.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        scroll.pack(side=tk.RIGHT, fill=tk.Y)
        attach_wheel_area(
            self._text,
            on_scroll_y=text_yscroll(self._text),
            area_id=f"aps-status-log-{id(self)}",
        )

    def append(self, line: str, *, timestamp: bool = True) -> None:
        prefix = f"{time.strftime('%H:%M:%S')} " if timestamp else ""
        self._text.insert(tk.END, f"{prefix}{line}\n")
        self._text.see(tk.END)
