"""Shared inline status helpers for APS-UX-NONBLOCK-001 / APS-UX-POLISH-001."""

from __future__ import annotations

import tkinter as tk

from .aps_theme import COLOR_ACCENT, COLOR_FAIL, COLOR_MUTED, COLOR_PASS, COLOR_WARN


def validation_foreground(ok: bool | None) -> str:
    if ok is True:
        return COLOR_PASS
    if ok is False:
        return COLOR_FAIL
    return COLOR_WARN if ok is None else COLOR_MUTED


def set_inline_status(
    label: tk.Label | tk.Widget,
    var: tk.StringVar,
    text: str,
    *,
    ok: bool | None = None,
) -> None:
    var.set(text)
    label.configure(foreground=validation_foreground(ok))
