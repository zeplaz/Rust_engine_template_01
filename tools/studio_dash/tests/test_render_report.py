"""P2 — studio HTML renderer."""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
if str(ROOT.parent) not in sys.path:
    sys.path.insert(0, str(ROOT.parent))

from studio_dash.render_report import render_report, write_report
from studio_dash.mission_adapter import emit_mission_dashboard, emit_worldgen_dashboard, cross_link_doc


def test_render_contains_title_and_table():
    html = render_report({"title": "Kit coverage", "status": "ok", "cells": 12})
    assert "Kit coverage" in html
    assert "<table>" in html
    assert "ok" in html


def test_write_report(tmp_path):
    out = write_report({"title": "t", "status": "pass", "series": [1, 2, 3]}, tmp_path / "r.html")
    text = out.read_text(encoding="utf-8")
    assert "polyline" in text


def test_mission_and_worldgen_adapters(tmp_path, monkeypatch):
    import studio_dash.mission_adapter as ma

    monkeypatch.setattr(ma, "_out_dir", lambda: tmp_path)
    m = emit_mission_dashboard({"status": "green", "objectives": 3}, name="m1")
    w = emit_worldgen_dashboard({"status": "ok", "towns": 2}, name="w1")
    assert m.is_file() and w.is_file()
    assert "Mission" in m.read_text(encoding="utf-8") or "mission" in m.read_text(encoding="utf-8").lower()
    assert "studio_dash" in cross_link_doc()
