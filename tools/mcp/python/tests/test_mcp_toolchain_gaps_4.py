"""Fourth pass of MCP Python toolchain logic gaps."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from rust_engine_mcp.assembly import footprint_cells_for_snapshot, save_assembly_snapshot
from rust_engine_mcp.blender_runner import read_status, write_status
from rust_engine_mcp.build_set_guards import _glob_match
from rust_engine_mcp.effect_promote import _shader_inside_staging
from rust_engine_mcp.grammar_iterate import iterate_grammar, run_grammar_iterate
from rust_engine_mcp.ops_atomic_witness import append_jsonl_atomic, write_witness_atomic
from rust_engine_mcp.tile_index import _parse_index_ron, entry_from_meta
from rust_engine_mcp.variant_set import _dict_to_ron, apply_json_patch, expand_variant_set_to_tile_batch


def _witness_kwargs() -> dict[str, str]:
    return {
        "actor": "witness_honesty_lib",
        "profile": "gap4",
        "source_system": "test",
    }


def _variant_doc(*, seed: object | None = None, include_seed: bool = True) -> dict:
    doc: dict = {
        "assembly_id": "asm_gap4",
        "variant_set_id": "vs_gap4",
        "variants": [],
    }
    if include_seed:
        doc["seed"] = seed
    return doc


def test_witness_write_cannot_escape_repo(tmp_path: Path, monkeypatch) -> None:
    import rust_engine_mcp.ops_atomic_witness as atomic

    repo = tmp_path / "repo"
    repo.mkdir()
    monkeypatch.setattr(atomic, "repo_root", lambda: repo)
    with pytest.raises(ValueError, match="escapes"):
        write_witness_atomic("../gap4_secret.json", {"n": 1}, **_witness_kwargs())
    assert not (tmp_path / "gap4_secret.json").exists()
    with pytest.raises(ValueError, match="escapes"):
        write_witness_atomic("..\\gap4_secret.json", {"n": 1}, **_witness_kwargs())
    written = write_witness_atomic("debug_runs/gap4_ok.json", {"n": 1}, **_witness_kwargs())
    assert (repo / written["path"]).is_file()
    assert not (tmp_path / "gap4_secret.json").exists()


def test_witness_append_cannot_escape_repo(tmp_path: Path, monkeypatch) -> None:
    import rust_engine_mcp.ops_atomic_witness as atomic

    repo = tmp_path / "repo"
    repo.mkdir()
    monkeypatch.setattr(atomic, "repo_root", lambda: repo)
    with pytest.raises(ValueError, match="escapes"):
        append_jsonl_atomic("../gap4_events.jsonl", {"ok": False}, actor="witness_honesty_lib")
    assert not (tmp_path / "gap4_events.jsonl").exists()
    append_jsonl_atomic("debug_runs/gap4_events.jsonl", {"ok": True}, actor="witness_honesty_lib")
    assert (repo / "debug_runs/gap4_events.jsonl").is_file()


def test_job_status_id_cannot_escape_jobs(tmp_path: Path, monkeypatch) -> None:
    import rust_engine_mcp.blender_runner as runner

    jobs = tmp_path / "jobs"
    jobs.mkdir()
    planted = tmp_path / "gap4_escape.status.json"
    planted.write_text('{"status": "leaked"}', encoding="utf-8")
    monkeypatch.setattr(runner, "jobs_root", lambda: jobs)
    with pytest.raises(ValueError, match="folder name"):
        write_status("../gap4_escape", {"status": "done"})
    assert json.loads(planted.read_text(encoding="utf-8"))["status"] == "leaked"
    with pytest.raises(ValueError, match="folder name"):
        read_status("../gap4_escape")
    write_status("gap4_ok", {"status": "done"})
    assert read_status("gap4_ok")["status"] == "done"


def test_json_patch_rejects_negative_index_inside_parent() -> None:
    doc = {"variants": [{"variant_key": "a"}, {"variant_key": "b"}]}
    with pytest.raises(ValueError, match="outside"):
        apply_json_patch(doc, [{"op": "replace", "path": "/variants/-1/variant_key", "value": "z"}])
    assert [row["variant_key"] for row in doc["variants"]] == ["a", "b"]
    patched = apply_json_patch(
        doc, [{"op": "replace", "path": "/variants/1/variant_key", "value": "c"}]
    )
    assert [row["variant_key"] for row in patched["variants"]] == ["a", "c"]
    assert doc["variants"][1]["variant_key"] == "b"


def test_json_patch_test_rejects_negative_index() -> None:
    doc = {"variants": [{"variant_key": "a"}, {"variant_key": "b"}]}
    with pytest.raises(ValueError, match="outside"):
        apply_json_patch(doc, [{"op": "test", "path": "/variants/-1", "value": {"variant_key": "b"}}])
    assert [row["variant_key"] for row in doc["variants"]] == ["a", "b"]
    patched = apply_json_patch(
        doc, [{"op": "test", "path": "/variants/0", "value": {"variant_key": "a"}}]
    )
    assert patched["variants"][0]["variant_key"] == "a"


def test_json_pointer_tilde_escape_keeps_tilde_then_digit() -> None:
    tilde = apply_json_patch({}, [{"op": "add", "path": "/a~01", "value": 1}])
    assert tilde == {"a~1": 1}
    slash = apply_json_patch({}, [{"op": "add", "path": "/b~1c", "value": 2}])
    assert slash == {"b/c": 2}


def test_variant_batch_keeps_seed_zero_and_rejects_non_integer() -> None:
    zero = expand_variant_set_to_tile_batch(_variant_doc(seed=0))
    assert zero["render"]["seed"] == 0
    missing = expand_variant_set_to_tile_batch(_variant_doc(include_seed=False))
    assert missing["render"]["seed"] == 42
    with pytest.raises(ValueError, match="integer"):
        expand_variant_set_to_tile_batch(_variant_doc(seed=True))
    with pytest.raises(ValueError, match="integer"):
        expand_variant_set_to_tile_batch(_variant_doc(seed="42"))
    with pytest.raises(ValueError, match="integer"):
        expand_variant_set_to_tile_batch(_variant_doc(seed=None))


def test_variant_ron_export_rejects_non_integer_seed() -> None:
    text = _dict_to_ron(
        {
            "schema_version": 1,
            "variant_set_id": "vs",
            "assembly_id": "asm",
            "seed": 0,
        }
    )
    assert "seed: 0," in text
    with pytest.raises(ValueError, match="integer"):
        _dict_to_ron(
            {
                "schema_version": 1,
                "variant_set_id": "vs",
                "assembly_id": "asm",
                "seed": True,
            }
        )
    with pytest.raises(ValueError, match="integer"):
        _dict_to_ron(
            {
                "schema_version": 1,
                "variant_set_id": "vs",
                "assembly_id": "asm",
                "seed": "42",
            }
        )


def test_footprint_seed_zero_is_kept(monkeypatch) -> None:
    import rust_engine_mcp.building_grammar as grammar

    seen: list[int] = []

    def fake_generate(_archetype: str, _style: str, seed: int):
        seen.append(seed)
        return {"width": 1, "depth": 1}

    monkeypatch.setattr(grammar, "generate", fake_generate)
    monkeypatch.setattr(grammar, "footprint_grid_from_grammar", lambda _grammar: [])
    snap = {
        "archetype_id": "IndustrialWarehouse",
        "district_style": "style_x",
        "footprint": {"width": 2, "depth": 2, "floors": 1},
        "seed": 0,
    }
    assert footprint_cells_for_snapshot(snap) == []
    assert seen == [0]
    snap["seed"] = True
    with pytest.raises(ValueError, match="integer"):
        footprint_cells_for_snapshot(snap)
    assert seen == [0]
    missing = dict(snap)
    missing.pop("seed")
    footprint_cells_for_snapshot(missing)
    assert seen == [0, 42]


def test_snapshot_path_cannot_escape_repo(tmp_path: Path, monkeypatch) -> None:
    import rust_engine_mcp.assembly as assembly

    repo = tmp_path / "repo"
    repo.mkdir()
    monkeypatch.setattr(assembly, "repo_root", lambda: repo)
    monkeypatch.setattr(assembly, "enrich_snapshot", lambda snap: snap)
    monkeypatch.setattr(assembly, "validate_assembly_snapshot", lambda _snap: None)
    with pytest.raises(ValueError, match="escapes"):
        save_assembly_snapshot({"assembly_id": "kept"}, path="../gap4_escape.json")
    assert not (tmp_path / "gap4_escape.json").exists()
    written = save_assembly_snapshot(
        {"assembly_id": "kept"},
        path="assets/staging/assemblies/kept.json",
    )
    assert written == (repo / "assets/staging/assemblies/kept.json")
    assert written.is_file()


def test_grammar_iterate_request_cannot_escape_repo(tmp_path: Path, monkeypatch) -> None:
    import rust_engine_mcp.grammar_iterate as iterate

    repo = tmp_path / "repo"
    repo.mkdir()
    monkeypatch.setattr(iterate, "repo_root", lambda: repo)
    with pytest.raises(ValueError, match="escapes"):
        run_grammar_iterate("../gap4_request.json")
    assert not (tmp_path / "gap4_request.json").exists()


def test_grammar_iterate_witness_cannot_escape_repo(tmp_path: Path, monkeypatch) -> None:
    import rust_engine_mcp.grammar_iterate as iterate

    repo = tmp_path / "repo"
    repo.mkdir()
    monkeypatch.setattr(iterate, "repo_root", lambda: repo)
    req = tmp_path / "req.json"
    req.write_text(json.dumps({"mode": "not-a-mode", "seed": 0}), encoding="utf-8")
    with pytest.raises(ValueError, match="escapes"):
        run_grammar_iterate(req, write_witness="../gap4_witness.json")
    assert not (tmp_path / "gap4_witness.json").exists()
    out = run_grammar_iterate(req, write_witness="debug_runs/gap4_ok.json")
    assert out["ok"] is False
    assert out["seed"] == 0
    assert (repo / "debug_runs/gap4_ok.json").is_file()


def test_grammar_iterate_seed_must_be_int() -> None:
    kept = iterate_grammar({"mode": "not-a-mode", "seed": 0})
    assert kept["seed"] == 0
    assert kept["ok"] is False
    with pytest.raises(ValueError, match="integer"):
        iterate_grammar({"mode": "not-a-mode", "seed": True})
    with pytest.raises(ValueError, match="integer"):
        iterate_grammar({"mode": "not-a-mode", "seed": "42"})


def test_atlas_comment_does_not_set_ship_allowed() -> None:
    text = """
(
    entries: [
        (
            atlas_id: "a",
            batch_id: "b",
            assembly_id: "",
            tile_id: "t",
            atlas_png: "p.png",
            meta_json: "m.json",
            development_tier: "lod0",
            style_pack_id: "pack",
            // preview ship_allowed: true
            ship_allowed: false,
        ),
        (
            atlas_id: "c",
            batch_id: "b",
            assembly_id: "",
            tile_id: "t2",
            atlas_png: "p2.png",
            meta_json: "m2.json",
            development_tier: "production",
            style_pack_id: "pack",
        ),
    ],
)
"""
    rows = _parse_index_ron(text)
    assert [row["atlas_id"] for row in rows] == ["a", "c"]
    assert rows[0]["ship_allowed"] is False
    assert rows[1]["ship_allowed"] is True


def test_atlas_png_path_cannot_leave_repo() -> None:
    with pytest.raises(ValueError, match="escapes"):
        entry_from_meta({"atlas_id": "a", "atlas_png": "../outside_gap4.png"})
    empty = entry_from_meta({"atlas_id": "a", "atlas_png": ""})
    assert empty["atlas_png"] == ""
    kept = entry_from_meta(
        {"atlas_id": "a", "batch_id": "b", "tile_id": "t", "atlas_png": "assets/textures/tiles/a.png"}
    )
    assert kept["atlas_png"] == "assets/textures/tiles/a.png"


def test_teachable_glob_star_does_not_cross_slash() -> None:
    pattern = ["tools/mcp/schemas/examples/*.json"]
    assert _glob_match("tools/mcp/schemas/examples/wall.json", pattern)
    assert not _glob_match("tools/mcp/schemas/examples/nested/wall.json", pattern)
    assert not _glob_match("tools/mcp/schemas/examples_extra/wall.json", pattern)
    assert _glob_match("tools/mcp/schemas/examples/nested/wall.json", ["*.json"])


def test_shader_absolute_dest_cannot_escape_staging(tmp_path: Path) -> None:
    staging = tmp_path / "fx"
    shaders = staging / "shaders"
    shaders.mkdir(parents=True)
    (shaders / "draw.wgsl").write_bytes(b"fn main() {}\n")
    inside = _shader_inside_staging(staging, "shaders/draw.wgsl")
    assert inside == (shaders / "draw.wgsl").resolve()
    with pytest.raises(ValueError, match="escapes"):
        _shader_inside_staging(staging, "C:/secret.wgsl")
    assert not Path("C:/secret.wgsl").exists()
