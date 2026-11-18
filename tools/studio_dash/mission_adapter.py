"""CQ-75 — thin emit helpers over shared studio_dash.render_report.

Play matrix dashboards stay in tools/sim_review/score_dump.py.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .render_report import write_report


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
    data = dict(payload)
    data.setdefault("title", "Mission / ops report")
    data.setdefault("tool", "mission_adapter")
    json_path = _out_dir() / f"{name}.json"
    html_path = _out_dir() / f"{name}.html"
    json_path.write_text(json.dumps(data, indent=2), encoding="utf-8")
    return write_report(data, html_path, title=str(data.get("title")))


def emit_frame_qc_dashboard(
    payload: dict[str, Any],
    *,
    name: str = "frame_qc",
) -> Path:
    from .frame_qc import frame_qc_view

    data = frame_qc_view(dict(payload))
    json_path = _out_dir() / f"{name}.json"
    html_path = _out_dir() / f"{name}.html"
    json_path.write_text(json.dumps(data, indent=2), encoding="utf-8")
    return write_report(data, html_path, title=str(data.get("title")))


def emit_massing_qc_dashboard(
    payload: dict[str, Any],
    *,
    name: str = "massing_qc",
) -> Path:
    from .massing_qc import massing_qc_view

    data = massing_qc_view(dict(payload))
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
        "(emit_* in mission_adapter). Play review = tools/sim_review/score_dump.py. "
        "Massing QC = rust_engine_mcp.massing_qc (CQ-75/76)."
    )
