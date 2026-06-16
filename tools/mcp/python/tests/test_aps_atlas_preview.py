"""APS-ATLAS-PREVIEW-001 smoke."""

from __future__ import annotations

from pathlib import Path

from rust_engine_mcp.paths import repo_root


def test_atlas_preview_modules_exist() -> None:
    root = repo_root()
    assert (root / "tools/mcp/art_pipeline_suite/atlas_preview_panel.py").is_file()
    assert (root / "tools/mcp/art_pipeline_suite/aps_tooltips.py").is_file()
    assert (root / "tools/mcp/art_pipeline_suite/pipeline_status_bar.py").is_file()
    assert (root / "tools/mcp/python/rust_engine_mcp/aps_grammar_labels.py").is_file()


def test_atlas_qc_witness_production_green() -> None:
    import json

    from rust_engine_mcp.aps_atlas_qc import (
        APS_ATLAS_PREVIEW_WITNESS,
        PRODUCTION_V2_ATLAS_FOLDER,
        write_aps_atlas_preview_witness,
    )

    path = write_aps_atlas_preview_witness()
    assert path.is_file()
    body = json.loads(path.read_text(encoding="utf-8"))
    assert body["folder"] == PRODUCTION_V2_ATLAS_FOLDER
    assert body["green"] is True
    assert body["atlas_meta_schema"] == "v2"
    assert body["validation_status"] == "passed"
    assert body["uv_grid_overlay"] is True
    assert (repo_root() / APS_ATLAS_PREVIEW_WITNESS).is_file()


def test_atlas_qc_pilot_v1_still_fails() -> None:
    import json

    from rust_engine_mcp.aps_atlas_qc import PILOT_V1_ATLAS_FOLDER, write_aps_atlas_preview_witness

    path = write_aps_atlas_preview_witness(repo_root() / PILOT_V1_ATLAS_FOLDER)
    body = json.loads(path.read_text(encoding="utf-8"))
    assert body["green"] is False
    assert body["atlas_meta_schema"] == "v1"


def test_atl_sign_001_witness_green() -> None:
    from rust_engine_mcp.aps_atlas_qc import ATL_SIGN_001_WITNESS, refresh_atl_sign_001_witness

    assert refresh_atl_sign_001_witness()
    import json

    body = json.loads((repo_root() / ATL_SIGN_001_WITNESS).read_text(encoding="utf-8"))
    assert body["gate_id"] == "ATL-SIGN-001"
    assert body["green"] is True
    assert body["aps_atlas_preview_002"]["green"] is True


def test_pilot_tile_folder_has_cells_and_meta() -> None:
    import json

    folder = repo_root() / "assets/staging/tiles/tile_warehouse_industrial_west_pilot_v1"
    if not folder.is_dir():
        return
    pngs = [
        p.name
        for p in folder.glob("*.png")
        if not p.name.lower().startswith("tile_map")
    ]
    assert len(pngs) >= 1
    meta = json.loads((folder / "atlas_meta.json").read_text(encoding="utf-8"))
    assert meta.get("tile_id") == "warehouse_industrial"
    assert len(meta.get("tiles") or []) >= 1


def test_tooltips_registry_nonempty() -> None:
    import sys

    aps = repo_root() / "tools/mcp/art_pipeline_suite"
    sys.path.insert(0, str(aps.parent.parent.parent))
    sys.path.insert(0, str(repo_root() / "tools/mcp"))
    from art_pipeline_suite.aps_tooltips import TOOLTIPS

    assert "asm_save" in TOOLTIPS
    assert "atl_preview" in TOOLTIPS
    assert "mat_status" in TOOLTIPS
    assert "pipeline_step" in TOOLTIPS
    assert len(TOOLTIPS) >= 15
