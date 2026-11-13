from __future__ import annotations

import json

from rust_engine_mcp import library
from rust_engine_mcp.paths import repo_root


def test_library_register_rebuild_all():
    result = library.write_module_index()
    assert result["entry_count"] >= 10
    assert (repo_root() / "assets/configs/buildings/_module_index.ron").is_file()
    assert (repo_root() / "assets/configs/buildings/_module_index.json").is_file()




def test_library_search_style_pack():
    library.write_module_index()
    rows = library.search_modules(style_pack="style_rural")
    assert any(r["module_id"] == "wall_wood_1u" for r in rows)


def test_register_module_wall_concrete():
    job_id = "wall_concrete_2u_run001"
    mod_dir = repo_root() / "assets/models/modules" / job_id
    if not mod_dir.is_dir():
        return
    result = library.register_module(job_id)
    assert result["registered"] == "wall_concrete_2u"
    assert result["job_id"] == job_id






def test_index_kit_lod0_001_entries_retired():
    """BQ-LOD0-TWIN-PRUNE-001 — lod0 twins leave the active index (archive quarantine)."""
    library.write_module_index()
    rows = library.search_modules(batch_id="kit_lod0_001")
    assert rows == []
    data = json.loads(
        (repo_root() / "assets/configs/buildings/_module_index.json").read_text(encoding="utf-8")
    )
    lod0 = [e for e in data["entries"] if e.get("development_tier") == "lod0"]
    assert lod0 == []
    archive = (
        repo_root()
        / "assets/archive/kit_lod0_twins_retired_2026-09/modules"
        / "wall_brick_1u_lod0_run001"
    )
    # Idempotent: either already archived by prune gate, or still pending a first run.
    assert archive.is_dir() or (repo_root() / "assets/models/modules/wall_brick_1u_lod0_run001").is_dir()




def test_geometry_job_accepts_window_and_prop():
    from rust_engine_mcp import schemas
    from rust_engine_mcp.paths import repo_root as root

    for name in ("window_smoke.example.json", "prop_smoke.example.json"):
        path = root() / "tools/mcp/schemas/examples" / name
        if path.is_file():
            schemas.validate_geometry_job(schemas.load_json_file(path))


def test_seed_required_when_material_profile_set():
    import pytest

    from rust_engine_mcp.blender_runner import _enforce_seed_in_params

    with pytest.raises(ValueError, match="seed"):
        _enforce_seed_in_params(
            {"params": {"width_m": 1.0, "material_profile": "steel_panel_01"}}
        )
    _enforce_seed_in_params({"params": {"width_m": 1.0, "seed": 42}})
