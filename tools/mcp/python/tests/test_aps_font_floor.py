"""P1-3 regression — no sub-9px fonts on primary content labels in the APS suite.

Greps the Tk source for hardcoded 8px font literals. Primary content must use the
shared FONT_SMALL / FONT_MONO_SMALL tokens (>= 9px). The footprint cell glyph uses
a deliberately dynamic Consolas size for tiny cells and is allowlisted by file+marker.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

import pytest

APS_ROOT = Path(__file__).resolve().parents[2]  # tools/mcp
if str(APS_ROOT) not in sys.path:
    sys.path.insert(0, str(APS_ROOT))

from rust_engine_mcp.paths import repo_root

SUITE = repo_root() / "tools/mcp/art_pipeline_suite"

# (file, substring) pairs allowed to use a sub-9 font literal, with justification.
ALLOWLIST = {
    # Dynamic per-cell glyph size for tiny grids; never below 7, default 9 — see APS-UX-NONCOLOR.
    ("footprint_canvas.py", "glyph_size"),
    # Token definitions — FONT_CAPTION 8 is decorative-only per aps_design_system_v1.md §3.1.
    ("aps_theme.py", "FONT_CAPTION"),
}

EIGHT_PX = re.compile(r'\(\s*"(?:Segoe UI|Consolas)"\s*,\s*8\b')


def test_no_8px_font_on_primary_labels():
    offenders: list[str] = []
    for path in sorted(SUITE.glob("*.py")):
        for i, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            if path.name == "aps_theme.py" and "Never use a literal" in line:
                continue  # legacy documentation comment
            if EIGHT_PX.search(line):
                if any(path.name == fname and token in line for fname, token in ALLOWLIST):
                    continue
                offenders.append(f"{path.name}:{i}: {line.strip()}")
    assert not offenders, "sub-9px font literals on content labels:\n" + "\n".join(offenders)


def test_font_small_token_exists():
    from art_pipeline_suite.aps_theme import FONT_MONO_SMALL, FONT_SMALL

    assert FONT_SMALL[1] >= 9
    assert FONT_MONO_SMALL[1] >= 9
