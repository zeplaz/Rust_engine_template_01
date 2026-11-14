"""T5 — ref-books index --check stays green (no PDFs required)."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

_REPO = Path(__file__).resolve().parents[4]
_LOOKUP = _REPO / "src" / "dev" / "ref_books_index" / "lookup.py"


def test_lookup_check_exit_zero():
    r = subprocess.run(
        [sys.executable, "-I", str(_LOOKUP), "--check"],
        cwd=_REPO,
        capture_output=True,
        text=True,
    )
    assert r.returncode == 0, r.stdout + r.stderr
    assert "0 problems" in r.stdout
