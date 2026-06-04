from __future__ import annotations

import json

from rust_engine_mcp import library
from rust_engine_mcp.paths import repo_root


def test_library_register_rebuild_all():
    result = library.write_module_index()
    assert result["entry_count"] >= 10
    assert (repo_root() / "assets/configs/buildings/_module_index.ron").is_file()
    assert (repo_root() / "assets/configs/buildings/_module_index.json").is_file()


def test_index_has_kit_greybox_001_entries():
    library.write_module_index()
    rows = library.search_modules(batch_id="kit_greybox_001")
    assert len(rows) == 10
    ids = {r["module_id"] for r in rows}
    assert "wall_concrete_2u" in ids
    assert "door_shop_1u" in ids
    for row in rows:
        assert row["module_id"] != row["job_id"]
        assert row["job_id"].endswith("_run001")
        assert row["glb"] == row["glb_path"]


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


def test_asset_id_job_id_mapping_in_json_mirror():
    library.write_module_index()
    data = json.loads(
        (repo_root() / "assets/configs/buildings/_module_index.json").read_text(encoding="utf-8")
    )
    by_id = {e["module_id"]: e for e in data["entries"]}
    assert by_id["wall_concrete_2u"]["job_id"] == "wall_concrete_2u_run001"


def test_index_all_kit_greybox_marked_smoke():
    library.write_module_index()
    data = json.loads(
        (repo_root() / "assets/configs/buildings/_module_index.json").read_text(encoding="utf-8")
    )
    greybox = [e for e in data["entries"] if str(e.get("batch_id", "")).startswith("kit_greybox")]
    assert len(greybox) >= 30
    for row in greybox:
        assert row["development_tier"] == "smoke"
        assert row["stylepack_visible"] is False
        assert row["pbr_status"] == "none"


def test_index_kit_lod0_001_entries():
    library.write_module_index()
    rows = library.search_modules(batch_id="kit_lod0_001")
    assert len(rows) == 5
    for row in rows:
        assert row["development_tier"] == "lod0"
        assert row["stylepack_visible"] is True
        assert row["job_id"] in library.KIT_LOD0_001_JOB_IDS


def test_replaced_by_on_superseded_smoke_rows():
    library.write_module_index()
    data = json.loads(
        (repo_root() / "assets/configs/buildings/_module_index.json").read_text(encoding="utf-8")
    )
    smoke = [e for e in data["entries"] if e.get("replaced_by")]
    assert any(
        e["module_id"] == "wall_concrete_1u"
        and e["job_id"] == "wall_concrete_1u_run001"
        and e["replaced_by"] == "wall_concrete_1u"
        for e in smoke
    )
    assert any(
        e["module_id"] == "door_residential_1u" and e["replaced_by"] == "door_residential" for e in smoke
    )


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
