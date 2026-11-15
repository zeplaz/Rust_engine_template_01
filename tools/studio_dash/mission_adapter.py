"""P4 — thin adapters so mission / world-builder tools reuse studio_dash HTML.

Call ``emit_mission_dashboard`` or ``emit_worldgen_dashboard`` with a JSON-serializable
dict; writes under ``debug_runs/studio_dash/``.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

try:
    from .render_report import write_report
except ImportError:
    from render_report import write_report  # type: ignore


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _out_dir() -> Path:
    p = _repo_root() / "debug_runs" / "studio_dash"
    p.mkdir(parents=True, exist_ok=True)
    return p


def emit_mission_dashboard(
    payload: dict[str, Any],
    *,
    name: str = "mission_report",
) -> Path:
    """World / mission tooling → HTML. Does not invent a second stack."""
    data = dict(payload)
    data.setdefault("title", "Mission / ops report")
    data.setdefault("tool", "mission_adapter")
    json_path = _out_dir() / f"{name}.json"
    html_path = _out_dir() / f"{name}.html"
    json_path.write_text(json.dumps(data, indent=2), encoding="utf-8")
    return write_report(data, html_path, title=str(data.get("title")))


def emit_worldgen_dashboard(
    payload: dict[str, Any],
    *,
    name: str = "worldgen_report",
) -> Path:
    data = dict(payload)
    data.setdefault("title", "World builder report")
    data.setdefault("tool", "worldgen_adapter")
    json_path = _out_dir() / f"{name}.json"
    html_path = _out_dir() / f"{name}.html"
    json_path.write_text(json.dumps(data, indent=2), encoding="utf-8")
    return write_report(data, html_path, title=str(data.get("title")))


def cross_link_doc() -> str:
    return (
        "Mission/world-builder dashboards use tools/studio_dash/render_report.py "
        "via mission_adapter.emit_* — same kit as APS DashboardPanel. "
        "See src/dev/plan_artist_studio_expansion_v1.md §P4."
    )
