"""Concrete MCP Python toolchain gaps — corrupt GLB, globs, tiers, gates."""

from __future__ import annotations

import json
import struct
from pathlib import Path

from rust_engine_mcp.dmcp_facility_binding_lane import power_tier_for_units
from rust_engine_mcp.effect_registry_browse import list_effect_entries
from rust_engine_mcp.glb_bounds import read_glb_json
from rust_engine_mcp.material_category_tree import category_label, infer_category_from_tree
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.power_tier_bands import power_tier_band_warning, power_tier_from_units
from rust_engine_mcp.symbolic_build_encoder import _glob_match, trip_hits_for_files
from rust_engine_mcp.tile_index import _parse_index_ron
from rust_engine_mcp.validate_glb import validate_glb


def _mesh_doc() -> dict:
    return {
        "asset": {"version": "2.0"},
        "meshes": [{"primitives": [{"attributes": {"POSITION": 0}}]}],
        "accessors": [
            {
                "count": 3,
                "type": "VEC3",
                "componentType": 5126,
                "min": [0, 0, 0],
                "max": [1, 1, 1],
            }
        ],
    }


def _pack_glb(payload: bytes, *, chunk_len: int | None = None, total_length: int | None = None) -> bytes:
    pad = (4 - (len(payload) % 4)) % 4
    body = payload + b" " * pad
    declared = len(body) if chunk_len is None else chunk_len
    chunk = struct.pack("<I4s", declared, b"JSON") + body
    total = 12 + len(chunk)
    header = struct.pack("<4sII", b"glTF", 2, total if total_length is None else total_length)
    return header + chunk


def test_validate_glb_reports_corrupt_chunks(tmp_path: Path) -> None:
    good = tmp_path / "good.glb"
    good.write_bytes(_pack_glb(json.dumps(_mesh_doc()).encode()))
    report = validate_glb(good)
    assert report.valid is True
    assert report.vertex_count == 3

    truncated = tmp_path / "truncated.glb"
    truncated.write_bytes(_pack_glb(json.dumps(_mesh_doc()).encode(), chunk_len=10_000))
    bad = validate_glb(truncated)
    assert bad.valid is False
    assert any("truncated" in issue for issue in bad.issues)

    broken = tmp_path / "broken.glb"
    broken.write_bytes(_pack_glb(b"{{{{"))
    invalid = validate_glb(broken)
    assert invalid.valid is False
    assert any("not valid JSON" in issue for issue in invalid.issues)
    assert read_glb_json(broken) is None
    assert read_glb_json(truncated) is None

    mismatched = tmp_path / "length.glb"
    raw = bytearray(_pack_glb(json.dumps(_mesh_doc()).encode()))
    struct.pack_into("<I", raw, 8, len(raw) + 4)
    mismatched.write_bytes(raw)
    length_report = validate_glb(mismatched)
    assert length_report.valid is False
    assert any("length header" in issue for issue in length_report.issues)
    assert length_report.vertex_count == 3


def test_trip_glob_does_not_match_sibling_prefix() -> None:
    assert _glob_match("src/terrain/generation/mod.rs", "src/terrain/generation/**")
    assert not _glob_match("src/terrain/generation_extra/mod.rs", "src/terrain/generation/**")
    assert _glob_match(
        "tools/mcp/schemas/examples/effect_spec_fire.json",
        "tools/mcp/schemas/examples/effect_spec_*.json",
    )
    assert not _glob_match(
        "tools/mcp/schemas/examples/other.json",
        "tools/mcp/schemas/examples/effect_spec_*.json",
    )
    assert _glob_match("foo.rs", "**/foo.rs")
    assert _glob_match("a/b/foo.rs", "**/foo.rs")

    root = repo_root()
    hits = trip_hits_for_files(root, ["src/terrain/generation/mod.rs"])
    assert "TRIP-VSS-SCALE" in hits
    misses = trip_hits_for_files(root, ["src/terrain/generation_extra/mod.rs"])
    assert "TRIP-VSS-SCALE" not in misses


def test_power_above_heavy_band_is_not_grid() -> None:
    assert power_tier_for_units(0) == "light"
    assert power_tier_for_units(30) == "light"
    assert power_tier_for_units(31) == "medium"
    assert power_tier_for_units(80) == "medium"
    assert power_tier_for_units(81) == "heavy"
    assert power_tier_for_units(200) == "heavy"
    assert power_tier_for_units(201) == "heavy"
    assert power_tier_from_units(8, utility_role="substation") == "grid"
    assert power_tier_from_units(4, utility_role="power_plant", power_generation=10) == "grid"
    assert power_tier_band_warning(201)
    assert power_tier_band_warning(18) is None
    assert power_tier_band_warning(500, utility_role="power_plant") is None


def test_facade_layer_refreshes_window_openings(monkeypatch) -> None:
    import rust_engine_mcp.grammar_layers as layers

    seen: dict = {}

    def fake_generate(archetype_id: str, district_style: str, **kwargs):
        seen["window_slot"] = kwargs.get("window_slot")
        seen["archetype_id"] = archetype_id
        return {"style_pack_id": "style_industrial_west", "slot_overrides": {}}

    def fake_refresh(snapshot, grammar, tokens):
        seen["tokens"] = set(tokens)
        return {**snapshot, "refreshed": True}

    monkeypatch.setattr(layers.building_grammar, "generate_with_overrides", fake_generate)
    monkeypatch.setattr(layers.assembly, "refresh_placements_for_tokens", fake_refresh)
    out = layers.apply_facade_layer(
        {
            "archetype_id": "IndustrialWarehouse",
            "district_style": "industrial_west",
            "seed": 7,
        },
        {"window_slot": "win_brick_1u"},
    )
    assert out["refreshed"] is True
    assert seen["window_slot"] == "win_brick_1u"
    assert seen["tokens"] == {"W", "D", "C", "O"}


def _effect_dir(root: Path, effect_id: str, gate: str) -> None:
    folder = root / "assets" / "effects" / "registry" / effect_id
    folder.mkdir(parents=True)
    (folder / "effect_spec.json").write_text(
        json.dumps(
            {
                "effect_id": effect_id,
                "lane": "overlay",
                "batch_id": "gap_batch",
                "display_name": effect_id,
                "preview_witness": {"honest_gate": gate, "frames_captured": 0},
            }
        ),
        encoding="utf-8",
    )


def test_dishonest_gate_is_not_upgraded_to_honest(tmp_path: Path) -> None:
    _effect_dir(tmp_path, "fx_blocked", "dishonest_gate")
    frames = tmp_path / "assets/effects/registry/fx_blocked/preview_frames"
    frames.mkdir()
    (frames / "frames_manifest.json").write_text(
        json.dumps({"preview_witness": {"honest_gate": "honest", "frames_captured": 4}}),
        encoding="utf-8",
    )
    staging = tmp_path / "assets/staging/fx_blocked"
    staging.mkdir(parents=True)
    (staging / "effect_spec.json").write_text(
        json.dumps(
            {
                "effect_id": "fx_blocked",
                "lane": "overlay",
                "batch_id": "gap_batch",
                "preview_witness": {"honest_gate": "honest", "frames_captured": 2},
            }
        ),
        encoding="utf-8",
    )
    rows = list_effect_entries(repo=tmp_path, include_staging=True)
    assert len(rows) == 1
    assert rows[0].honest_gate == "dishonest_gate"
    assert rows[0].assign_allowed is False
    assert rows[0].source == "registry"

    _effect_dir(tmp_path, "fx_pending", "pending")
    pending_frames = tmp_path / "assets/effects/registry/fx_pending/preview_frames"
    pending_frames.mkdir()
    (pending_frames / "frames_manifest.json").write_text(
        json.dumps({"preview_witness": {"honest_gate": "honest", "frames_captured": 1}}),
        encoding="utf-8",
    )
    pending = list_effect_entries(repo=tmp_path, include_staging=False)
    by_id = {row.effect_id: row for row in pending}
    assert by_id["fx_pending"].honest_gate == "honest"
    assert by_id["fx_pending"].assign_allowed is True
    assert by_id["fx_blocked"].honest_gate == "dishonest_gate"


def test_witness_hash_is_read_inside_lock(tmp_path: Path, monkeypatch) -> None:
    import rust_engine_mcp.ops_atomic_witness as atomic

    order: list[str] = []
    real_acquire = atomic.acquire_write_lock
    real_read = atomic.read_previous_hash

    def acquire(target, *, actor, timeout_sec=5.0):
        order.append("lock")
        return real_acquire(target, actor=actor, timeout_sec=timeout_sec)

    def read(target):
        order.append("read")
        return real_read(target)

    monkeypatch.setattr(atomic, "repo_root", lambda: tmp_path)
    monkeypatch.setattr(atomic, "acquire_write_lock", acquire)
    monkeypatch.setattr(atomic, "read_previous_hash", read)

    kwargs = {
        "actor": "witness_honesty_lib",
        "profile": "gap",
        "source_system": "test",
    }
    atomic.write_witness_atomic("debug_runs/gap_witness.json", {"n": 1}, **kwargs)
    first = json.loads((tmp_path / "debug_runs/gap_witness.json").read_text(encoding="utf-8"))
    assert atomic.witness_content_matches(first)
    atomic.write_witness_atomic("debug_runs/gap_witness.json", {"n": 2}, **kwargs)
    second = json.loads((tmp_path / "debug_runs/gap_witness.json").read_text(encoding="utf-8"))
    assert second["previous_hash"] == first["content_hash"]
    assert atomic.witness_content_matches(second)
    assert order[:2] == ["lock", "read"]
    assert "read" not in order or order.index("lock") < order.index("read")


def test_atlas_ron_parser_does_not_steal_next_ship_flag() -> None:
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
        ),
        (
            atlas_id: "c",
            batch_id: "b",
            assembly_id: "",
            tile_id: "t2",
            atlas_png: "p2.png",
            meta_json: "m2.json",
            development_tier: "lod0",
            style_pack_id: "pack",
            ship_allowed: true,
        ),
    ],
)
"""
    rows = _parse_index_ron(text)
    assert [row["atlas_id"] for row in rows] == ["a", "c"]
    assert rows[0]["ship_allowed"] is False
    assert rows[1]["ship_allowed"] is True


def test_material_category_tokens_and_nested_labels() -> None:
    assert infer_category_from_tree("wood_clapboard") == "residential/wood"
    assert infer_category_from_tree("stainless_steel_panel") == "industrial/steel"
    assert infer_category_from_tree("plywood_sheet") is None
    assert infer_category_from_tree("steel_panel_01") == "industrial/steel"

    tree = {
        "roots": [
            {
                "id": "industrial",
                "label": "Industrial",
                "children": [
                    {"id": "steel", "label": "Hot Steel"},
                    {
                        "id": "nope",
                        "label": "Nope",
                        "children": [{"id": "panel", "label": "Panel Face"}],
                    },
                ],
            }
        ]
    }
    assert category_label("industrial/steel", tree=tree) == "Hot Steel"
    assert category_label("industrial/nope/panel", tree=tree) == "Panel Face"
    assert category_label("industrial/nope/steel", tree=tree) == "Steel"
    assert category_label("industrial/steel") == "Steel"
