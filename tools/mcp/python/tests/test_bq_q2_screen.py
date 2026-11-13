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


