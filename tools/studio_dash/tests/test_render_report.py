"""P2 — studio HTML renderer."""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
if str(ROOT.parent) not in sys.path:
    sys.path.insert(0, str(ROOT.parent))

from studio_dash.render_report import render_report, write_report
from studio_dash.frame_qc import frame_qc_view
from studio_dash.massing_qc import massing_qc_view
from studio_dash.mission_adapter import (
    emit_frame_qc_dashboard,
    emit_massing_qc_dashboard,
    emit_mission_dashboard,
    emit_worldgen_dashboard,
    cross_link_doc,
)


def test_render_contains_title_and_table():
    html = render_report({"title": "Kit coverage", "status": "ok", "cells": 12})
    assert "Kit coverage" in html
    assert "<table>" in html
    assert "ok" in html


def test_write_report(tmp_path):
    out = write_report({"title": "t", "status": "pass", "series": [1, 2, 3]}, tmp_path / "r.html")
    text = out.read_text(encoding="utf-8")
    assert "polyline" in text


def test_render_republic_massing_lineage():
    html = render_report(
        {
            "title": "massing QC",
            "status": "presentation_only",
            "seed": 1,
            "pilot_id": "shape_l_3x2",
            "material_strategy": "l_shape",
            "grammar_rule_chain": {"archetype": "IndustrialWarehouse", "massing": "l_shape"},
            "floors_histogram": {"1": 2, "2": 1, "3": 0, "4": 0},
            "massing_columns": [
                {"tile": [0, 0], "floors": 1, "color_kind": "Valid", "weight": 1.0}
            ],
        }
    )
    assert "Grammar rule chain" in html
    assert "IndustrialWarehouse" in html
    assert "shape_l_3x2" in html
    assert "Floors histogram" in html


def test_render_param_frame_qc():
    root = Path(__file__).resolve().parents[3]
    example = (
        root
        / "tools"
        / "mcp"
        / "schemas"
        / "examples"
        / "param_frame_rail_warehouse_v1.json"
    )
    import json

    raw = json.loads(example.read_text(encoding="utf-8"))
    html = render_report(frame_qc_view(raw), title="frame QC")
    assert "Grammar chain (param frame)" in html
    assert "Modifiers" in html
    assert "Massing intent" in html
    assert "rail_warehouse_param_frame_v1" in html
    assert "Ref-books dual cite" not in html  # book bookkeeping is not shown in asset QC (CQ-79)


def test_qc_smoke_writes_html(tmp_path, monkeypatch):
    import studio_dash.qc_smoke as smoke

    monkeypatch.setattr(smoke, "_out_dir", lambda: tmp_path)
    frame_p, massing_p = smoke.run_smoke(out_dir=tmp_path)
    assert frame_p.is_file() and massing_p.is_file()
    assert "Grammar chain" in frame_p.read_text(encoding="utf-8")
    assert "Floors histogram" in massing_p.read_text(encoding="utf-8")


def test_frame_and_massing_qc_adapters(tmp_path, monkeypatch):
    import studio_dash.mission_adapter as ma

    monkeypatch.setattr(ma, "_out_dir", lambda: tmp_path)
    root = Path(__file__).resolve().parents[3]
    import json

    frame_raw = json.loads(
        (
            root
            / "tools"
            / "mcp"
            / "schemas"
            / "examples"
            / "param_frame_rail_warehouse_v1.json"
        ).read_text(encoding="utf-8")
    )
    massing_raw = json.loads(
        (
            root
            / "assets/configs/buildings/examples/massing_preview_pilot_rail_warehouse_v1.json"
        ).read_text(encoding="utf-8")
    )
    f = emit_frame_qc_dashboard(frame_raw, name="f1")
    m = emit_massing_qc_dashboard(massing_qc_view(massing_raw), name="m1")
    assert f.is_file() and m.is_file()
    assert "connection_graph" in f.read_text(encoding="utf-8").lower() or "Connection" in f.read_text(
        encoding="utf-8"
    )


def test_mission_and_worldgen_adapters(tmp_path, monkeypatch):
    import studio_dash.mission_adapter as ma

    monkeypatch.setattr(ma, "_out_dir", lambda: tmp_path)
    m = emit_mission_dashboard({"status": "green", "objectives": 3}, name="m1")
    w = emit_worldgen_dashboard({"status": "ok", "towns": 2}, name="w1")
    assert m.is_file() and w.is_file()
    assert "Mission" in m.read_text(encoding="utf-8") or "mission" in m.read_text(encoding="utf-8").lower()
    assert "studio_dash" in cross_link_doc()
