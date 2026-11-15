"""RND-08 — atlas gutter bleed and texel-density QC (synthetic atlases)."""

from __future__ import annotations

from pathlib import Path

from PIL import Image

from rust_engine_mcp.aps_atlas_qc import (
    check_atlas_gutter_bleed,
    check_texel_density_band,
    min_gutter_for_mip_levels,
    validate_atlas_rnd08_qc,
)


def _fill_cell(
    image: Image.Image,
    *,
    col: int,
    row: int,
    cell: int,
    content: int,
    gutter: int,
    fill: tuple[int, int, int, int],
    extrude: bool,
) -> None:
    px = image.load()
    base_x = col * cell
    base_y = row * cell
    cx = base_x + gutter
    cy = base_y + gutter
    for y in range(content):
        for x in range(content):
            px[cx + x, cy + y] = fill
    if not extrude or gutter <= 0:
        return
    for y in range(content):
        edge = px[cx, cy + y]
        for dx in range(gutter):
            px[base_x + dx, cy + y] = edge
        edge_r = px[cx + content - 1, cy + y]
        for dx in range(gutter):
            px[cx + content + dx, cy + y] = edge_r
    for x in range(content):
        edge = px[cx + x, cy]
        for dy in range(gutter):
            px[cx + x, base_y + dy] = edge
        edge_b = px[cx + x, cy + content - 1]
        for dy in range(gutter):
            px[cx + x, cy + content + dy] = edge_b


def _synthetic_meta(
    *,
    content: int,
    gutter: int,
    mip_levels: int,
    zoom_band: str | None = None,
    lookups: list[dict] | None = None,
) -> dict:
    cell = content + 2 * gutter
    meta: dict = {
        "schema_version": 2,
        "columns": 1,
        "rows": 1,
        "tile_px": content,
        "atlas_qc": {"mip_levels": mip_levels, "check_bleed": True},
    }
    if zoom_band:
        meta["atlas_qc"]["zoom_band"] = zoom_band
        meta["atlas_qc"]["check_texel_density"] = True
    meta["lookups"] = lookups or [
        {"variant": "a", "facing": 0, "frame": 0, "grid": [0, 0], "uv": [0, 0, 1, 1]}
    ]
    return meta


def test_min_gutter_for_mip_levels() -> None:
    assert min_gutter_for_mip_levels(0) == 1
    assert min_gutter_for_mip_levels(2) == 4
    assert min_gutter_for_mip_levels(3) == 8


def test_gutter_bleed_passes_when_gutter_meets_mip_and_extruded() -> None:
    content, gutter, mip = 24, 4, 2
    cell = content + 2 * gutter
    image = Image.new("RGBA", (cell, cell), (0, 0, 0, 0))
    _fill_cell(
        image,
        col=0,
        row=0,
        cell=cell,
        content=content,
        gutter=gutter,
        fill=(200, 40, 40, 255),
        extrude=True,
    )
    meta = _synthetic_meta(content=content, gutter=gutter, mip_levels=mip)
    issues = check_atlas_gutter_bleed(image, meta)
    assert issues == []


def test_gutter_bleed_fails_when_gutter_too_small() -> None:
    content, gutter, mip = 28, 2, 2
    cell = content + 2 * gutter
    image = Image.new("RGBA", (cell, cell), (0, 0, 0, 0))
    _fill_cell(
        image,
        col=0,
        row=0,
        cell=cell,
        content=content,
        gutter=gutter,
        fill=(10, 120, 200, 255),
        extrude=True,
    )
    meta = _synthetic_meta(content=content, gutter=gutter, mip_levels=mip)
    issues = check_atlas_gutter_bleed(image, meta)
    assert any(i.signature == "atlas_qc_gutter_too_small" for i in issues)


def test_gutter_bleed_fails_when_gutter_not_extruded() -> None:
    content, gutter, mip = 24, 4, 2
    cell = content + 2 * gutter
    image = Image.new("RGBA", (cell, cell), (0, 0, 0, 0))
    _fill_cell(
        image,
        col=0,
        row=0,
        cell=cell,
        content=content,
        gutter=gutter,
        fill=(80, 200, 80, 255),
        extrude=False,
    )
    meta = _synthetic_meta(content=content, gutter=gutter, mip_levels=mip)
    issues = check_atlas_gutter_bleed(image, meta)
    assert any(i.signature == "atlas_qc_gutter_not_extruded" for i in issues)


def test_texel_density_passes_within_five_percent() -> None:
    meta = _synthetic_meta(
        content=128,
        gutter=0,
        mip_levels=0,
        zoom_band="building_iso",
        lookups=[
            {"variant": "a", "world_metres_per_tile": 4.0},
            {"variant": "b", "world_metres_per_tile": 4.1},
        ],
    )
    issues = check_texel_density_band(meta)
    assert issues == []


def test_texel_density_fails_when_drift_over_five_percent() -> None:
    meta = _synthetic_meta(
        content=128,
        gutter=0,
        mip_levels=0,
        zoom_band="building_iso",
        lookups=[
            {"variant": "a", "world_metres_per_tile": 4.0},
            {"variant": "b", "world_metres_per_tile": 4.5},
        ],
    )
    issues = check_texel_density_band(meta)
    assert any(i.signature == "atlas_qc_texel_density_drift" for i in issues)


def test_validate_atlas_rnd08_qc_combined_pass(tmp_path: Path) -> None:
    content, gutter, mip = 24, 4, 2
    cell = content + 2 * gutter
    image = Image.new("RGBA", (cell, cell), (0, 0, 0, 0))
    _fill_cell(
        image,
        col=0,
        row=0,
        cell=cell,
        content=content,
        gutter=gutter,
        fill=(220, 180, 60, 255),
        extrude=True,
    )
    meta = _synthetic_meta(
        content=content,
        gutter=gutter,
        mip_levels=mip,
        zoom_band="building_iso",
        lookups=[
            {"variant": "a", "world_metres_per_tile": 4.0},
            {"variant": "b", "world_metres_per_tile": 4.05},
        ],
    )
    issues = validate_atlas_rnd08_qc(meta, image, folder=tmp_path)
    assert issues == []


def test_plain_language_maps_bleed_signatures() -> None:
    from rust_engine_mcp.aps_atlas_qc import _PLAIN, plain_language_lines
    from rust_engine_mcp.validators.report import ValidationIssue, ValidationReport

    report = ValidationReport(
        validator="tile",
        status="failed",
        errors=[
            ValidationIssue(kind="AtlasBleed", signature="atlas_qc_gutter_too_small"),
        ],
    )
    lines = plain_language_lines(report)
    assert any("mip" in line.lower() for line in lines)
    assert "atlas_qc_gutter_too_small" in _PLAIN