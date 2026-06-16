"""APS-UX-SCROLL-002 — nested scroll routing (one wheel owner; debounced scrollregion)."""

from __future__ import annotations

import sys
import tkinter as tk
from collections.abc import Callable
from typing import Any

_WHEEL_HANDLERS: dict[str, Callable[[Any], str | None]] = {}
_WHEEL_STACK: list[str] = []
_WHEEL_ROOT: tk.Misc | None = None


def _wheel_delta(event: Any) -> int:
    if sys.platform == "darwin":
        delta = int(event.delta)
    else:
        delta = int(event.delta // 120) if getattr(event, "delta", 0) else 0
    if delta == 0 and getattr(event, "num", None) == 4:
        return 1
    if delta == 0 and getattr(event, "num", None) == 5:
        return -1
    return delta


def init_aps_scroll(root: tk.Misc) -> None:
    """Install global wheel dispatch once on the app root."""
    global _WHEEL_ROOT
    _WHEEL_ROOT = root
    root.bind_all("<MouseWheel>", _dispatch_wheel, add="+")
    root.bind_all("<Button-4>", _dispatch_wheel, add="+")
    root.bind_all("<Button-5>", _dispatch_wheel, add="+")


def _dispatch_wheel(event: Any) -> str | None:
    for area_id in reversed(_WHEEL_STACK):
        handler = _WHEEL_HANDLERS.get(area_id)
        if handler is None:
            continue
        result = handler(event)
        if result == "break":
            return "break"
    return None


def attach_wheel_area(
    *widgets: tk.Misc,
    on_scroll_y: Callable[[int], None],
    on_scroll_x: Callable[[int], None] | None = None,
    area_id: str | None = None,
) -> str:
    """Register a scroll region; innermost region under the pointer wins."""
    if not widgets:
        raise ValueError("attach_wheel_area requires at least one widget")
    area_id = area_id or f"aps-wheel-{widgets[0]}"

    def _scroll(event: Any) -> str:
        delta = _wheel_delta(event)
        if not delta:
            return "break"
        if on_scroll_x and (getattr(event, "state", 0) & 0x1):
            on_scroll_x(delta)
        else:
            on_scroll_y(delta)
        return "break"

    def _enter(_event: Any) -> None:
        if area_id in _WHEEL_STACK:
            _WHEEL_STACK.remove(area_id)
        _WHEEL_STACK.append(area_id)

    def _leave(_event: Any) -> None:
        if area_id in _WHEEL_STACK:
            if _WHEEL_STACK[-1] == area_id:
                _WHEEL_STACK.pop()
            else:
                _WHEEL_STACK.remove(area_id)

    _WHEEL_HANDLERS[area_id] = _scroll
    for widget in widgets:
        widget.bind("<Enter>", _enter, add="+")
        widget.bind("<Leave>", _leave, add="+")
    return area_id


def canvas_yscroll(canvas: tk.Canvas, *, steps: int = 3) -> Callable[[int], None]:
    def _scroll(delta: int) -> None:
        canvas.yview_scroll(int(-delta * steps), "units")

    return _scroll


def canvas_xscroll(canvas: tk.Canvas, *, steps: int = 3) -> Callable[[int], None]:
    def _scroll(delta: int) -> None:
        canvas.xview_scroll(int(-delta * steps), "units")

    return _scroll


def text_yscroll(text: tk.Text, *, steps: int = 3) -> Callable[[int], None]:
    def _scroll(delta: int) -> None:
        text.yview_scroll(int(-delta * steps), "units")

    return _scroll


def bind_debounced_scrollregion(canvas: tk.Canvas, *interiors: tk.Misc) -> None:
    """Coalesce scrollregion updates to one idle pass (reduces layout blips)."""
    job_attr = "_aps_scrollregion_job"
    bound_attr = "_aps_scrollregion_bound"

    def _apply() -> None:
        setattr(canvas, job_attr, None)
        try:
            bbox = canvas.bbox("all")
            if bbox:
                canvas.configure(scrollregion=bbox)
        except tk.TclError:
            pass

    def _schedule(_event: Any = None) -> None:
        if getattr(canvas, job_attr, None):
            return

        def _idle() -> None:
            _apply()

        canvas.after_idle(_idle)
        setattr(canvas, job_attr, True)

    if not getattr(canvas, bound_attr, False):
        for interior in interiors:
            interior.bind("<Configure>", _schedule, add="+")
        setattr(canvas, bound_attr, True)
    _schedule()
