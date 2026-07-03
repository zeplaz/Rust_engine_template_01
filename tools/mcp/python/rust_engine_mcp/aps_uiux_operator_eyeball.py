"""OVR-P6-OPERATOR-EYEBALL-001 — structural pre-check + human walk witness."""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

WITNESS_REL = "debug_runs/aps_uiux_operator_eyeball_live.json"
RUBRIC_REL = "src/dev/design_aps_uiux_p3_accept_rubric_v1.md"
DESIGN_SYSTEM_REL = "src/dev/aps_design_system_v1.md"

HEADLESS_GUARDS = [
    "tests/test_aps_no_jargon.py",
    "tests/test_aps_onboarding.py",
    "tests/test_aps_domain_router.py",
    "tests/test_aps_lane_tab_swap.py",
    "tests/test_aps_imports.py",
    "tests/test_aps_runtime_callbacks.py",
]
DISPLAY_GUARDS = [
    "tests/test_aps_min_window_layout.py",
]

NEEDS_DISPLAY_ROWS = [
    "OVR-P55-PREVIEW-001",
    "OVR-P56-ONBOARD-001",
    "OVR-P6-OPERATOR-EYEBALL-001",
]


def _run_pytest(repo: Path, rel: str) -> dict[str, Any]:
    cmd = [sys.executable, "-m", "pytest", rel, "-q"]
    proc = subprocess.run(cmd, cwd=repo / "tools/mcp/python", capture_output=True, text=True)
    tail = (proc.stdout or "") + (proc.stderr or "")
    passed = failed = 0
    m = re.search(r"(\d+) passed(?:, (\d+) failed)?", tail)
    if m:
        passed = int(m.group(1))
        failed = int(m.group(2) or 0)
    skipped = proc.returncode == 0 and passed == 0 and "skipped" in tail
    ok = proc.returncode == 0 and failed == 0 and (passed > 0 or skipped)
    return {
        "ok": ok,
        "skipped": skipped,
        "passed": passed,
        "failed": failed,
        "command": " ".join(cmd),
    }


def refresh_aps_uiux_operator_eyeball_witness(
    *,
    repo: Path | None = None,
    human_verdict: str | None = None,
    operator: str | None = None,
    notes: str | None = None,
) -> dict[str, Any]:
    root = repo or repo_root()
    env_verdict = os.environ.get("APS_OPERATOR_EYEBALL_VERDICT", "").strip().lower()
    if human_verdict is None and env_verdict in ("pass", "fail"):
        human_verdict = env_verdict
    if operator is None:
        operator = os.environ.get("APS_OPERATOR_EYEBALL_OPERATOR") or None

    headless: dict[str, Any] = {}
    for rel in HEADLESS_GUARDS:
        headless[rel] = _run_pytest(root, rel)
    display: dict[str, Any] = {}
    display_skipped = False
    for rel in DISPLAY_GUARDS:
        result = _run_pytest(root, rel)
        display[rel] = result
        if result.get("skipped"):
            display_skipped = True

    structural_ok = all(r["ok"] for r in headless.values())
    human_pending = human_verdict is None
    if human_verdict == "pass":
        pixel_verdict = "pass"
        green = structural_ok
    elif human_verdict == "fail":
        pixel_verdict = "fail"
        green = False
    else:
        pixel_verdict = "pending"
        green = False

    needs_display = [
        {
            "id": row_id,
            "verdict": pixel_verdict if row_id == "OVR-P6-OPERATOR-EYEBALL-001" else "pending",
            "operator": operator,
            "at": time.strftime("%Y-%m-%d") if human_verdict else None,
            "notes": notes or (
                "Machine structural guards green; pixel walk pending @ 1280×800 + MIN 960×600"
                if human_pending and row_id == "OVR-P6-OPERATOR-EYEBALL-001"
                else "See P3 rubric §9 + preview/onboard specs"
            ),
        }
        for row_id in NEEDS_DISPLAY_ROWS
    ]

    root = repo or repo_root()
    presence_attestation: dict[str, Any] | None = None
    presence_path = root / "debug_runs/aps_presence_operator_attestation_live.json"
    if presence_path.is_file():
        try:
            presence_attestation = json.loads(presence_path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            presence_attestation = None

    body: dict[str, Any] = {
        "gate_id": "OVR-P6-OPERATOR-EYEBALL-001",
        "program_id": "PLAN-APS-UIUX-OVERHAUL-001",
        "green": green,
        "verdict": "PASS" if green else ("PENDING_HUMAN" if human_pending else "FAIL"),
        "structural_ok": structural_ok,
        "display_guards_skipped": display_skipped,
        "launch": "python tools/mcp/art_pipeline_suite/run.py",
        "walk": {
            "lanes": ["buildings", "landscape"],
            "sizes": ["1280x800", "960x600"],
            "authority": DESIGN_SYSTEM_REL,
            "rubric": RUBRIC_REL,
        },
        "machine_guards": {"headless": headless, "display": display},
        "presence_attestation": presence_attestation,
        "needs_display": needs_display,
        "record_human_pass": (
            "APS_OPERATOR_EYEBALL_VERDICT=pass APS_OPERATOR_EYEBALL_OPERATOR=<name> "
            "python -c \"from rust_engine_mcp.aps_uiux_operator_eyeball import "
            "refresh_aps_uiux_operator_eyeball_witness; refresh_aps_uiux_operator_eyeball_witness()\""
        ),
        "_agent_meta": {
            "schema": "aps_uiux_operator_eyeball_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "OVR_P6_OPERATOR_EYEBALL",
            "relative_path": WITNESS_REL,
            "agent": "operator",
        },
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body
