"""APSR-S1 — synchronous pub-sub for SuiteState changes (Tk-safe via scheduler)."""

from __future__ import annotations

from collections import defaultdict
from typing import Any, Callable


class EventBus:
    """Tiny synchronous event bus; optional ``scheduler`` wraps delivery (e.g. ``root.after(0, fn)``)."""

    def __init__(self, scheduler: Callable[[Callable[[], None]], None] | None = None) -> None:
        self._subs: dict[str, list[Callable[[dict[str, Any]], None]]] = defaultdict(list)
        self._scheduler = scheduler or (lambda fn: fn())

    def subscribe(self, event: str, handler: Callable[[dict[str, Any]], None]) -> None:
        self._subs[event].append(handler)

    def publish(self, event: str, payload: dict[str, Any], *, sync: bool = False) -> None:
        for handler in list(self._subs.get(event, [])):
            if sync or self._scheduler is None:
                handler(payload)
            else:
                self._scheduler(lambda h=handler, p=payload: h(p))

    def clear(self) -> None:
        self._subs.clear()
