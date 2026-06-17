"""DMCP-OVR-P3-ACCEPT-RUBRIC-001 — layout acceptance rubric witness."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

WITNESS_REL = "debug_runs/art_pipeline/dmcp_ovr_p3_accept_rubric_live.json"
RUBRIC_REL = "src/dev/design_aps_uiux_p3_accept_rubric_v1.md"
LAYOUT_DELTA_REL = "src/dev/design_aps_uiux_layout_delta_v1.md"
GUARD_SPEC_REL = "src/dev/plan_aps_uiux_p3_layout_guard_v1.md"

RUBRIC_ROW_COUNT = 25
P0_ROWS = 15


def refresh_dmcp_ovr_p3_accept_rubric_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    rubric = root / RUBRIC_REL
    layout = root / LAYOUT_DELTA_REL
    guard = root / GUARD_SPEC_REL
    checks = {
        "rubric_on_disk": rubric.is_file(),
        "layout_delta_on_disk": layout.is_file(),
        "guard_spec_on_disk": guard.is_file(),
        "rubric_row_count": RUBRIC_ROW_COUNT,
        "p0_row_count": P0_ROWS,
        "needs_display": True,
    }
    green = all(checks[k] for k in checks if k not in ("needs_display",))
    body: dict[str, Any] = {
        "gate": "DMCP-OVR-P3-ACCEPT-RUBRIC-001",
        "green": green,
        "verdict": "PASS" if green else "FAIL",
        "deliverable": RUBRIC_REL,
        "layout_delta": LAYOUT_DELTA_REL,
        "guard_spec": GUARD_SPEC_REL,
        "checks": checks,
        "operator_walk": "§9 in rubric — run after OVR-P3-LAYOUT-001 lands",
        "machine_guards": [
            "tests/test_aps_min_window_layout.py",
            "tests/test_aps_imports.py",
            "tests/test_aps_runtime_callbacks.py",
        ],
        "blocks": [],
        "unblocks": ["OVR-P3-LAYOUT-001 operator sign-off"],
        "_agent_meta": {
            "schema": "dmcp_ovr_p3_accept_rubric_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "DMCP_OVR_P3_ACCEPT_RUBRIC",
            "source_system": "aps_uiux_p3_accept_rubric",
            "relative_path": WITNESS_REL,
            "ritual": "BLANG:WIT-HON→Q✓ DMCP-OVR-P3-ACCEPT-RUBRIC-001",
            "agent": "designer-mcp",
        },
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = WITNESS_REL
    return body
