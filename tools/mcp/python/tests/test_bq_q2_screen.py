"""BQ-Q2-SCREEN-001 — quality screen pass tests (honest SILH + PNG paths)."""

from __future__ import annotations

from rust_engine_mcp import bq_q2_screen
from rust_engine_mcp.paths import repo_root


def test_screen_sweep_covers_three_packs() -> None:
    packs = {r["style_pack_id"] for r in bq_q2_screen.SCREEN_SWEEP}
    assert len(packs) >= 3
    by_pack: dict[str, int] = {}
    for row in bq_q2_screen.SCREEN_SWEEP:
        by_pack[row["style_pack_id"]] = by_pack.get(row["style_pack_id"], 0) + 1
    assert sum(1 for n in by_pack.values() if n >= 2) >= 3


def test_format_q2_strip_without_witness() -> None:
    # Uses live witness if present; only asserts return shape.
    text, ok = bq_q2_screen.format_q2_strip_text()
    assert isinstance(text, str)
    assert text.startswith("Q2")
    assert ok is None or isinstance(ok, bool)


def test_schematic_png_usable() -> None:
    from rust_engine_mcp import assembly

    snap = assembly.generate_assembly_snapshot(
        style_pack_id="style_victorian",
        width=4,
        depth=2,
        floors=2,
        seed=1,
        source_tier="lod0",
        write=False,
    )
    out = repo_root() / "debug_runs" / "bq_q2_screens" / "_schematic_unit.png"
    assert bq_q2_screen.render_assembly_schematic_png(snap, out)
    assert out.is_file()
    assert out.stat().st_size > 256


def test_write_bq_q2_screen_witness_no_bevy() -> None:
    body = bq_q2_screen.write_bq_q2_screen_witness(try_bevy=False)
    assert body["written"] == bq_q2_screen.WITNESS_REL
    path = repo_root() / bq_q2_screen.WITNESS_REL
    assert path.is_file()
    assert body["gate"] == "BQ-Q2-SCREEN-001"
    assert body["silh_prerequisite"]["ok"] is True
    assert body["coverage"]["pack_count_meeting_min"] >= 3
    assert body["no_pass_without_screenshot"] is True
    assert body["screens_with_png"] == body["screen_count"]
    assert body["screen_count"] >= 6
    modes = {s.get("preview_mode") for s in body["screens"]}
    assert "schematic_pil" in modes or "trimesh_thumbnail" in modes or "bevy_worker" in modes
    for screen in body["screens"]:
        assert screen["preview_png"], f"missing PNG for {screen['assembly_id']}"
        png = repo_root() / screen["preview_png"]
        assert png.is_file()
    for row in (
        __import__("json").loads(
            (repo_root() / bq_q2_screen.RUBRIC_ROWS_REL).read_text(encoding="utf-8")
        )["rows"]
    ):
        assert row["criterion"] == bq_q2_screen.RUBRIC_CRITERION
        assert row["screen_pass"] is False
        assert row["verdict"] in ("pending_operator", "blocked_no_screenshot")
        if row["verdict"] == "pending_operator":
            assert row["preview_png"]
    assert body["next_residual"] == "BQ-Q3-OPS-APPROVE-001"
    assert body.get("green") is True
    assert body.get("_agent_meta", {}).get("task_id") == "BQ-Q2-SCREEN-001"
