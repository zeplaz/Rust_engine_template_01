"""STEWARD-OVR-APS-REGRESS-001 — between-phase APS regression watch."""

from __future__ import annotations

import json
import re
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

WITNESS_REL = "debug_runs/aps_uiux_overhaul_regress_watch_live.json"
GUARD_TESTS = [
    "tests/test_aps_imports.py",
    "tests/test_aps_runtime_callbacks.py",
]
FULL_FILTER = "aps and not e0_e2_relaunch"


def _run_pytest(
    *,
    repo: Path,
    extra_args: list[str],
) -> dict[str, Any]:
    cmd = [sys.executable, "-m", "pytest", *extra_args, "-q"]
    proc = subprocess.run(cmd, cwd=repo / "tools/mcp/python", capture_output=True, text=True)
    tail = (proc.stdout or "") + (proc.stderr or "")
    passed = failed = skipped = 0
    m = re.search(r"(\d+) passed(?:, (\d+) failed)?(?:, (\d+) skipped)?", tail)
    if m:
        passed = int(m.group(1))
        failed = int(m.group(2) or 0)
        skipped = int(m.group(3) or 0)
    return {
        "ok": proc.returncode == 0,
        "returncode": proc.returncode,
        "passed": passed,
        "failed": failed,
        "skipped": skipped,
        "command": " ".join(cmd),
        "tail": tail.strip()[-400:],
    }


def refresh_aps_uiux_regress_watch_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    guards: dict[str, Any] = {}
    for rel in GUARD_TESTS:
        guards[rel] = _run_pytest(repo=root, extra_args=[rel])
    full = _run_pytest(repo=root, extra_args=["tests", "-k", FULL_FILTER])
    green = full["ok"] and all(g["ok"] for g in guards.values())
    body: dict[str, Any] = {
        "gate_id": "STEWARD-OVR-APS-REGRESS-001",
        "program_id": "PLAN-APS-UIUX-OVERHAUL-001",
        "green": green,
        "verdict": "PASS" if green else "FAIL",
        "guards": guards,
        "pytest_aps": {
            "passed": full["passed"],
            "failed": full["failed"],
            "skipped": full["skipped"],
            "command": full["command"],
        },
        "regression_baseline": 149,
        "delta_from_baseline": full["passed"] - 149,
        "exit_predicate": {
            "imports_ok": {"eq": guards[GUARD_TESTS[0]]["ok"]},
            "runtime_callbacks_ok": {"eq": guards[GUARD_TESTS[1]]["ok"]},
            "full_aps_ok": {"eq": full["ok"]},
        },
        "_agent_meta": {
            "schema": "aps_uiux_overhaul_regress_watch_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "STEWARD_OVR_APS_REGRESS",
            "relative_path": WITNESS_REL,
            "agent": "sim-steward",
        },
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body
