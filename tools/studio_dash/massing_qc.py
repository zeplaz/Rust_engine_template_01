"""Massing QC — re-export from rust_engine_mcp (CQ-76)."""

from __future__ import annotations

import sys
from pathlib import Path

_PY = Path(__file__).resolve().parents[1] / "mcp" / "python"
if str(_PY) not in sys.path:
    sys.path.insert(0, str(_PY))

from rust_engine_mcp.massing_qc import floors_histogram, massing_qc_view  # noqa: E402

__all__ = ["floors_histogram", "massing_qc_view"]
