"""Second pass of MCP Python toolchain logic gaps."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

import pytest

from rust_engine_mcp.assembly import _module_width_cells
from rust_engine_mcp.blender_runner import _enforce_seed_in_params
from rust_engine_mcp.effect_promote import _verify_staging_hashes
from rust_engine_mcp.library import _style_tags_from_spec
from rust_engine_mcp.module_contract import grid_units_from_width_m
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.promote import promote_module
from rust_engine_mcp.spine_build_graph import EXAMPLE_REL, graph_errors
from rust_engine_mcp.spine_render_contract import contract_errors
from rust_engine_mcp.variant_set import _ron_to_dict, apply_json_patch


def test_promote_picks_sorted_glb_when_model_missing(tmp_path: Path, monkeypatch) -> None:
    import rust_engine_mcp.promote as promote

    job = "gap_pick_glb"
    staging = tmp_path / "assets" / "staging" / job
    staging.mkdir(parents=True)
    (staging / "z_late.glb").write_bytes(b"ZZZ")
    (staging / "a_early.glb").write_bytes(b"AAA")
    monkeypatch.setattr(promote, "repo_root", lambda: tmp_path)
    monkeypatch.setattr(promote, "staging_root", lambda: tmp_path / "assets" / "staging")
    manifest = promote_module(job, force=True, register=False, allow_smoke=True)
    copied = (tmp_path / manifest["glb"]).read_bytes()
    assert copied == b"AAA"


def test_json_patch_add_rejects_index_past_end() -> None:
    doc = {"variants": [{"variant_key": "a"}]}
    with pytest.raises(ValueError, match="outside"):
        apply_json_patch(doc, [{"op": "add", "path": "/variants/5", "value": {"variant_key": "z"}}])
    patched = apply_json_patch(
        doc, [{"op": "add", "path": "/variants/-", "value": {"variant_key": "b"}}]
    )
    assert [row["variant_key"] for row in patched["variants"]] == ["a", "b"]
    assert len(doc["variants"]) == 1


def test_shader_dest_cannot_escape_staging(tmp_path: Path) -> None:
    staging = tmp_path / "fx"
    shaders = staging / "shaders"
    shaders.mkdir(parents=True)
    body = b"fn main() {}\n"
    (shaders / "draw.wgsl").write_bytes(body)
    digest = hashlib.sha256(body).hexdigest()
    _verify_staging_hashes(
        staging,
        {"shader_hashes": {"draw": {"dest": "shaders/draw.wgsl", "sha256": digest}}},
    )
    with pytest.raises(ValueError, match="escapes"):
        _verify_staging_hashes(
            staging,
            {"shader_hashes": {"draw": {"dest": "../secret.wgsl", "sha256": digest}}},
        )


def test_module_width_uses_2u_token_not_suffix() -> None:
    assert _module_width_cells({"module_id": "wall_brick_2u"}) == 2
    assert _module_width_cells({"module_id": "roof_2u_shed"}) == 2
    assert _module_width_cells({"module_id": "wall_12u"}) == 1
    assert _module_width_cells({"module_id": "foo_2unit"}) == 1


def test_grid_units_round_half_away_from_zero() -> None:
    assert grid_units_from_width_m(8) == 2
    assert grid_units_from_width_m(6) == 2
    assert grid_units_from_width_m(10) == 3
    assert grid_units_from_width_m(18) == 5
    assert grid_units_from_width_m(0) == 1


def test_variation_job_rejects_null_or_bool_seed() -> None:
    _enforce_seed_in_params({"params": {"width_m": 4}})
    _enforce_seed_in_params({"params": {"material_profile": "brick_red_01", "seed": 0}})
    with pytest.raises(ValueError, match="integer"):
        _enforce_seed_in_params({"params": {"material_profile": "brick_red_01", "seed": None}})
    with pytest.raises(ValueError, match="integer"):
        _enforce_seed_in_params({"params": {"prop_kind": "vent", "seed": True}})


def test_render_contract_seed_must_be_int() -> None:
    job = {
        "schema_version": 1,
        "job": "render_variant",
        "asset": "x",
        "variant": "clean",
        "facing": 0,
        "seed": 0,
        "role": "smoke",
        "ship": False,
        "render": {
            "method": "blender_orthographic_iso",
            "bake_source": "smoke_ortho_headless",
        },
    }
    assert contract_errors(job) == []
    job["seed"] = True
    assert any("seed" in err for err in contract_errors(job))
    job["seed"] = "43001"
    assert any("seed" in err for err in contract_errors(job))


def test_ship_build_graph_rejects_non_integer_seed() -> None:
    data = json.loads((repo_root() / EXAMPLE_REL).read_text(encoding="utf-8"))
    data["ship"] = True
    data["seed"] = 0
    assert not any("seed" in err for err in graph_errors(data))
    data["seed"] = True
    assert any("seed" in err for err in graph_errors(data))
    data["seed"] = "42"
    assert any("seed" in err for err in graph_errors(data))


def test_style_tags_match_whole_tokens() -> None:
    plywood = _style_tags_from_spec({"asset_id": "plywood_sheet", "material_profile": ""})
    assert "wood" not in plywood
    wood = _style_tags_from_spec({"asset_id": "wood_plank_wall", "material_profile": ""})
    assert "wood" in wood
    glass = _style_tags_from_spec({"asset_id": "fiberglass_panel", "material_profile": ""})
    assert "glass" not in glass
    real_glass = _style_tags_from_spec({"asset_id": "glass_curtain_2u", "material_profile": ""})
    assert "glass" in real_glass


def test_variant_ron_keeps_colon_inside_strings() -> None:
    text = """(
    variant_set_id: "set",
    note: "clean:day",
)
"""
    data = _ron_to_dict(text)
    assert data["note"] == "clean:day"
    assert data["variant_set_id"] == "set"
