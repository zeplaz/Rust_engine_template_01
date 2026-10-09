"""Fifth pass — fail-closed holes in industrial power, binding, spine, and promote."""

from __future__ import annotations

import json
import os
from pathlib import Path

import pytest

from rust_engine_mcp.dmcp_facility_binding_lane import (
    _check_concrete_pilot,
    _check_site_grid,
    power_tier_for_units,
    run_facility_binding_lane_audit,
)
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.power_tier_bands import power_tier_band_warning, power_tier_from_units
from rust_engine_mcp.promote import promote_module
from rust_engine_mcp.spine_build_graph import EXAMPLE_REL, graph_errors, probe_node, run_spine_build_audit


def _posix_rel(target: Path, start: Path) -> str:
    return Path(os.path.relpath(target, start)).as_posix()


def _site_grid() -> dict:
    return {
        "schema_version": "site_zone_grid_v1",
        "site_id": "gap_site",
        "width": 1,
        "depth": 1,
        "cells": [{"zone": "pad"}],
        "chain_id": "concrete_portland",
    }


def test_power_units_reject_bool_and_non_finite() -> None:
    assert power_tier_for_units(0) == "light"
    assert power_tier_from_units(0) == "light"
    assert power_tier_from_units(0, utility_role="power_plant", power_generation=1) == "grid"
    assert power_tier_band_warning(0) is None
    with pytest.raises(ValueError, match="finite"):
        power_tier_for_units(True)
    with pytest.raises(ValueError, match="finite"):
        power_tier_for_units(False)
    with pytest.raises(ValueError, match="finite"):
        power_tier_from_units(float("nan"))
    with pytest.raises(ValueError, match="finite"):
        power_tier_from_units(float("inf"))
    with pytest.raises(ValueError, match="finite"):
        power_tier_band_warning(float("nan"))
    with pytest.raises(ValueError, match="finite"):
        power_tier_from_units(10, utility_role="power_plant", power_generation=float("nan"))


def test_facility_slice_verdict_follows_status(monkeypatch) -> None:
    import rust_engine_mcp.dmcp_facility_binding_lane as lane

    monkeypatch.setattr(lane, "_check_binding_schema", lambda root: {"green": False, "checks": {}})
    body = run_facility_binding_lane_audit()
    row = next(item for item in body["rows"] if item["id"] == "DES-FACILITY-BINDING-001")
    assert row["status"] == "fail"
    assert row["verdict"] == "FAIL"
    assert body["green"] is False
    assert body["verdict"] == "FAIL"


def test_facility_site_plan_cannot_escape_repo(tmp_path: Path) -> None:
    root = tmp_path / "repo"
    inside = root / "assets" / "configs" / "buildings" / "pilots" / "mine_site.json"
    inside.parent.mkdir(parents=True)
    inside.write_text(json.dumps(_site_grid()), encoding="utf-8")
    kept = _check_site_grid(root, "assets/configs/buildings/pilots/mine_site.json")
    assert kept["green"] is True
    assert kept["checks"]["path_inside_repo"] is True

    outside = tmp_path / "outside_site.json"
    outside.write_text(json.dumps(_site_grid()), encoding="utf-8")
    escaped = _posix_rel(outside, root)
    assert ".." in escaped.split("/")
    stolen = _check_site_grid(root, escaped)
    assert stolen["green"] is False
    assert stolen["checks"]["path_inside_repo"] is False


def test_facility_catalog_cannot_escape_repo(tmp_path: Path) -> None:
    root = tmp_path / "repo"
    buildings = root / "assets" / "configs" / "buildings"
    escaped_file = tmp_path / "escaped_catalog.json"
    escaped_file.write_text(
        json.dumps({"power_consumption": 12, "supply_chain_role": "aggregate_mine"}),
        encoding="utf-8",
    )
    evil_id = _posix_rel(escaped_file, buildings)
    assert evil_id.endswith(".json")
    evil_id = evil_id[: -len(".json")]
    assert ".." in evil_id.split("/")
    naive = (buildings / f"{evil_id}.json").resolve()
    assert not naive.is_relative_to(root.resolve())

    site_rel = "assets/configs/buildings/pilots/mine_site.json"
    site = root / site_rel
    site.parent.mkdir(parents=True)
    site.write_text(json.dumps(_site_grid()), encoding="utf-8")
    chain = root / "assets" / "configs" / "industrial_supply_chains.json"
    chain.write_text(
        json.dumps(
            {
                "chains": {
                    "concrete_portland": {
                        "steps": [
                            {"role": "aggregate_mine", "catalog_id": evil_id},
                            {"role": "cement_kiln", "catalog_id": "cement_kiln"},
                            {"role": "concrete_mixer", "catalog_id": "concrete_mixer"},
                        ]
                    }
                }
            }
        ),
        encoding="utf-8",
    )
    pilot = {
        "schema_version": "facility_chain_pilot_v1",
        "chain_id": "concrete_portland",
        "grammar_archetype_hint": "FactoryCluster",
        "steps": [
            {
                "role": "aggregate_mine",
                "catalog_id": evil_id,
                "power_consumption_ref": 12,
                "power_tier": "light",
                "site_plan_json": site_rel,
            }
        ],
    }
    pilot_path = root / "assets/configs/buildings/pilots/concrete_portland_chain_pilot_v1.json"
    pilot_path.write_text(json.dumps(pilot), encoding="utf-8")

    audit = _check_concrete_pilot(root)
    mine = next(step for step in audit["steps"] if step["role"] == "aggregate_mine")
    assert mine["green"] is False
    assert mine["checks"]["catalog_on_disk"] is False


def test_build_graph_artifact_cannot_escape_repo(tmp_path: Path) -> None:
    outside = tmp_path / "secret.json"
    outside.write_text("{}", encoding="utf-8")
    root = tmp_path / "repo"
    root.mkdir()
    relative = probe_node(
        {"node_id": "build_assembly", "artifact": "../secret.json"},
        root=root,
    )
    absolute = probe_node(
        {"node_id": "render_frames", "artifact": str(outside)},
        root=root,
    )
    assert relative["artifact_exists"] is False
    assert relative["ok"] is False
    assert relative["status"] == "escapes_repo"
    assert absolute["artifact_exists"] is False
    assert absolute["ok"] is False

    data = json.loads((repo_root() / EXAMPLE_REL).read_text(encoding="utf-8"))
    for node in data["nodes"]:
        if node["node_id"] == "build_assembly":
            node["artifact"] = "../secret.json"
            break
    errs = graph_errors(data)
    assert any("escapes repo" in err for err in errs)


def test_build_graph_rejects_self_dependency() -> None:
    data = json.loads((repo_root() / EXAMPLE_REL).read_text(encoding="utf-8"))
    for node in data["nodes"]:
        if node["node_id"] == "pack_atlas":
            node["depends_on"] = ["pack_atlas"]
            break
    errs = graph_errors(data)
    assert any(err.startswith("cycle detected:") and "pack_atlas" in err for err in errs)


def test_spine_rules_check_follows_green(monkeypatch) -> None:
    import rust_engine_mcp.spine_build_graph as spine

    live = run_spine_build_audit()
    assert live["green"] is True
    seed = live["rules_check"]["seed"]
    assert isinstance(seed, int) and not isinstance(seed, bool)
    assert live["rules_check"]["passed"] is True
    assert live["rules_check"]["deterministic_output"] is True
    assert live["rules_check"]["blocked_by"] == []

    def reject(_data: dict) -> None:
        raise ValueError("schema rejected")

    monkeypatch.setattr(spine, "validate_build_graph", reject)
    failed = spine.run_spine_build_audit()
    assert failed["green"] is False
    assert failed["rules_check"]["passed"] is False
    assert failed["rules_check"]["blocked_by"] == ["spine_build_audit"]
    assert isinstance(failed["rules_check"]["seed"], int)
    assert not isinstance(failed["rules_check"]["seed"], bool)


def test_effect_spec_cannot_escape_repo(tmp_path: Path, monkeypatch) -> None:
    import rust_engine_mcp.effect_promote as ep

    root = tmp_path / "repo"
    spec = root / "effects" / "gap_spec.json"
    spec.parent.mkdir(parents=True)
    spec.write_text("{}", encoding="utf-8")
    outside = tmp_path / "effect.json"
    outside.write_text("{}", encoding="utf-8")
    monkeypatch.setattr(ep, "repo_root", lambda: root)
    kept = ep._resolve_spec("effects/gap_spec.json")
    assert kept == spec.resolve()
    with pytest.raises(ValueError, match="escapes"):
        ep._resolve_spec("../effect.json")
    with pytest.raises(ValueError, match="escapes"):
        ep._resolve_spec(outside)


def test_effect_pack_shader_cannot_escape_repo(tmp_path: Path, monkeypatch) -> None:
    import rust_engine_mcp.effect_promote as ep

    root = tmp_path / "repo"
    (root / "assets" / "staging").mkdir(parents=True)
    spec = root / "specs" / "gap.json"
    spec.parent.mkdir(parents=True)
    spec.write_text("{}", encoding="utf-8")
    outside = tmp_path / "stolen.wgsl"
    outside.write_text("fn main() {}", encoding="utf-8")
    monkeypatch.setattr(ep, "repo_root", lambda: root)
    monkeypatch.setattr(ep, "staging_root", lambda: root / "assets" / "staging")
    monkeypatch.setattr(ep, "_require_rules_check", lambda *args, **kwargs: None)

    def pack_of(shader: str) -> None:
        monkeypatch.setattr(
            ep,
            "_require_validate",
            lambda path: {
                "effect_id": "gap_fx",
                "batch_id": "gap_batch",
                "visual": {"source": "wgsl_pack", "shader_pack": {"compute": shader}},
            },
        )
        ep.pack_effect_spec(spec)

    with pytest.raises(ValueError, match="escapes"):
        pack_of("../stolen.wgsl")
    with pytest.raises(ValueError, match="escapes"):
        pack_of(str(outside))
    staged = root / "assets" / "staging" / "gap_fx" / "shaders" / "stolen.wgsl"
    assert not staged.exists()


def test_effect_witness_keeps_dishonest_gate(tmp_path: Path, monkeypatch) -> None:
    import rust_engine_mcp.effect_promote as ep

    root = tmp_path / "repo"
    rel = "tools/mcp/schemas/examples/effect_spec_gap.json"
    spec = root / rel
    spec.parent.mkdir(parents=True)
    spec.write_text(
        json.dumps({"effect_id": "gap_fx", "lane": "particle_instanced", "batch_id": "gap_batch"}),
        encoding="utf-8",
    )
    frames = root / "assets" / "staging" / "gap_fx" / "preview_frames"
    frames.mkdir(parents=True)
    (frames / "frames_manifest.json").write_text(
        json.dumps({"preview_witness": {"honest_gate": "dishonest_gate", "frames_captured": 0}}),
        encoding="utf-8",
    )
    registry = root / "assets" / "effects" / "registry" / "gap_fx"
    registry.mkdir(parents=True)
    (registry / "manifest.json").write_text(
        json.dumps(
            {
                "effect_id": "gap_fx",
                "preview_witness": {"honest_gate": "honest", "frames_captured": 4},
            }
        ),
        encoding="utf-8",
    )

    class _Report:
        status = "passed"

    monkeypatch.setattr(ep, "REFERENCE_SPEC_RELS", (rel, rel, rel))
    monkeypatch.setattr(ep, "validate_effect_spec", lambda *args, **kwargs: _Report())
    body = ep.refresh_artist_vfx_pipeline_witness(repo=root)
    promoted = body["reference_specs"]["promoted"]["gap_fx"]["preview_witness"]
    assert promoted["honest_gate"] == "dishonest_gate"
    assert body["reference_specs"]["preview_capture"]["honest_count"] == 0


def test_promote_spec_ref_cannot_escape_repo(tmp_path: Path, monkeypatch) -> None:
    import rust_engine_mcp.promote as promote

    root = tmp_path / "repo"
    staging = root / "assets" / "staging" / "gap_job"
    staging.mkdir(parents=True)
    (staging / "model.glb").write_bytes(b"glTF")
    outside = tmp_path / "secret.module.json"
    outside.write_text(
        json.dumps(
            {
                "development_tier": "production",
                "pbr_status": "shipped",
                "material_profile": "gap_mat",
            }
        ),
        encoding="utf-8",
    )
    monkeypatch.setattr(promote, "repo_root", lambda: root)
    monkeypatch.setattr(promote, "staging_root", lambda: root / "assets" / "staging")
    monkeypatch.setattr(promote, "read_status", lambda job_id: {"spec_ref": "../secret.module.json"})
    with pytest.raises(ValueError, match="escapes"):
        promote_module("gap_job", force=True, register=False, allow_smoke=True)
    assert not (root / "assets" / "models").exists()

    monkeypatch.setattr(promote, "read_status", lambda job_id: {"spec_ref": str(outside)})
    with pytest.raises(ValueError, match="escapes"):
        promote_module("gap_job", force=True, register=False, allow_smoke=True)
    assert not (root / "assets" / "models").exists()
