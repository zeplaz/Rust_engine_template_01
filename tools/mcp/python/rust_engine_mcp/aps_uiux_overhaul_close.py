"""OVR-P6-CLOSE-001 — program close witness rollup."""

from __future__ import annotations

import json
import re
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.aps_uiux_g0_audit import run_ban_list_audit
from rust_engine_mcp.paths import repo_root

WITNESS_REL = "debug_runs/aps_uiux_overhaul_close_live.json"
OPERATOR_WITNESS_REL = "debug_runs/aps_uiux_operator_eyeball_live.json"


def _pytest_aps_pass(*, repo: Path) -> tuple[bool, dict[str, Any]]:
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
    tail = (proc.stdout or "") + (proc.stderr or "")
    passed = failed = skipped = 0
    m = re.search(r"(\d+) passed(?:, (\d+) failed)?(?:, (\d+) skipped)?", tail)
    if m:
        passed = int(m.group(1))
        failed = int(m.group(2) or 0)
        skipped = int(m.group(3) or 0)
    return proc.returncode == 0, {
        "passed": passed,
        "failed": failed,
        "skipped": skipped,
        "command": " ".join(cmd),
    }


def _human_gates_pending(root: Path) -> list[str]:
    pending = [
        "OVR-P6-OPERATOR-EYEBALL-001",
        "OVR-P6-DESIGN-SIGN-001",
        "DMCP-OVR-ARTIST-ACCEPT-001",
    ]
    op_path = root / OPERATOR_WITNESS_REL
    if op_path.is_file():
        op = json.loads(op_path.read_text(encoding="utf-8"))
        if op.get("verdict") == "PASS":
            pending = [x for x in pending if x != "OVR-P6-OPERATOR-EYEBALL-001"]
    signoff = root / "src/dev/design_aps_uiux_overhaul_signoff_v1.md"
    if signoff.is_file():
        pending = [x for x in pending if x != "OVR-P6-DESIGN-SIGN-001"]
    artist = root / "src/dev/design_aps_artist_ship_review_uiux_v1.md"
    if artist.is_file():
        pending = [x for x in pending if x != "DMCP-OVR-ARTIST-ACCEPT-001"]
    return pending


def refresh_aps_uiux_overhaul_close_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    audit = run_ban_list_audit(repo=root)
    pytest_ok, pytest_aps = _pytest_aps_pass(repo=root)
    human_pending = _human_gates_pending(root)
    machine_green = pytest_ok and bool(audit.get("ui_clean"))
    green = machine_green and not human_pending
    needs_display: list[dict[str, Any]] = []
    op_path = root / OPERATOR_WITNESS_REL
    if op_path.is_file():
        op = json.loads(op_path.read_text(encoding="utf-8"))
        needs_display = list(op.get("needs_display") or [])
    body: dict[str, Any] = {
        "gate_id": "OVR-P6-CLOSE-001",
        "program_id": "PLAN-APS-UIUX-OVERHAUL-001",
        "status": "pass" if green else "pending_human",
        "green": green,
        "pytest_aps_ok": pytest_ok,
        "pytest_aps": pytest_aps,
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
        "human_gates_pending": human_pending,
        "needs_display": needs_display,
        "guards": {
            "P1": ["test_aps_font_floor.py", "test_aps_style_tokens.py", "test_aps_ux_polish_density_tokens.py"],
            "P2": ["test_aps_no_jargon.py"],
            "P3": ["test_aps_min_window_layout.py"],
            "P4": ["test_aps_lane_tab_swap.py", "test_aps_runtime_callbacks.py"],
            "P4.5": ["test_aps_runtime_callbacks.py"],
            "P5": ["test_aps_style_tokens.py"],
            "P5.5": ["test_aps_runtime_callbacks.py"],
            "P5.6": ["test_aps_onboarding.py"],
            "P6": ["pytest -k aps"],
        },
        "exit_predicate": {
            "pytest_aps_ok": {"eq": pytest_ok},
            "ban_list_clean": {"eq": bool(audit.get("ui_clean"))},
            "human_gates_clear": {"eq": not human_pending},
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
