"""APS-MAT-STUDIO-PHASE-A witness tests."""

from __future__ import annotations

from pathlib import Path

from rust_engine_mcp.material_studio_preview import (
    APS_MATERIAL_STUDIO_WITNESS,
    preview_modes_for_profile,
    write_material_studio_witness,
)
from rust_engine_mcp.paths import repo_root


def test_preview_modes_sphere_wall_building() -> None:
    modes = preview_modes_for_profile("steel_panel_01")
    assert modes["ok"]
    for key in ("sphere", "wall_strip", "building_section"):
        assert modes[key]["ok"]
        path = repo_root() / modes[key]["path"]
        assert path.is_file(), key


def test_material_studio_witness_written() -> None:
    result = write_material_studio_witness()
    assert result.get("ok")
    out = repo_root() / APS_MATERIAL_STUDIO_WITNESS
    assert out.is_file()
    assert result.get("ui", {}).get("list_rows") == "scrollable_thumb_rows"
    assert "aps_mat_003" in result
