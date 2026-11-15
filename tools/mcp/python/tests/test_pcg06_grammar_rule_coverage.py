"""PCG-06 — grammar rule coverage report (golden seeds + tiny dead rule)."""

from __future__ import annotations

from typing import Any

import pytest

from rust_engine_mcp import building_grammar, kit_coverage_audit
from rust_engine_mcp.paths import repo_root


def _tiny_grammar_with_dead_rule() -> dict[str, Any]:
    return {
        "schema_version": 1,
        "grammar_id": "tiny_dead_rule_v1",
        "archetype": {
            "id": "TinyArchetype",
            "usage": "test",
            "footprint_bounds": {
                "min_width": 4,
                "max_width": 6,
                "min_depth": 4,
                "max_depth": 6,
                "min_floors": 1,
                "max_floors": 1,
            },
        },
        "massing": {
            "strategies": [
                {
                    "id": "alive_rect",
                    "weight": 100,
                    "width_depth_ratio": 1.0,
                    "footprint_mode": "rect",
                },
                {
                    "id": "dead_never_massing",
                    "weight": 0,
                    "width_depth_ratio": 1.0,
                    "footprint_mode": "rect",
                },
            ]
        },
        "roof": {"default_slot": "roof_default"},
        "facade": {
            "wall_slot": "wall_1u",
            "door_slot": "door_default",
            "window_slot": "window_1u",
            "placement_tags": ["exterior"],
        },
        "detail": {"prop_slot": "none", "density": 0.0},
        "age": {
            "bands": [
                {"id": "new", "weight": 100, "variant_tags": ["clean"]},
                {"id": "abandoned_never", "weight": 0, "variant_tags": ["abandoned"]},
            ]
        },
        "district_styles": [
            {
                "id": "test_style",
                "style_pack_id": "style_industrial_west",
                "style_tags": ["test"],
                "zoning": "industrial",
            }
        ],
    }


def test_pcg06_tiny_grammar_dead_rules_never_fire(monkeypatch: pytest.MonkeyPatch) -> None:
    tiny = _tiny_grammar_with_dead_rule()
    monkeypatch.setattr(
        building_grammar,
        "load_building_grammar_by_archetype",
        lambda _aid: tiny,
    )
    fired: set[str] = set()
    for seed in range(24):
        result = building_grammar.generate("TinyArchetype", "test_style", seed)
        fired.update(building_grammar.fired_rule_keys_from_result(result))
    declared = building_grammar.declared_rules_from_grammar(tiny)
    never = [
        r
        for r in declared
        if building_grammar.rule_step_key(str(r["layer"]), str(r["rule_id"])) not in fired
    ]
    dead_ids = {r["rule_id"] for r in never}
    assert "dead_never_massing" in dead_ids
    assert "abandoned_never" in dead_ids
    assert "massing:alive_rect" in fired


def test_pcg06_write_staging_report(tmp_path: Any, monkeypatch: pytest.MonkeyPatch) -> None:
    tiny = _tiny_grammar_with_dead_rule()
    monkeypatch.setattr(
        building_grammar,
        "load_building_grammar_by_archetype",
        lambda _aid: tiny,
    )

    entries = [
        {"archetype_id": "TinyArchetype", "district_style": "test_style", "seed": s}
        for s in (1, 2, 3, 4)
    ]
    body = kit_coverage_audit.write_pcg06_grammar_rule_coverage_report(entries=entries, repo=tmp_path)
    assert body.get("written") == kit_coverage_audit.PCG_06_STAGING_REL
    out = tmp_path / kit_coverage_audit.PCG_06_STAGING_REL
    assert out.is_file()
    assert body.get("report_only") is True
    never = body.get("never_fire_rules") or []
    assert any(r.get("rule_id") == "dead_never_massing" for r in never)
    table = body.get("designer_action_table") or []
    assert any(r.get("category") == "never_fire" for r in table)


def test_pcg06_golden_seed_report_when_witness_present() -> None:
    from rust_engine_mcp import golden_seed_review

    seeds = golden_seed_review.load_golden_seeds(repo=repo_root())
    if len(seeds) < 1:
        pytest.skip("BQ-Q3 golden witness missing")
    body = kit_coverage_audit.run_grammar_rule_coverage_report(entries=seeds[:3], repo=repo_root())
    assert body.get("task_id") == kit_coverage_audit.PCG_06_TASK_ID
    assert "designer_action_table" in body
    assert body.get("golden_seed_count") == 3