#!/usr/bin/env python3
"""Bake Phase 4 icon atlas silhouettes (UI-OH-P4-ART-001).

Spec: prompts/guides/ui/ui_phase4_icon_atlas_brief_v1.md §4–§7
Output: assets/textures/ui/icon_atlas_phase4_v1.png (256×128, 32×32 cells)
"""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageOps

REPO = Path(__file__).resolve().parents[3]
ASSETS = REPO / "assets"
OUT = ASSETS / "textures" / "ui" / "icon_atlas_phase4_v1.png"

CELL = 32
CONTENT = 24
MARGIN = (CELL - CONTENT) // 2
ATLAS_W, ATLAS_H = 256, 128
ALPHA_THRESH = 48


def load_rgba(path: Path) -> Image.Image:
    return Image.open(path).convert("RGBA")


def alpha_bbox(img: Image.Image, region: tuple[int, int, int, int] | None = None) -> tuple[int, int, int, int] | None:
    work = img.crop(region) if region else img
    alpha = work.split()[3]
    return alpha.point(lambda a: 255 if a > ALPHA_THRESH else 0).getbbox()


def trace_silhouette(
    img: Image.Image,
    *,
    region: tuple[int, int, int, int] | None = None,
    rotate_deg: float = 0,
) -> Image.Image:
    if region:
        img = img.crop(region)
    box = alpha_bbox(img)
    if not box:
        return Image.new("RGBA", (CONTENT, CONTENT), (0, 0, 0, 0))
    cropped = img.crop(box)
    alpha = cropped.split()[3].point(lambda a: 255 if a > ALPHA_THRESH else 0)
    silhouette = Image.new("RGBA", cropped.size, (255, 255, 255, 0))
    silhouette.putalpha(alpha)
    if rotate_deg:
        silhouette = silhouette.rotate(rotate_deg, expand=True, resample=Image.Resampling.BICUBIC)
        box2 = silhouette.split()[3].getbbox()
        if box2:
            silhouette = silhouette.crop(box2)
    silhouette.thumbnail((CONTENT, CONTENT), Image.Resampling.LANCZOS)
    cell = Image.new("RGBA", (CELL, CELL), (0, 0, 0, 0))
    ox = MARGIN + (CONTENT - silhouette.width) // 2
    oy = MARGIN + (CONTENT - silhouette.height) // 2
    cell.paste(silhouette, (ox, oy), silhouette)
    return cell


def center_crop_square(img: Image.Image, size: int) -> Image.Image:
    w, h = img.size
    half = size // 2
    cx, cy = w // 2, h // 2
    return img.crop((cx - half, cy - half, cx + half, cy + half))


def railroad_region(img: Image.Image, *, rail_focus: bool) -> tuple[int, int, int, int]:
    w, h = img.size
    band_h = max(48, h // 4)
    y0 = (h - band_h) // 2
    if rail_focus:
        # Stronger parallel-rail read: lower band + slight vertical emphasis
        y0 = min(h - band_h, y0 + band_h // 4)
    return (0, y0, w, y0 + band_h)


def sheet_icon_region(img: Image.Image) -> tuple[int, int, int, int]:
    """Pick largest alpha cluster in center crop of a tile sheet."""
    probe = center_crop_square(img, min(img.width, img.height, 768))
    alpha = probe.split()[3]
    mask = alpha.point(lambda a: 255 if a > ALPHA_THRESH else 0)
    bbox = mask.getbbox()
    if not bbox:
        w, h = probe.size
        s = min(w, h) // 3
        return (w // 2 - s // 2, h // 2 - s // 2, w // 2 + s // 2, h // 2 + s // 2)
    x0, y0, x1, y1 = bbox
    pad = max(8, (x1 - x0) // 8)
    return (
        max(0, x0 - pad),
        max(0, y0 - pad),
        min(probe.width, x1 + pad),
        min(probe.height, y1 + pad),
    )


def building_region(img: Image.Image, *, industrial: bool) -> tuple[int, int, int, int]:
    w, h = img.size
    if industrial:
        # Chimney / vertical mass — upper-center crop
        return (w // 4, h // 8, w * 3 // 4, h * 5 // 8)
    # Civic low-rise — mid band
    return (w // 6, h // 3, w * 5 // 6, h * 2 // 3)


def paste_cell(atlas: Image.Image, cell: Image.Image, col: int, row: int) -> None:
    atlas.paste(cell, (col * CELL, row * CELL), cell)


def main() -> None:
    cells: dict[str, Image.Image] = {}

    rd_src = load_rgba(ASSETS / "textures/misc/railroad_track.png")
    cells["RD"] = trace_silhouette(rd_src, region=railroad_region(rd_src, rail_focus=False))
    cells["RL"] = trace_silhouette(
        rd_src,
        region=railroad_region(rd_src, rail_focus=True),
        rotate_deg=15,
    )

    tx_src = load_rgba(
        ASSETS / "textures/power/tile_map_powerstuff_power_trasformer_oil_cooled_alpha.png"
    )
    tx_region = sheet_icon_region(tx_src)
    cells["UT"] = trace_silhouette(tx_src, region=tx_region)
    cells["UT_TX"] = cells["UT"].copy()

    mg_src = load_rgba(ASSETS / "textures/power/tile_map_rust_dev_utils_alpha.png")
    cells["UT_MG"] = trace_silhouette(mg_src, region=sheet_icon_region(mg_src))

    in_src = load_rgba(ASSETS / "textures/misc/wooden_buildings_01.png")
    cells["IN"] = trace_silhouette(in_src, region=building_region(in_src, industrial=True))

    cv_src = load_rgba(ASSETS / "textures/misc/cities.png")
    cells["CV"] = trace_silhouette(cv_src, region=building_region(cv_src, industrial=False))

    truck_src = load_rgba(ASSETS / "textures/vehicles/civ_truck_01/tile_map_8_empty_miday.png")
    cells["TRUCK"] = trace_silhouette(truck_src, region=sheet_icon_region(truck_src))

    ural_src = load_rgba(ASSETS / "textures/vehicles/ural_01/tile_map_ural_01_empty_midday.png")
    cells["URAL"] = trace_silhouette(ural_src, region=sheet_icon_region(ural_src))

    bus_src = load_rgba(ASSETS / "textures/vehicles/bus_01/tilemap_bus_01_alpha.png")
    cells["BUS"] = trace_silhouette(bus_src, region=sheet_icon_region(bus_src))

    barrel_src = load_rgba(ASSETS / "textures/misc/hjm-barrel_alpha.png")
    cells["P5_BR"] = trace_silhouette(barrel_src, region=sheet_icon_region(barrel_src))

    atlas = Image.new("RGBA", (ATLAS_W, ATLAS_H), (0, 0, 0, 0))
    layout = {
        "RD": (0, 0),
        "RL": (1, 0),
        "UT": (2, 0),
        "IN": (3, 0),
        "CV": (4, 0),
        "UT_TX": (0, 1),
        "UT_MG": (1, 1),
        "TRUCK": (0, 2),
        "URAL": (1, 2),
        "BUS": (2, 2),
        "P5_BR": (0, 3),
    }
    for name, (col, row) in layout.items():
        paste_cell(atlas, cells[name], col, row)

    OUT.parent.mkdir(parents=True, exist_ok=True)
    atlas.save(OUT, optimize=True)
    print(f"wrote {OUT.relative_to(REPO)} ({ATLAS_W}x{ATLAS_H})")


if __name__ == "__main__":
    main()
