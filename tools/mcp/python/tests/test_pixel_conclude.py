"""pixel_conclude — synthetic PNG, no operator screen."""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path

from PIL import Image

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.pixel_pipeline.cli import cli_main
from rust_engine_mcp.pixel_pipeline.conclude import (
    ingest,
    pixel_conclude,
    segment_kmeans,
    segment_meanshift,
    verdict,
)


def _paint(path: Path, width: int, height: int, color_at) -> None:
    raw = bytearray()
    for y in range(height):
        for x in range(width):
            raw.extend(color_at(x, y))
    Image.frombytes("RGB", (width, height), bytes(raw)).save(path, format="PNG")


def _lod(x: int, _y: int) -> tuple[int, int, int]:
    if x < 16:
        return (255, 220, 0)
    return (40, 200, 40)


def _plain(x: int, y: int) -> tuple[int, int, int]:
    if 8 <= x < 12 and 8 <= y < 12:
        return (180, 100, 60)
    return (90, 90, 100)


def _magenta_edge(x: int, _y: int) -> tuple[int, int, int]:
    if x < 4:
        return (255, 0, 255)
    return (240, 240, 240)


def _magenta_center(x: int, y: int) -> tuple[int, int, int]:
    if 12 <= x < 20 and 12 <= y < 20:
        return (255, 0, 255)
    return (240, 240, 240)


def test_ingest_missing(tmp_path: Path) -> None:
    missing = tmp_path / "absent.png"
    try:
        ingest(missing)
    except FileNotFoundError as exc:
        assert "image missing" in str(exc)
    else:
        raise AssertionError("missing png must fail closed")


def test_ingest_rejects_non_png(tmp_path: Path) -> None:
    other = tmp_path / "notes.txt"
    other.write_text("not a png", encoding="utf-8")
    try:
        ingest(other)
    except ValueError as exc:
        assert "png required" in str(exc)
    else:
        raise AssertionError("non-png must fail closed")


def test_kmeans_and_meanshift_are_deterministic() -> None:
    pixels = [(255, 220, 0)] * 32 + [(40, 200, 40)] * 32
    first_k = segment_kmeans(8, 8, pixels)
    second_k = segment_kmeans(8, 8, pixels)
    first_m = segment_meanshift(8, 8, pixels)
    second_m = segment_meanshift(8, 8, pixels)
    assert first_k["labels"] == second_k["labels"]
    assert first_k["seed"] == 17
    assert first_k["k"] == 2
    assert first_m["labels"] == second_m["labels"]
    assert len(set(first_k["labels"])) == 2
    assert len(set(first_m["labels"])) == 2
    source = Path(segment_kmeans.__code__.co_filename).read_text(encoding="utf-8")
    assert "import random" not in source


def test_world_and_art_reject_lod_chrome(tmp_path: Path) -> None:
    image = tmp_path / "lod.png"
    _paint(image, 32, 32, _lod)
    for kind in ("world", "art"):
        body = pixel_conclude(image, kind, out_dir=tmp_path / kind)
        assert body["verdict"] == "reject"
        assert body["ok"] is False
        assert body["honest"] is True
        assert body["matrix_written"] is True
        assert any(r.startswith("dominant_yellow_green_lod_chrome:") for r in body["reasons"])


def test_world_plain_and_yellow_alone_are_green(tmp_path: Path) -> None:
    plain = tmp_path / "plain.png"
    _paint(plain, 32, 32, _plain)
    body = pixel_conclude(plain, "world", out_dir=tmp_path / "plain")
    assert body["verdict"] == "green"
    assert body["ok"] is True
    assert body["honest"] is True

    yellow = tmp_path / "yellow.png"
    _paint(yellow, 32, 32, lambda _x, _y: (255, 220, 0))
    alone = pixel_conclude(yellow, "world", out_dir=tmp_path / "yellow")
    assert alone["verdict"] == "green"


def test_gui_and_ui_flag_clipped_debug_block(tmp_path: Path) -> None:
    image = tmp_path / "clip.png"
    _paint(image, 32, 32, _magenta_edge)
    for kind in ("gui", "ui"):
        body = pixel_conclude(image, kind, out_dir=tmp_path / kind)
        assert body["verdict"] == "flag"
        assert body["ok"] is False
        assert any(r.startswith("overflow_or_clipped_debug_block:") for r in body["reasons"])


def test_gui_centered_debug_block_is_green(tmp_path: Path) -> None:
    image = tmp_path / "center.png"
    _paint(image, 32, 32, _magenta_center)
    body = pixel_conclude(image, "gui", out_dir=tmp_path / "center")
    assert body["verdict"] == "green"
    assert body["reasons"] == []


def test_verdict_refuses_green_without_matrix() -> None:
    judged = verdict("world", 8, 8, [], [], matrix_written=False)
    assert judged["verdict"] == "incomplete"
    assert judged["verdict"] != "green"
    assert judged["honest"] is True
    assert "matrix_not_written" in judged["reasons"]


def test_matrix_sha256_counts_and_repeat(tmp_path: Path) -> None:
    image = tmp_path / "plain.png"
    _paint(image, 32, 32, _plain)
    out_a = tmp_path / "a"
    out_b = tmp_path / "b"
    first = pixel_conclude(image, "world", out_dir=out_a)
    second = pixel_conclude(image, "world", out_dir=out_b)
    raw = Image.open(image).convert("RGB").tobytes()
    assert first["sha256"] == hashlib.sha256(raw).hexdigest()
    matrix_path = Path(first["matrix"])
    matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
    assert matrix["kind"] == "world"
    assert matrix["sha256"] == first["sha256"]
    assert matrix["kmeans"]["clusters"]
    assert matrix["meanshift"]["clusters"]
    for cluster in matrix["kmeans"]["clusters"]:
        assert "count" in cluster and "centroid" in cluster and "bbox" in cluster
    map_path = Path(first["segmentation_map"])
    label = Image.open(map_path).convert("RGB")
    assert label.size == (32, 32)
    assert json.loads(Path(second["matrix"]).read_text(encoding="utf-8")) == matrix


def test_cli_module_green_and_missing(tmp_path: Path) -> None:
    image = tmp_path / "plain.png"
    _paint(image, 16, 16, lambda _x, _y: (90, 90, 100))
    assert cli_main(["--image", str(image), "--kind", "world", "--out", str(tmp_path / "cli")]) == 0
    code = cli_main(["--image", str(tmp_path / "missing.png"), "--kind", "gui"])
    assert code == 2


def test_cli_subprocess_prog(tmp_path: Path) -> None:
    image = tmp_path / "plain.png"
    _paint(image, 16, 16, lambda _x, _y: (90, 90, 100))
    proc = subprocess.run(
        [
            sys.executable,
            "-m",
            "rust_engine_mcp.pixel_pipeline",
            "--image",
            str(image),
            "--kind",
            "art",
            "--out",
            str(tmp_path / "mod"),
        ],
        cwd=repo_root() / "tools/mcp/python",
        capture_output=True,
        text=True,
        check=False,
    )
    assert proc.returncode == 0, proc.stderr or proc.stdout
    body = json.loads(proc.stdout)
    assert body["verdict"] == "green"
    assert body["kind"] == "art"
