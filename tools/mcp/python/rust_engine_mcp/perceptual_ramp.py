"""RND-20 / PLAN-REF-BOOKS S2 — deterministic heatmap ramp LUT + RON export + lightness QC."""

from __future__ import annotations

import struct
from pathlib import Path
from typing import Any

from .paths import repo_root
from .schemas import load_json_file, validate_heatmap_ramp

TASK_ID = "PLAN-REF-BOOKS-S2"
REF_ROW = "RND-20"
LUT_MAGIC = b"RND20LUT"
LUT_VERSION = 1
DEFAULT_STOP_COUNT = 16
LIGHTNESS_EPS = 1e-5

LinearStop = tuple[float, float, float]


def srgb_channel_to_linear(c: float) -> float:
    c = max(0.0, min(1.0, c))
    if c <= 0.04045:
        return c / 12.92
    return ((c + 0.055) / 1.055) ** 2.4


def linear_channel_to_srgb(c: float) -> float:
    c = max(0.0, min(1.0, c))
    if c <= 0.0031308:
        return 12.92 * c
    return 1.055 * (c ** (1.0 / 2.4)) - 0.055


def linear_rgb_to_lightness(rgb: LinearStop) -> float:
    return 0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]


def default_heatmap_ramp_linear(*, stop_count: int = DEFAULT_STOP_COUNT) -> list[LinearStop]:
    """Mirror ``src/gui/style/heatmap_ramp.rs::default_heatmap_ramp_linear``."""
    if stop_count < 2:
        raise ValueError("stop_count must be >= 2")
    ramp: list[LinearStop] = []
    denom = float(stop_count - 1)
    for i in range(stop_count):
        t = i / denom
        r = srgb_channel_to_linear(0.05 + 0.82 * t)
        g = srgb_channel_to_linear(0.07 + 0.58 * (t ** 1.08))
        b = srgb_channel_to_linear(0.20 * (1.0 - 0.65 * t))
        ramp.append((r, g, b))
    return ramp


def ramp_from_linear_stops(stops: list[LinearStop]) -> list[LinearStop]:
    if len(stops) < 2:
        raise ValueError("linear_stops must have at least 2 entries")
    out: list[LinearStop] = []
    for row in stops:
        if len(row) != 3:
            raise ValueError("each linear stop must be [r, g, b]")
        out.append((float(row[0]), float(row[1]), float(row[2])))
    return out


def generate_ramp_from_spec(spec: dict[str, Any]) -> list[LinearStop]:
    validate_heatmap_ramp(spec)
    stop_count = int(spec.get("stop_count") or DEFAULT_STOP_COUNT)
    preset = str(spec.get("preset") or "engine_default")
    if preset == "engine_default":
        if stop_count != DEFAULT_STOP_COUNT:
            raise ValueError("engine_default preset requires stop_count 16")
        return default_heatmap_ramp_linear(stop_count=stop_count)
    if preset == "linear_stops":
        raw = spec.get("linear_stops")
        if not isinstance(raw, list) or not raw:
            raise ValueError("linear_stops preset requires linear_stops[]")
        stops = ramp_from_linear_stops(raw)
        if len(stops) != stop_count:
            raise ValueError("linear_stops length must match stop_count")
        return stops
    raise ValueError(f"unknown preset: {preset}")


def qc_ramp_lightness_monotonic(
    ramp: list[LinearStop],
    *,
  eps: float = LIGHTNESS_EPS,
) -> dict[str, Any]:
    """QC gate — rejects ramps whose Rec.709 lightness does not strictly increase."""
    if len(ramp) < 2:
        return {
            "ok": False,
            "reason": "too_few_stops",
            "failed_index": None,
            "lightness": [],
        }
    lightness = [linear_rgb_to_lightness(row) for row in ramp]
    prev = -1.0
    for i, l in enumerate(lightness):
        if not (l > prev + eps):
            return {
                "ok": False,
                "reason": "lightness_not_strictly_increasing",
                "failed_index": i,
                "lightness": lightness,
                "previous": prev,
                "at_fail": l,
            }
        prev = l
    return {"ok": True, "reason": None, "lightness": lightness}


def pack_lut_bytes(ramp: list[LinearStop]) -> bytes:
    """Little-endian binary LUT — same ramp always yields byte-identical output."""
    parts = [LUT_MAGIC, struct.pack("<HH", LUT_VERSION, len(ramp))]
    for r, g, b in ramp:
        parts.append(struct.pack("<fff", r, g, b))
    return b"".join(parts)


def unpack_lut_bytes(data: bytes) -> list[LinearStop]:
    if len(data) < 12 or data[:8] != LUT_MAGIC:
        raise ValueError("invalid LUT magic")
    version, stop_count = struct.unpack("<HH", data[8:12])
    if version != LUT_VERSION:
        raise ValueError(f"unsupported LUT version {version}")
    expected = 12 + stop_count * 12
    if len(data) != expected:
        raise ValueError(f"LUT size mismatch: got {len(data)}, expected {expected}")
    ramp: list[LinearStop] = []
    offset = 12
    for _ in range(stop_count):
        r, g, b = struct.unpack("<fff", data[offset : offset + 12])
        ramp.append((r, g, b))
        offset += 12
    return ramp


def format_ron_entry(
    *,
    ramp_id: str,
    lut_rel: str,
    stop_count: int,
    qc: dict[str, Any],
) -> str:
    mono = "true" if qc.get("ok") else "false"
    return (
        f"// {REF_ROW} — perceptual heatmap ramp LUT (toolchain-generated; do not hand-edit stops)\n"
        f"(\n"
        f"    schema_version: 1,\n"
        f"    ramp_id: \"{ramp_id}\",\n"
        f"    ref_book_row: \"{REF_ROW}\",\n"
        f"    stop_count: {stop_count},\n"
        f"    color_space: \"linear_rgb\",\n"
        f"    lut_path: \"{lut_rel}\",\n"
        f"    monotonic_lightness: {mono},\n"
        f")\n"
    )


def export_heatmap_ramp(
    spec: dict[str, Any],
    *,
    out_dir: Path | None = None,
    write_files: bool = True,
) -> dict[str, Any]:
    """Generate ramp, run QC, optionally write ``.lut`` + ``.ron`` under staging."""
    ramp = generate_ramp_from_spec(spec)
    qc = qc_ramp_lightness_monotonic(ramp)
    lut_bytes = pack_lut_bytes(ramp)
    ramp_id = str(spec["ramp_id"])
    root = repo_root()
    base = out_dir or (root / "assets" / "staging" / "ramps")
    lut_name = f"{ramp_id}_v1.lut"
    ron_name = f"{ramp_id}_v1.ron"
    lut_path = base / lut_name
    ron_path = base / ron_name
    try:
        lut_rel = lut_path.relative_to(root).as_posix()
    except ValueError:
        lut_rel = lut_path.as_posix()
    ron_body = format_ron_entry(
        ramp_id=ramp_id,
        lut_rel=lut_rel,
        stop_count=len(ramp),
        qc=qc,
    )
    result: dict[str, Any] = {
        "task_id": TASK_ID,
        "ref_book_row": REF_ROW,
        "ramp_id": ramp_id,
        "stop_count": len(ramp),
        "lut_sha256_prefix": None,
        "lut_bytes_len": len(lut_bytes),
        "qc": qc,
        "lut_path": str(lut_path),
        "ron_path": str(ron_path),
        "green": bool(qc.get("ok")),
    }
    if write_files:
        if not qc.get("ok"):
            result["written"] = False
            result["error"] = "qc_failed: non-monotonic lightness"
            return result
        base.mkdir(parents=True, exist_ok=True)
        lut_path.write_bytes(lut_bytes)
        ron_path.write_text(ron_body, encoding="utf-8", newline="\n")
        result["written"] = True
    return result


def export_heatmap_ramp_from_path(
    spec_path: Path,
    *,
    out_dir: Path | None = None,
    write_files: bool = True,
) -> dict[str, Any]:
    spec = load_json_file(spec_path)
    return export_heatmap_ramp(spec, out_dir=out_dir, write_files=write_files)


def qc_lut_file(path: Path) -> dict[str, Any]:
    ramp = unpack_lut_bytes(path.read_bytes())
    qc = qc_ramp_lightness_monotonic(ramp)
    return {"path": str(path), "stop_count": len(ramp), "qc": qc, "green": bool(qc.get("ok"))}