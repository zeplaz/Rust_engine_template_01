"""Console entry: module-viewer (after pip install -e tools/mcp/python)."""

from __future__ import annotations

import sys
from pathlib import Path


def main() -> None:
    mcp_root = Path(__file__).resolve().parents[2]  # tools/mcp
    for p in (mcp_root, mcp_root / "python"):
        s = str(p)
        if s not in sys.path:
            sys.path.insert(0, s)
    from art_pipeline_suite.app import run_app

    run_app()
