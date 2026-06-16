"""APS-UX-NONBLOCK-001 + SCROLL-001 tests."""

from __future__ import annotations

import json
import re
from pathlib import Path

import pytest

from rust_engine_mcp.aps_ux_nonblock_witness import (
    APS_UX_NONBLOCK_WITNESS,
    PANELS_MIGRATED,
    refresh_aps_ux_nonblock_witness,
)
from rust_engine_mcp.paths import repo_root


def test_routine_modals_migrated():
    suite = repo_root() / "tools/mcp/art_pipeline_suite"
    pat = re.compile(r"messagebox\.(showinfo|showwarning|showerror|askyesno)")
    calls: list[str] = []
    for name in PANELS_MIGRATED:
        path = suite / name
        if not path.is_file():
            continue
        for m in pat.finditer(path.read_text(encoding="utf-8")):
            line = path.read_text(encoding="utf-8").count("\n", 0, m.start()) + 1
            calls.append(f"{name}:{line}:{m.group(0)}")
    allowlisted = [
        c
        for c in calls
        if "askyesno" in c
        or ("material_library_widget" in c and "showerror" in c)
    ]
    routine = [c for c in calls if c not in allowlisted]
    assert not routine, f"routine modals remain: {routine}"


def test_catalog_list_wheel_binding():
    text = (repo_root() / "tools/mcp/art_pipeline_suite/catalog.py").read_text(encoding="utf-8")
    assert "attach_wheel_area" in text
    assert "canvas_yscroll" in text


def test_refresh_nonblock_witness():
    assert refresh_aps_ux_nonblock_witness()
    data = json.loads((repo_root() / APS_UX_NONBLOCK_WITNESS).read_text(encoding="utf-8"))
    assert data["green"] is True
    assert data["messagebox_routine_remaining"] == 0
    assert data["scroll_catalog_wheel"] is True
