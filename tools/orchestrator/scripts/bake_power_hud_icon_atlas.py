#!/usr/bin/env python3
"""Bake power HUD icon atlas wireframes (DES-ART-HUD-POWER-ICONS-001).

Output: assets/textures/ui/power_hud_atlas.png (80×80, 20×20 cells)
"""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw

REPO = Path(__file__).resolve().parents[3]
OUT = REPO / "assets" / "textures" / "ui" / "power_hud_atlas.png"
CELL = 20
COLS = 4
ROWS = 4
WHITE = (220, 220, 220, 255)


def cell_origin(col: int, row: int) -> tuple[int, int]:
    return col * CELL, row * CELL


def draw_line_tool(draw: ImageDraw.ImageDraw, ox: int, oy: int) -> None:
    draw.line([(ox + 3, oy + 15), (ox + 8, oy + 8), (ox + 14, oy + 12), (ox + 17, oy + 5)], fill=WHITE, width=2)
    draw.ellipse([(ox + 2, oy + 14), (ox + 6, oy + 18)], outline=WHITE)
    draw.ellipse([(ox + 15, oy + 3), (ox + 19, oy + 7)], outline=WHITE)


def draw_volt(draw: ImageDraw.ImageDraw, ox: int, oy: int, bars: int) -> None:
    for i in range(bars):
        h = 6 + i * 3
        x = ox + 4 + i * 4
        draw.line([(x, oy + 17), (x, oy + 17 - h)], fill=WHITE, width=2)


def draw_curve(draw: ImageDraw.ImageDraw, ox: int, oy: int) -> None:
    draw.arc([(ox + 3, oy + 5), (ox + 17, oy + 15)], 200, 340, fill=WHITE, width=2)


def draw_90(draw: ImageDraw.ImageDraw, ox: int, oy: int) -> None:
    draw.line([(ox + 4, oy + 14), (ox + 4, oy + 6), (ox + 14, oy + 6)], fill=WHITE, width=2)


def draw_box_snap(draw: ImageDraw.ImageDraw, ox: int, oy: int, junction: bool) -> None:
    draw.rectangle([(ox + 5, oy + 5), (ox + 14, oy + 14)], outline=WHITE, width=1)
    if junction:
        draw.ellipse([(ox + 8, oy + 8), (ox + 12, oy + 12)], fill=WHITE)
    else:
        draw.line([(ox + 7, oy + 10), (ox + 13, oy + 10)], fill=WHITE, width=1)


def draw_substation(draw: ImageDraw.ImageDraw, ox: int, oy: int) -> None:
    draw.rectangle([(ox + 4, oy + 6), (ox + 10, oy + 14)], outline=WHITE, width=1)
    draw.rectangle([(ox + 11, oy + 6), (ox + 17, oy + 14)], outline=WHITE, width=1)


def draw_transformer(draw: ImageDraw.ImageDraw, ox: int, oy: int) -> None:
    draw.rectangle([(ox + 6, oy + 5), (ox + 14, oy + 15)], outline=WHITE, width=1)
    draw.line([(ox + 8, oy + 8), (ox + 12, oy + 12)], fill=WHITE, width=1)
    draw.line([(ox + 12, oy + 8), (ox + 8, oy + 12)], fill=WHITE, width=1)


def draw_diesel(draw: ImageDraw.ImageDraw, ox: int, oy: int) -> None:
    draw.rectangle([(ox + 5, oy + 7), (ox + 15, oy + 13)], outline=WHITE, width=1)
    draw.line([(ox + 7, oy + 10), (ox + 13, oy + 10)], fill=WHITE, width=1)
    draw.arc([(ox + 12, oy + 8), (ox + 17, oy + 12)], 270, 30, fill=WHITE, width=1)


def draw_scram(draw: ImageDraw.ImageDraw, ox: int, oy: int) -> None:
    draw.polygon([(ox + 10, oy + 5), (ox + 15, oy + 15), (ox + 5, oy + 15)], outline=WHITE)


def draw_island(draw: ImageDraw.ImageDraw, ox: int, oy: int) -> None:
    draw.polygon([(ox + 10, oy + 4), (ox + 16, oy + 16), (ox + 4, oy + 16)], outline=WHITE)
    draw.line([(ox + 10, oy + 8), (ox + 10, oy + 12)], fill=WHITE, width=2)
    draw.point((ox + 10, oy + 14), fill=WHITE)


def draw_repair(draw: ImageDraw.ImageDraw, ox: int, oy: int) -> None:
    for y in range(6, 15, 3):
        draw.line([(ox + 5, oy + y), (ox + 15, oy + y)], fill=WHITE, width=1)


DRAWERS = [
    draw_line_tool,
    lambda d, x, y: draw_volt(d, x, y, 1),
    lambda d, x, y: draw_volt(d, x, y, 2),
    lambda d, x, y: draw_volt(d, x, y, 3),
    draw_curve,
    draw_90,
    lambda d, x, y: draw_box_snap(d, x, y, False),
    lambda d, x, y: draw_box_snap(d, x, y, True),
    draw_substation,
    draw_transformer,
    draw_diesel,
    draw_scram,
    draw_island,
    draw_repair,
]


def main() -> None:
    img = Image.new("RGBA", (CELL * COLS, CELL * ROWS), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    for idx, drawer in enumerate(DRAWERS):
        col, row = idx % COLS, idx // COLS
        ox, oy = cell_origin(col, row)
        drawer(draw, ox, oy)
    OUT.parent.mkdir(parents=True, exist_ok=True)
    img.save(OUT)
    print(f"wrote {OUT}")


if __name__ == "__main__":
    main()
