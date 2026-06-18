"""APS pytest tiers — fast (no Tk) vs full (headless GUI smoke)."""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path
from typing import Any

from .paths import repo_root

_PY_ROOT = repo_root() / "tools/mcp/python"

# Fast: static guards, rust_engine_mcp validators, import smoke — no Tk windows.
_APS_FAST_EXPR = "aps and not aps_gui and not e0_e2_relaunch"

# Full: includes headless Tk smoke (single root per module via tests/conftest.py).
_APS_FULL_EXPR = "aps and not e0_e2_relaunch"


def _run_pytest(k_expr: str, *, extra_args: list[str] | None = None) -> dict[str, Any]:
    cmd = [
        sys.executable,
        "-m",
        "pytest",
        "tests/",
        "-k",
        k_expr,
        "-q",
        "--tb=no",
    ]
    if extra_args:
        cmd.extend(extra_args)
    env = os.environ.copy()
    env["APS_TEST_HEADLESS"] = "1"
    env.setdefault("RUST_ENGINE_BEVY_PREVIEW", "0")
    proc = subprocess.run(cmd, cwd=str(_PY_ROOT), capture_output=True, text=True, env=env)
    out = (proc.stdout or "") + (proc.stderr or "")
    tail = out.strip().splitlines()
    summary = tail[-1] if tail else ""
    return {
        "ok": proc.returncode == 0,
        "summary": summary,
        "returncode": proc.returncode,
        "expr": k_expr,
    }


def run_pytest_aps_fast_gate() -> dict[str, Any]:
    """No Tk — guards, validators, domain_router, witness hooks (~seconds)."""
    return _run_pytest(_APS_FAST_EXPR)


def run_pytest_aps_full_gate() -> dict[str, Any]:
    """Headless Tk smoke + fast tests (~1–2 min)."""
    return _run_pytest(_APS_FULL_EXPR)


def run_pytest_aps_gui_gate() -> dict[str, Any]:
    """Tk smoke only."""
    return _run_pytest("aps_gui")
