from __future__ import annotations

import json
from pathlib import Path

import pytest

from rust_engine_mcp import paths, schemas, validate_glb


def test_repo_root_exists():
    root = paths.repo_root()
    assert (root / "Cargo.toml").is_file()


def test_blender_exe_configured():
    exe = paths.blender_exe()
    assert exe.is_file()
    assert exe.name.lower() == "blender.exe"


def test_validate_asset_spec_example():
    spec_path = paths.repo_root() / "assets/staging/specs/wall_brick_1u.example.json"
    data = schemas.load_json_file(spec_path)
    schemas.validate_asset_spec(data)
    assert data["asset_id"] == "wall_brick_1u"


def test_validate_geometry_job_example():
    job_path = paths.repo_root() / "tools/mcp/schemas/examples/wall_job.example.json"
    data = schemas.load_json_file(job_path)
    schemas.validate_geometry_job(data)


def test_promote_resolves_spec_ref_from_example_job():
    from rust_engine_mcp.promote import _resolve_spec_path

    spec = _resolve_spec_path("wall_brick_1u_example")
    assert spec is not None
    assert spec.name == "wall_brick_1u.example.json"
