"""LG-5 landscape topology atlas — deterministic keyframes + tile batch run."""

from __future__ import annotations

import hashlib
from pathlib import Path
from typing import Any

from PIL import Image, ImageDraw

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.tile_pipeline import tile_batch_run

BATCH_JSON = (
    repo_root()
    / "tools/mcp/schemas/examples/tile_batch_landscape_lg5_pilot_v1.json"
)
KEYFRAME_DIR = (
    repo_root() / "assets/staging/tiles/keyframe_stills/tile_landscape_lg5_pilot_v1"
)
SEED = 550005
SIZE = 64


def _rng(seed: int, tag: str) -> int:
    h = hashlib.sha256(f"{seed}:{tag}".encode()).hexdigest()
    return int(h[:8], 16)


def _draw_patch(im: Image.Image, draw: ImageDraw.ImageDraw) -> None:
    cx, cy = SIZE // 2, SIZE // 2
    w, h = SIZE - 8, SIZE - 12
    draw.polygon(
        [(cx, cy - h // 2), (cx + w // 2, cy), (cx, cy + h // 2), (cx - w // 2, cy)],
        fill=(34, 120, 52, 255),
        outline=(18, 70, 30, 255),
    )
    for i in range(3):
        ox = (_rng(SEED, f"patch{i}") % 11) - 5
        oy = (_rng(SEED, f"patchy{i}") % 9) - 4
        draw.ellipse(
            [cx + ox - 6, cy + oy - 4, cx + ox + 6, cy + oy + 4],
            fill=(48, 150, 64, 220),
        )


def _draw_corridor(im: Image.Image, draw: ImageDraw.ImageDraw) -> None:
    draw.rectangle([0, 0, SIZE, SIZE], fill=(58, 92, 48, 255))
    band = SIZE // 3
    y0 = (SIZE - band) // 2
    draw.rectangle([4, y0, SIZE - 4, y0 + band], fill=(120, 82, 44, 255))
    draw.line([(0, y0 + band // 2), (SIZE, y0 + band // 2)], fill=(180, 140, 72, 255), width=2)


def _draw_ring(im: Image.Image, draw: ImageDraw.ImageDraw) -> None:
    draw.rectangle([0, 0, SIZE, SIZE], fill=(40, 55, 38, 255))
    cx, cy = SIZE // 2, SIZE // 2
    outer = SIZE // 2 - 4
    inner = outer - 10
    draw.ellipse(
        [cx - outer, cy - outer, cx + outer, cy + outer],
        outline=(210, 170, 60, 255),
        width=4,
    )
    draw.ellipse(
        [cx - inner, cy - inner, cx + inner, cy + inner],
        outline=(210, 170, 60, 180),
        width=2,
    )


def write_landscape_lg5_keyframes(out_dir: Path | None = None) -> list[Path]:
    folder = out_dir or KEYFRAME_DIR
    folder.mkdir(parents=True, exist_ok=True)
    writers = {
        "topology_patch.png": _draw_patch,
        "topology_corridor.png": _draw_corridor,
        "topology_ring.png": _draw_ring,
    }
    written: list[Path] = []
    for name, fn in writers.items():
        im = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
        draw = ImageDraw.Draw(im)
        fn(im, draw)
        path = folder / name
        im.save(path, format="PNG")
        written.append(path)
    return written


def run_landscape_lg5_atlas_batch(*, refresh_keyframes: bool = True) -> dict[str, Any]:
    if refresh_keyframes:
        write_landscape_lg5_keyframes()
    if not BATCH_JSON.is_file():
        return {"ok": False, "error": f"missing batch json: {BATCH_JSON}"}
    result = tile_batch_run(BATCH_JSON)
    witness_path = (
        repo_root()
        / "debug_runs/art_pipeline/tile_tile_landscape_lg5_pilot_v1_live.json"
    )
    result["art_pipeline_witness"] = str(witness_path) if witness_path.is_file() else None
    result["witness_green"] = False
    if witness_path.is_file():
        import json

        body = json.loads(witness_path.read_text(encoding="utf-8"))
        result["witness_green"] = bool(body.get("green"))
    return result


def main() -> int:
    result = run_landscape_lg5_atlas_batch()
    print(result)
    return 0 if result.get("ok") and result.get("witness_green") else 1


if __name__ == "__main__":
    raise SystemExit(main())
