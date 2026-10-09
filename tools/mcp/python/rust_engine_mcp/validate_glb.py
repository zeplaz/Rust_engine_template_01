"""Minimal glb checks without heavy deps — fast micro-tool."""

from __future__ import annotations

import json
import struct
from dataclasses import dataclass, field
from pathlib import Path


@dataclass
class ValidationReport:
    valid: bool
    path: str
    issues: list[str] = field(default_factory=list)
    vertex_count: int | None = None
    file_bytes: int = 0

    def to_dict(self) -> dict:
        return {
            "valid": self.valid,
            "path": self.path,
            "issues": self.issues,
            "vertex_count": self.vertex_count,
            "file_bytes": self.file_bytes,
        }


def _read_glb_json_chunk(data: bytes) -> tuple[dict | None, list[str]]:
    """Return JSON chunk plus structural issues. Never raises on a corrupt file."""
    issues: list[str] = []
    if len(data) < 12:
        return None, ["file too small for glb header"]
    magic, version, length = struct.unpack("<4sII", data[:12])
    if magic != b"glTF":
        return None, ["missing glTF magic header"]
    if version != 2:
        return None, [f"unsupported glTF version {version}"]
    if length != len(data):
        issues.append(f"glb length header {length} != file size {len(data)}")
    offset = 12
    found: dict | None = None
    while offset + 8 <= len(data) and offset < length:
        chunk_len, chunk_type = struct.unpack("<I4s", data[offset : offset + 8])
        offset += 8
        if chunk_len > len(data) - offset:
            issues.append("glb chunk truncated")
            return None, issues
        chunk = data[offset : offset + chunk_len]
        offset += chunk_len
        if chunk_type == b"JSON":
            try:
                parsed = json.loads(chunk.decode("utf-8"))
            except (UnicodeDecodeError, json.JSONDecodeError):
                issues.append("glb JSON chunk is not valid JSON")
                return None, issues
            if not isinstance(parsed, dict):
                issues.append("glb JSON chunk is not an object")
                return None, issues
            found = parsed
            break
    if found is None:
        issues.append("could not parse glb JSON chunk")
    return found, issues


def validate_glb(path: Path, max_vertices: int = 50_000) -> ValidationReport:
    path = path.resolve()
    issues: list[str] = []
    if not path.is_file():
        return ValidationReport(valid=False, path=str(path), issues=["file not found"])

    data = path.read_bytes()
    report = ValidationReport(valid=True, path=str(path), file_bytes=len(data))

    gltf, structural = _read_glb_json_chunk(data)
    issues.extend(structural)
    if gltf is not None:
        meshes = gltf.get("meshes") or []
        total_verts = 0
        accessors = gltf.get("accessors") or []
        for mesh in meshes:
            for prim in mesh.get("primitives") or []:
                acc_idx = prim.get("attributes", {}).get("POSITION")
                if acc_idx is not None and acc_idx < len(accessors):
                    total_verts += int(accessors[acc_idx].get("count") or 0)
        report.vertex_count = total_verts
        if total_verts == 0:
            issues.append("no vertices in mesh")
        elif total_verts > max_vertices:
            issues.append(f"vertex count {total_verts} exceeds budget {max_vertices}")

    if path.suffix.lower() != ".glb":
        issues.append("expected .glb extension")

    report.issues = issues
    report.valid = len(issues) == 0
    return report
