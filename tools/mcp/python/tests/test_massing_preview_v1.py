"""IO-09 / Lane A — massing_preview_v1 schema + APS export validation (no Blender)."""

from __future__ import annotations

import json
import sys
from pathlib import Path

import jsonschema
import pytest

from rust_engine_mcp.paths import schemas_dir
from rust_engine_mcp.schemas import validate_massing_preview

APS_PARENT = Path(__file__).resolve().parents[2]
if str(APS_PARENT) not in sys.path:
    sys.path.insert(0, str(APS_PARENT))

from art_pipeline_suite.voxel_massing_panel import (  # noqa: E402
    build_massing_preview,
    export_massing_preview,
    floors_histogram,
    massing_qc_view,
    normalize_massing_columns,
)


EXAMPLE = schemas_dir() / "examples" / "massing_preview_v1.json"


def test_example_schema_validates() -> None:
    data = json.loads(EXAMPLE.read_text(encoding="utf-8"))
    validate_massing_preview(data)


def test_required_presentation_flags() -> None:
    data = json.loads(EXAMPLE.read_text(encoding="utf-8"))
    bad = dict(data)
    bad["presentation_only"] = False
    with pytest.raises(jsonschema.ValidationError):
        validate_massing_preview(bad)
    bad2 = dict(data)
    bad2["sim_occupation"] = True
    with pytest.raises(jsonschema.ValidationError):
        validate_massing_preview(bad2)


def test_seed_must_be_non_negative() -> None:
    data = json.loads(EXAMPLE.read_text(encoding="utf-8"))
    data["seed"] = -1
    with pytest.raises(jsonschema.ValidationError):
        validate_massing_preview(data)


def test_floors_zero_rejected_by_schema() -> None:
    data = json.loads(EXAMPLE.read_text(encoding="utf-8"))
    data["massing_columns"] = [
        {"tile": [0, 0], "floors": 0, "color_kind": "Valid", "weight": 1.0}
    ]
    with pytest.raises(jsonschema.ValidationError):
        validate_massing_preview(data)


def test_normalize_omits_floors_zero_and_clamps() -> None:
    cols = normalize_massing_columns(
        [
            {"tile": [0, 0], "floors": 0, "color_kind": "Valid", "weight": 1.0},
            {"tile": [1, 0], "floors": 9, "color_kind": "Valid", "weight": 1.0},
            {"tile": [2, 0], "floors": 3, "color_kind": "Risky", "weight": 0.5},
        ]
    )
    assert len(cols) == 2
    assert cols[0]["floors"] == 4
    assert cols[1]["floors"] == 3
    assert cols[1]["color_kind"] == "Risky"


def test_build_and_export_validates(tmp_path: Path) -> None:
    payload = build_massing_preview(
        seed=42,
        columns=[
            {"tile": [0, 0], "floors": 0, "color_kind": "Valid", "weight": 1.0},
            {"tile": [1, 0], "floors": 2, "color_kind": "Valid", "weight": 1.0},
            {"tile": [2, 0], "floors": 5, "color_kind": "Invalid", "weight": 0.25},
        ],
    )
    assert payload["presentation_only"] is True
    assert payload["sim_occupation"] is False
    assert payload["seed"] == 42
    assert all(c["floors"] >= 1 for c in payload["massing_columns"])
    out = tmp_path / "massing_preview.json"
    written = export_massing_preview(out, payload)
    validate_massing_preview(written)
    loaded = json.loads(out.read_text(encoding="utf-8"))
    assert loaded["seed"] == 42
    assert len(loaded["massing_columns"]) == 2


def test_massing_qc_view_histogram() -> None:
    data = build_massing_preview(
        seed=7,
        columns=[
            {"tile": [0, 0], "floors": 1, "color_kind": "Valid", "weight": 1.0},
            {"tile": [1, 0], "floors": 1, "color_kind": "Valid", "weight": 1.0},
            {"tile": [0, 1], "floors": 4, "color_kind": "Valid", "weight": 1.0},
        ],
    )
    qc = massing_qc_view(data)
    assert qc["column_count"] == 3
    assert qc["seed"] == 7
    assert floors_histogram(data["massing_columns"]) == {"1": 2, "2": 0, "3": 0, "4": 1}
    assert qc["floors_histogram"]["1"] == 2
    assert qc["floors_histogram"]["4"] == 1
    assert qc["metrics"]["column_count"] == 3


def test_studio_dash_renders_massing_qc() -> None:
    studio = Path(__file__).resolve().parents[3] / "studio_dash"
    parent = studio.parent
    if str(parent) not in sys.path:
        sys.path.insert(0, str(parent))
    from studio_dash.render_report import render_report

    data = massing_qc_view(json.loads(EXAMPLE.read_text(encoding="utf-8")))
    html = render_report(data, title="massing_preview QC")
    assert "column_count" in html
    assert "Floors histogram" in html
    assert "Massing columns" in html
    assert str(data["seed"]) in html


PILOT_EXAMPLE = schemas_dir() / "examples" / "massing_preview_pilot_rail_warehouse_v1.json"


def test_pilot_example_schema_and_lineage() -> None:
    from rust_engine_mcp.massing_from_pilot import build_massing_preview_from_pilot

    data = json.loads(PILOT_EXAMPLE.read_text(encoding="utf-8"))
    validate_massing_preview(data)
    assert data["pilot_id"] == "logistics_rail_warehouse_v0"
    assert data.get("grammar_rule_chain")
    assert data.get("arch_build_grammar_preset_id")
    rebuilt = build_massing_preview_from_pilot("logistics_rail_warehouse_v0", seed=1)
    assert len(rebuilt["massing_columns"]) == len(data["massing_columns"])
    assert rebuilt["grammar_rule_chain"]["archetype"] == "IndustrialWarehouse"


def test_shape_qa_pilot_export_no_chain_required() -> None:
    from rust_engine_mcp.massing_from_pilot import build_massing_preview_from_pilot

    data = build_massing_preview_from_pilot("shape_l_3x2", seed=3)
    validate_massing_preview(data)
    assert data["pilot_id"] == "shape_l_3x2"
    assert len(data["massing_columns"]) == 4
    assert "grammar_rule_chain" not in data
