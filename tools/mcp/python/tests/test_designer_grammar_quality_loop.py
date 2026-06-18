"""Designer grammar quality loop — compressed tier + guard rollup."""

from __future__ import annotations

from rust_engine_mcp.designer_grammar_quality_loop import run_designer_grammar_quality_loop


def test_designer_grammar_quality_loop_fast() -> None:
    body = run_designer_grammar_quality_loop(mode="fast")
    assert body["task_id"] == "DESIGNER-GRAMMAR-QUALITY-LOOP-001"
    assert body["tier"] in ("G1", "G2", "G3", "G4")
    assert body["tier_detail"]["archetype_count"] >= 3
    assert body["next_actions"]
    assert len(body["grammar_checks"]) >= 3
    assert all(r.get("schema_ok") for r in body["grammar_checks"])
    assert all(r.get("generate_ok") for r in body["grammar_checks"])


def test_designer_grammar_quality_loop_writes_witness(tmp_path, monkeypatch) -> None:
    from rust_engine_mcp import designer_grammar_quality_loop as mod

    monkeypatch.setattr(mod, "repo_root", lambda: tmp_path)
    body = run_designer_grammar_quality_loop(mode="fast", write_witness=True)
    assert "witness_path" in body
    assert (tmp_path / mod.DESIGNER_GRAMMAR_LOOP_WITNESS).is_file()
