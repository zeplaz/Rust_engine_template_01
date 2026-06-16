"""APS-UX-DENSITY-001 — collapsible accordion sections."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk


class CollapsibleSection(ttk.Frame):
    """Chevron header toggles body visibility; default collapsed for density."""

    def __init__(
        self,
        master: tk.Misc,
        title: str,
        *,
        expanded: bool = False,
        padding: int = 4,
    ) -> None:
        super().__init__(master)
        self._title = title
        self._expanded = expanded
        self._padding = padding

        head = ttk.Frame(self)
        head.pack(fill=tk.X)
        self._head_btn = ttk.Button(
            head,
            text=self._header_text(),
            style="Aps.Toolbar.TButton",
            command=self._toggle,
        )
        self._head_btn.pack(side=tk.LEFT, anchor=tk.W, fill=tk.X, expand=True)

        self.body = ttk.Frame(self, padding=padding)
        self._sync_body()

    def _header_text(self) -> str:
        chevron = "▾" if self._expanded else "▸"
        return f"{chevron}  {self._title}"

    def set_title(self, title: str) -> None:
        self._title = title
        self._head_btn.configure(text=self._header_text())

    def _toggle(self) -> None:
        self._expanded = not self._expanded
        self._head_btn.configure(text=self._header_text())
        self._sync_body()

    def _sync_body(self) -> None:
        if self._expanded:
            if not self.body.winfo_ismapped():
                self.body.pack(fill=tk.BOTH, expand=True, pady=(4, 0))
        else:
            self.body.pack_forget()

    @property
    def is_expanded(self) -> bool:
        return self._expanded
