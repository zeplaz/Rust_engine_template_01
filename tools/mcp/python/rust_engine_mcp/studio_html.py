"""CQ-75/76 — HTML write helper for MCP without scattering sys.path inserts.

Delegates to tools/studio_dash/render_report.py (shared kit). Sim review
(`tools/sim_review/score_dump.py`) remains the play-matrix dashboard.
"""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Any


def _ensure_studio_dash() -> None:
    tools = Path(__file__).resolve().parents[3]  # tools/
    if str(tools) not in sys.path:
        sys.path.insert(0, str(tools))


def write_studio_report(
    data: dict[str, Any], out_path: Path, *, title: str | None = None
) -> Path:
    _ensure_studio_dash()
    from studio_dash.render_report import write_report

    return write_report(data, Path(out_path), title=title)
