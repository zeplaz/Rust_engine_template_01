"""TILE-FIX-010 promotion gate tests."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from rust_engine_mcp.atlas_meta_v2_pack import pack_cells_to_atlas, write_atlas_meta_v2
from rust_engine_mcp.building_definition import expand_bake_matrix_minimum, load_building_definition
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators.atlas_meta import validate_atlas_meta_v2
from rust_engine_mcp.validators.tile_promotion import validate_tile_promotion

ROOT = repo_root()
WAREHOUSE_BDEF = ROOT / "tools/mcp/schemas/examples/building_definition_warehouse_industrial_west_production_v1.json"


def test_minimum_meta_v2_passes_lookup_validator(tmp_path) -> None:
    if not WAREHOUSE_BDEF.is_file():
        pytest.skip("warehouse bdef missing")
    try:
        from PIL import Image
    except ImportError:
        pytest.skip("Pillow not installed")

    defn = load_building_definition(WAREHOUSE_BDEF)
    cells = expand_bake_matrix_minimum(defn)
    from rust_engine_mcp.atlas_meta_v2_pack import cell_png_basename

    for cell in cells:
        Image.new("RGBA", (128, 128), (200, 100, 50, 255)).save(tmp_path / cell_png_basename(cell))

    pack_info = pack_cells_to_atlas(cells, tmp_path, atlas_png=tmp_path / "atlas.png", columns=8)
    batch = {
        "batch_id": "tile_test_minimum",
        "tile_id": defn.building_id,
        "atlas": {"atlas_id": "warehouse_industrial_west_v2", "meta_json": str(tmp_path / "meta.json")},
        "render_contract": defn.render_contract,
    }
    meta_path = write_atlas_meta_v2(
        batch=batch,
        pack_info=pack_info,
        atlas_png_rel="atlas.png",
        visual_config_rel="assets/configs/buildings/visual_config_warehouse_industrial_west_v2.json",
        minimum_g4_ship=True,
    )
    vc = ROOT / "assets/configs/buildings/visual_config_warehouse_industrial_west_v2.json"
    rep = validate_atlas_meta_v2(meta_path, visual_config_path=vc)
    assert rep.status == "passed"
    assert len(json.loads(meta_path.read_text(encoding="utf-8"))["lookups"]) == 24


def test_detect_art_quality_rejects_headless_marker(tmp_path) -> None:
    from rust_engine_mcp.tile_compile_loop import _detect_art_quality

    staging = tmp_path / "tile_warehouse_industrial_v2_minimum_g4"
    staging.mkdir()
    (staging / "clean_day_f0.png").write_bytes(b"fake")
    (staging / "keyframe_manual.export").write_text(
        json.dumps({"method": "blender_keyframe_light_rig", "exported_at": "2026-01-01T00:00:00Z"}),
        encoding="utf-8",
    )
    assert _detect_art_quality(staging) == "rejected_headless_procedural"


def test_detect_art_quality_accepts_real_manual_marker(tmp_path) -> None:
    from rust_engine_mcp.tile_compile_loop import _detect_art_quality

    staging = tmp_path / "tile_test_minimum_g4"
    staging.mkdir()
    (staging / "keyframe_manual.export").write_text(
        json.dumps(
            {
                "method": "keyframe_render.py",
                "export_mode": "keyframe_manual",
                "exported_at": "2026-01-01T00:00:00Z",
            }
        ),
        encoding="utf-8",
    )
    assert _detect_art_quality(staging) == "keyframe_manual"


def test_tile_promotion_rejects_identical_facings(tmp_path) -> None:
    try:
        from PIL import Image
    except ImportError:
        pytest.skip("Pillow not installed")

    staging = tmp_path / "tile_warehouse_industrial_v2_minimum_g4"
    staging.mkdir()
    img = Image.new("RGBA", (128, 128), (100, 100, 100, 255))
    for i in range(8):
        img.save(staging / f"clean_day_f{i}.png")
    img.save(staging / "clean_night_on_f0.png")
    for i in range(1, 8):
        img.save(staging / f"clean_night_on_f{i}.png")

    if not WAREHOUSE_BDEF.is_file():
        pytest.skip("warehouse bdef missing")

    rep = validate_tile_promotion(
        building_definition_path=WAREHOUSE_BDEF,
        staging_dir=staging,
        ship=True,
        batch={"dry_run": False, "render": {"method": "keyframe_render.py"}},
    )
    kinds = {e.kind for e in rep.errors}
    assert "FacingRotationMissing" in kinds
