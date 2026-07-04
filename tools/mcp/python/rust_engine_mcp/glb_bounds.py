"""GLB accessor bounds helpers — shared by BQ-F1/C2/C3 validators."""

from __future__ import annotations

import json
import struct
from pathlib import Path
from typing import Any


def read_glb_json(path: Path) -> dict[str, Any] | None:
    if not path.is_file():
        return None
    data = path.read_bytes()
    if len(data) < 12 or data[:4] != b"glTF":
        return None
    offset = 12
    while offset + 8 <= len(data):
        chunk_len, chunk_type = struct.unpack("<I4s", data[offset : offset + 8])
        offset += 8
        chunk = data[offset : offset + chunk_len]
        offset += chunk_len
        if chunk_type == b"JSON":
            return json.loads(chunk.decode("utf-8"))
    return None


def glb_position_bounds(path: Path) -> dict[str, list[float]] | None:
    gltf = read_glb_json(path)
    if gltf is None:
        return None
    accessors = gltf.get("accessors") or []
    meshes = gltf.get("meshes") or []
    mins: list[float] | None = None
    maxs: list[float] | None = None
    for mesh in meshes:
        for prim in mesh.get("primitives") or []:
            acc_idx = prim.get("attributes", {}).get("POSITION")
            if acc_idx is None or acc_idx >= len(accessors):
                continue
            acc = accessors[acc_idx]
            lo = acc.get("min")
            hi = acc.get("max")
            if not lo or not hi:
                continue
            if mins is None:
                mins = [float(x) for x in lo]
                maxs = [float(x) for x in hi]
            else:
                for i in range(min(3, len(lo), len(mins))):
                    mins[i] = min(mins[i], float(lo[i]))
                    maxs[i] = max(maxs[i], float(hi[i]))
    if mins is None or maxs is None:
        return None
    return {"min": mins, "max": maxs}


def bounds_extent(bounds: dict[str, list[float]]) -> tuple[float, float, float]:
    mn = bounds.get("min") or [0.0, 0.0, 0.0]
    mx = bounds.get("max") or [0.0, 0.0, 0.0]
    return (
        float(mx[0]) - float(mn[0]),
        float(mx[1]) - float(mn[1]),
        float(mx[2]) - float(mn[2]),
    )


def near(a: float, b: float, tol: float) -> bool:
    return abs(a - b) <= tol
