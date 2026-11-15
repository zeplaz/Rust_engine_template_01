"""RND-20 S2 — perceptual heatmap ramp LUT determinism + monotonic QC."""

from __future__ import annotations

import struct
from pathlib import Path

import pytest

from rust_engine_mcp import perceptual_ramp
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.schemas import load_json_file

EXAMPLE_SPEC = (
    repo_root() / "tools/mcp/schemas/examples/heatmap_ramp_default_overlay_v1.json"
)


def test_default_lut_byte_identical_across_runs() -> None:
    spec = load_json_file(EXAMPLE_SPEC)
    ramp = perceptual_ramp.generate_ramp_from_spec(spec)
    lut_a = perceptual_ramp.pack_lut_bytes(ramp)
    lut_b = perceptual_ramp.pack_lut_bytes(perceptual_ramp.generate_ramp_from_spec(spec))
    assert lut_a == lut_b
    assert len(lut_a) == 12 + 16 * 12
    a = perceptual_ramp.export_heatmap_ramp(spec, write_files=False)
    b = perceptual_ramp.export_heatmap_ramp(spec, write_files=False)
    assert a["lut_bytes_len"] == b["lut_bytes_len"] == len(lut_a)
    assert a["green"] is True


def test_pack_unpack_roundtrip() -> None:
    ramp = perceptual_ramp.default_heatmap_ramp_linear()
    data = perceptual_ramp.pack_lut_bytes(ramp)
    back = perceptual_ramp.unpack_lut_bytes(data)
    assert len(back) == 16
    for i, row in enumerate(back):
        for j, ch in enumerate(row):
            assert row[j] == pytest.approx(ramp[i][j], rel=0, abs=1e-7)


def test_qc_rejects_non_monotonic_ramp() -> None:
    bad: list[perceptual_ramp.LinearStop] = [
        (0.05, 0.05, 0.05),
        (0.40, 0.40, 0.40),
        (0.30, 0.30, 0.30),
        (0.90, 0.90, 0.90),
    ]
    qc = perceptual_ramp.qc_ramp_lightness_monotonic(bad)
    assert qc["ok"] is False
    assert qc["failed_index"] == 2
    assert qc["reason"] == "lightness_not_strictly_increasing"


def test_export_rejects_non_monotonic_spec(tmp_path: Path) -> None:
    spec = {
        "schema_version": 1,
        "ramp_id": "bad_monotonic",
        "preset": "linear_stops",
        "stop_count": 4,
        "linear_stops": [
            [0.05, 0.05, 0.05],
            [0.40, 0.40, 0.40],
            [0.30, 0.30, 0.30],
            [0.90, 0.90, 0.90],
        ],
    }
    out = perceptual_ramp.export_heatmap_ramp(spec, out_dir=tmp_path, write_files=True)
    assert out["green"] is False
    assert out.get("written") is False
    assert not (tmp_path / "bad_monotonic_v1.lut").is_file()


def test_qc_lut_file_rejects_packed_bad_ramp(tmp_path: Path) -> None:
    bad = [
        (0.1, 0.1, 0.1),
        (0.5, 0.5, 0.5),
        (0.2, 0.2, 0.2),
    ]
    path = tmp_path / "bad.lut"
    path.write_bytes(perceptual_ramp.pack_lut_bytes(bad))
    body = perceptual_ramp.qc_lut_file(path)
    assert body["green"] is False


def test_lut_header_layout() -> None:
    data = perceptual_ramp.pack_lut_bytes(perceptual_ramp.default_heatmap_ramp_linear())
    assert data[:8] == perceptual_ramp.LUT_MAGIC
    version, stops = struct.unpack("<HH", data[8:12])
    assert version == 1
    assert stops == 16