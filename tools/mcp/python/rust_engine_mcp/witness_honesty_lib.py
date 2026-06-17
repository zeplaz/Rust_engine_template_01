"""Bridge to tools/orchestrator/scripts/witness_honesty_lib.py (single engine)."""

from __future__ import annotations

import sys
from pathlib import Path

_SCRIPTS = Path(__file__).resolve().parents[3] / "orchestrator" / "scripts"
if str(_SCRIPTS) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS))

from witness_honesty_lib import (  # noqa: E402
    OPS_WITNESS_REL,
    build_integrity_cache,
    classify_honest_gate_v2,
    honest_gate_v1,
    refresh_mcp_witness_integrity_ops_witness,
    run_post_build_hook,
    scan_queue_integrity,
    scan_witness_honesty,
)

__all__ = [
    "OPS_WITNESS_REL",
    "build_integrity_cache",
    "classify_honest_gate_v2",
    "honest_gate_v1",
    "refresh_mcp_witness_integrity_ops_witness",
    "run_post_build_hook",
    "scan_queue_integrity",
    "scan_witness_honesty",
]
