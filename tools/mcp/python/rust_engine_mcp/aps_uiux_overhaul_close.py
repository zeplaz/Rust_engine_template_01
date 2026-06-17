"""OVR-P6-CLOSE-001 — program close witness rollup."""

from __future__ import annotations

import json
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.aps_uiux_g0_audit import run_ban_list_audit
from rust_engine_mcp.paths import repo_root

WITNESS_REL = "debug_runs/aps_uiux_overhaul_close_live.json"


def _pytest_aps_pass(*, repo: Path) -> bool:
    cmd = [
        sys.executable,
        "-m",
        "pytest",
        "tests",
        "-k",
        "aps and not e0_e2_relaunch",
        "-q",
    ]
    proc = subprocess.run(cmd, cwd=repo / "tools/mcp/python", capture_output=True, text=True)
    return proc.returncode == 0


def refresh_aps_uiux_overhaul_close_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    audit = run_ban_list_audit(repo=root)
    pytest_ok = _pytest_aps_pass(repo=root)
    green = pytest_ok and bool(audit.get("ui_clean"))
    body: dict[str, Any] = {
        "gate_id": "OVR-P6-CLOSE-001",
        "program_id": "PLAN-APS-UIUX-OVERHAUL-001",
        "green": green,
        "pytest_aps_ok": pytest_ok,
        "ban_list": audit,
        "phases_complete": [
            "OVR-P1-TOKENS-001",
            "OVR-P2-TEXT-001",
            "OVR-P3-LAYOUT-001",
            "OVR-P4-IA-001",
            "OVR-P45-SPINE-001",
            "OVR-P5-STYLE-001",
            "OVR-P55-PREVIEW-001",
            "OVR-P56-ONBOARD-001",
        ],
        "human_gates_pending": [
            "OVR-P6-OPERATOR-EYEBALL-001",
            "OVR-P6-DESIGN-SIGN-001",
            "DMCP-OVR-ARTIST-ACCEPT-001",
        ],
        "exit_predicate": {
            "pytest_aps_ok": {"eq": pytest_ok},
            "ban_list_clean": {"eq": bool(audit.get("ui_clean"))},
        },
        "_agent_meta": {
            "schema": "aps_uiux_overhaul_close_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "OVR_P6_CLOSE",
            "relative_path": WITNESS_REL,
        },
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body
