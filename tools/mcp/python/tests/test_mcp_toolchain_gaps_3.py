"""Third pass of MCP Python toolchain logic gaps."""

from __future__ import annotations

import json
from pathlib import Path

import pytest
from PIL import Image

from rust_engine_mcp.assembly import _index_by_module_id
from rust_engine_mcp.build_set_guards import MAX_VIOLATIONS, example_teachable_audit
from rust_engine_mcp.effect_preview_capture import _capture_seed
from rust_engine_mcp.effect_registry_browse import EffectBrowseEntry
from rust_engine_mcp.grammar_layers import apply_facade_layer
from rust_engine_mcp.library import entry_from_promoted
from rust_engine_mcp.material_textures import _category_for_profile
from rust_engine_mcp.pixel_pipeline.conclude import pixel_conclude
from rust_engine_mcp.promote import promote_module
from rust_engine_mcp.scenario_trigger_write import AssignBlockedError, assign_effect_to_marker
from rust_engine_mcp.tile_pipeline import _render_seed
from rust_engine_mcp.validators.tile_batch import validate_tile_batch
from rust_engine_mcp.variant_set import apply_json_patch


def _entry(gate: str) -> EffectBrowseEntry:
    return EffectBrowseEntry(
        effect_id="spark_shower",
        display_name="Spark",
        lane="fx",
        lane_caption="",
        batch_id="b",
        development_tier="lod0",
        spawn_hook="none",
        honest_gate=gate,
        frames_captured=1 if gate == "honest" else 0,
        source="staging",
        spec_path="",
        manifest_path=None,
        preview_witness={},
        emit={},
        lod_tiers={},
        spawn_hook_config={},
        staging_thumb=None,
    )


def _tile_batch(seed: object) -> dict:
    return {
        "schema_version": 1,
        "batch_id": "gap3_batch",
        "tile_id": "gap3_tile",
        "base": "wood",
        "rules_applied": [
            "no_ai_generated_images",
            "deterministic_output",
            "batch_processing",
            "grid_alignment",
        ],
        "bake_source": "smoke_ortho_headless",
        "render": {"method": "blender_orthographic_iso", "seed": seed},
        "variants": [
            {"state": "clean", "power": "off", "fill": "empty", "lighting": "day"},
            {"state": "dirty", "power": "off", "fill": "empty", "lighting": "day"},
        ],
    }


def test_promote_rejects_job_id_path_escape(tmp_path: Path, monkeypatch) -> None:
    import rust_engine_mcp.promote as promote

    monkeypatch.setattr(promote, "repo_root", lambda: tmp_path)
    monkeypatch.setattr(promote, "staging_root", lambda: tmp_path / "staging")
    with pytest.raises(ValueError, match="folder name"):
        promote_module("../escape", force=True, register=False, allow_smoke=True)
    assert not any(tmp_path.rglob("*"))


def test_layer_seed_zero_is_not_replaced(monkeypatch) -> None:
    import rust_engine_mcp.grammar_layers as layers

    captured: dict[str, int] = {}

    def _fake(_archetype: str, _style: str, **kwargs):
        captured["seed"] = kwargs["seed"]
        return {}

    monkeypatch.setattr(layers.building_grammar, "generate_with_overrides", _fake)
    monkeypatch.setattr(
        layers.assembly, "refresh_placements_for_tokens", lambda snap, _grammar, _tokens: snap
    )
    snap = {"archetype_id": "IndustrialWarehouse", "district_style": "style_x", "seed": 0}
    apply_facade_layer(snap, {})
    assert captured["seed"] == 0
    with pytest.raises(ValueError, match="integer"):
        apply_facade_layer(snap, {}, seed=True)


def test_capture_seed_keeps_zero() -> None:
    assert _capture_seed({"rules_check": {"seed": 0}, "seed": 99}) == "0"
    assert _capture_seed({"rules_check": {"seed": "42001"}}) == "42001"
    assert _capture_seed({"seed": 0}) == "0"
    with pytest.raises(ValueError, match="integer"):
        _capture_seed({"rules_check": {"seed": True}})


def test_tile_batch_seed_must_be_int(tmp_path: Path) -> None:
    zero = tmp_path / "zero.json"
    zero.write_text(json.dumps(_tile_batch(0)), encoding="utf-8")
    ok = validate_tile_batch(zero, compression_level=4)
    assert ok.status != "failed"
    bad = tmp_path / "bool.json"
    bad.write_text(json.dumps(_tile_batch(True)), encoding="utf-8")
    report = validate_tile_batch(bad, compression_level=4)
    assert any(issue.signature == "tile_batch_seed_not_int" for issue in report.errors)


def test_render_seed_keeps_zero() -> None:
    assert _render_seed({"seed": 0}) == 0
    assert _render_seed({}) == 42
    with pytest.raises(ValueError, match="integer"):
        _render_seed({"seed": True})
    with pytest.raises(ValueError, match="integer"):
        _render_seed({"seed": "42"})


def test_production_tier_not_displaced_by_name() -> None:
    kept = _index_by_module_id(
        [
            {"module_id": "wall", "job_id": "wall_run001", "development_tier": "production"},
            {
                "module_id": "wall",
                "job_id": "wall_production_debug",
                "development_tier": "lod0",
            },
        ]
    )
    assert kept["wall"]["job_id"] == "wall_run001"
    not_production = _index_by_module_id(
        [
            {"module_id": "wall", "job_id": "wall_lod0_run", "development_tier": "lod0"},
            {"module_id": "wall", "job_id": "reproduction_wall", "development_tier": ""},
        ]
    )
    assert not_production["wall"]["job_id"] == "wall_lod0_run"
    upgraded = _index_by_module_id(
        [
            {"module_id": "wall", "job_id": "wall_lod0_run", "development_tier": "lod0"},
            {"module_id": "wall", "job_id": "wall_production_run001", "development_tier": ""},
        ]
    )
    assert upgraded["wall"]["job_id"] == "wall_production_run001"


def test_material_profile_category_uses_tokens() -> None:
    assert _category_for_profile("steel_panel_01") == "industrial/steel"
    assert _category_for_profile("wood_plank_01") == "residential/wood"
    assert _category_for_profile("roof_tile_01") == "roof"
    assert _category_for_profile("roof_metal_01") == "industrial/steel"
    assert _category_for_profile("plywood_sheet") == "other"
    assert _category_for_profile("fiberglass_panel") == "other"
    assert _category_for_profile("nonmetal_paint") == "other"


def test_module_sidecar_prefers_job_name(tmp_path: Path, monkeypatch) -> None:
    import rust_engine_mcp.library as library

    monkeypatch.setattr(library, "repo_root", lambda: tmp_path)
    job = tmp_path / "assets" / "models" / "modules" / "job_a"
    job.mkdir(parents=True)
    (job / "model.glb").write_bytes(b"glb")
    (job / "zzz_other.module.json").write_text(
        json.dumps({"asset_id": "other", "module": {"grid_units": [1, 1]}}),
        encoding="utf-8",
    )
    (job / "job_a.module.json").write_text(
        json.dumps({"asset_id": "kept", "module": {"grid_units": [1, 1]}}),
        encoding="utf-8",
    )
    row = entry_from_promoted(job)
    assert row is not None
    assert row["module_id"] == "kept"


def test_marker_id_cannot_escape_triggers(tmp_path: Path) -> None:
    with pytest.raises(AssignBlockedError):
        assign_effect_to_marker(
            "spark_shower",
            "../secret",
            repo=tmp_path,
            entry=_entry("dishonest_gate"),
        )
    with pytest.raises(ValueError, match="file name"):
        assign_effect_to_marker(
            "spark_shower",
            "../secret",
            repo=tmp_path,
            force=True,
            entry=_entry("dishonest_gate"),
            cells=[],
        )
    written = assign_effect_to_marker(
        "spark_shower",
        "test_spark_shower",
        repo=tmp_path,
        entry=_entry("honest"),
        cells=[],
    )
    assert (tmp_path / written["path"]).is_file()
    assert not (tmp_path / "secret.trigger.ron").exists()
    assert not (tmp_path / "assets" / "scenarios" / "secret.trigger.ron").exists()


def test_teachable_violation_count_stays_numeric(tmp_path: Path) -> None:
    examples = tmp_path / "examples"
    examples.mkdir()
    extra = MAX_VIOLATIONS + 1
    for index in range(extra):
        (examples / f"gap_{index:02d}.json").write_text("{}", encoding="utf-8")
    config = tmp_path / "guards.json"
    config.write_text(
        json.dumps(
            {
                "min_teaches_axes": 2,
                "valid_teaches_axes": ["grid"],
                "teachable": {
                    "examples_root": examples.as_posix(),
                    "subject_globs": ["*.json"],
                    "exempt_globs": [],
                    "canonical_fixture_globs": [],
                },
            }
        ),
        encoding="utf-8",
    )
    report = example_teachable_audit(config_path=config)
    assert report["violations_truncated"] is True
    assert report["violation_count"] == extra
    assert len(report["violations"]) == MAX_VIOLATIONS


def test_json_patch_remove_rejects_negative_index() -> None:
    doc = {"variants": [{"variant_key": "a"}, {"variant_key": "b"}]}
    with pytest.raises(ValueError, match="outside"):
        apply_json_patch(doc, [{"op": "remove", "path": "/variants/-1"}])
    assert [row["variant_key"] for row in doc["variants"]] == ["a", "b"]
    with pytest.raises(ValueError, match="outside"):
        apply_json_patch(doc, [{"op": "replace", "path": "/variants/-1", "value": {"variant_key": "z"}}])
    assert doc["variants"][1]["variant_key"] == "b"


def test_pixel_conclude_rules_check_follows_verdict(tmp_path: Path) -> None:
    image = tmp_path / "lod.png"
    raw = bytearray()
    for _y in range(16):
        for x in range(16):
            raw.extend((255, 220, 0) if x < 8 else (40, 200, 40))
    Image.frombytes("RGB", (16, 16), bytes(raw)).save(image, format="PNG")
    body = pixel_conclude(image, "world", out_dir=tmp_path / "out")
    assert body["verdict"] == "reject"
    assert body["rules_check"]["passed"] is False
    assert body["rules_check"]["blocked_by"]
    assert body["rules_check"]["seed"] == "17"
