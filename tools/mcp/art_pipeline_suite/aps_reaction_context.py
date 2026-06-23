"""Reaction-territory preview status copy — APS P0-C."""

from __future__ import annotations


def reaction_preview_status_line(event_id: str, cell_x: int, cell_y: int) -> str:
    label = event_id if event_id and event_id != "base_session" else "base_session"
    return f"Generating reaction preview — {label} · cell ({cell_x},{cell_y})…"
