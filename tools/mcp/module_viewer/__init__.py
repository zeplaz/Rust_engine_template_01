"""Module Kit Viewer — browse promoted MCP GLBs without Bevy."""

from __future__ import annotations

__all__ = ["run_app"]


def run_app() -> None:
    """Launch Art Pipeline Suite (lazy import avoids circular load with art_pipeline_suite)."""
    from .app import run_app as _run_app

    _run_app()
