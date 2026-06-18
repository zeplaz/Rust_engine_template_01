"""Designer grammar iteration loop — compressed tier + guards + sweep (no chat prose)."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Literal

from . import building_grammar, grammar_build_set
from .paths import repo_root

DESIGNER_GRAMMAR_LOOP_WITNESS = "debug_runs/designer_grammar_quality_loop_live.json"


def _tier_next_actions(tier_body: dict[str, Any], brief: dict[str, Any]) -> list[str]:
    tier = str(tier_body.get("tier") or "G0")
    actions: list[str] = []
    archetype_count = int(tier_body.get("archetype_count") or 0)

    if tier == "G0":
        actions.append(
            "GRAM-CONTENT-002: add >=2 grammar *.ron + JSON mirrors "
            "(see src/dev/design_grammar_archetype_family_g1_v1.md)"
        )
        actions.append("validate each new grammar: validate-report arch_build_grammar on preset JSON twin")
        actions.append("refresh tier: python -m rust_engine_mcp.cli grammar-set-tier --write-witness")
    elif tier == "G1":
        gaps = brief.get("gaps") or []
        if gaps:
            actions.append(f"grammar_set_brief gaps: {'; '.join(gaps[:4])}")
        actions.append("G2: ship >=4 ARCH-DNA presets with distinct F-axis values")
        actions.append("grammar_preset_pair_validate each new preset ↔ pilot catalog row")
    elif tier == "G2":
        actions.append("G3: extend grammar rule_chain — facade + detail + age layers in snapshots")
        actions.append("grammar_eval_sweep per archetype — massing histogram must show >=2 strategies")
    elif tier == "G3":
        actions.append("G4: building_set_coverage + grammar_pilot_parity must green")
        actions.append("close pilot_hardcode_lint violations before ship")
    else:
        actions.append("maintain: grammar_eval_sweep on edit; building-set-coverage before promote")

    if archetype_count < 3 and tier != "G0":
        actions.insert(0, f"regression: archetype_count={archetype_count} (expected >=3 at G1+)")

    return actions[:8]


def _grammar_file_checks() -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for aid in building_grammar.list_archetype_ids():
        districts = building_grammar.list_district_styles(aid)
        row: dict[str, Any] = {
            "archetype_id": aid,
            "district_count": len(districts),
            "schema_ok": False,
            "generate_ok": False,
            "error": None,
        }
        try:
            grammar = building_grammar.load_building_grammar_by_archetype(aid)
            row["grammar_id"] = grammar.get("grammar_id")
            row["schema_ok"] = True
            if districts:
                building_grammar.generate(aid, districts[0], 42)
                row["generate_ok"] = True
        except Exception as exc:  # noqa: BLE001
            row["error"] = str(exc)
        rows.append(row)
    return rows


def run_designer_grammar_quality_loop(
    *,
    mode: Literal["fast", "full"] = "fast",
    sweep_seeds: int = 24,
    write_witness: bool = False,
) -> dict[str, Any]:
    """Compressed designer loop — tier, brief, coverage, optional per-archetype sweeps."""
    tier_body = grammar_build_set.grammar_set_tier()
    brief = grammar_build_set.grammar_set_brief()
    coverage = grammar_build_set.building_set_coverage_report()
    parity = grammar_build_set.grammar_pilot_parity()
    grammar_checks = _grammar_file_checks()

    sweeps: list[dict[str, Any]] = []
    if mode == "full":
        seeds = list(range(40, 40 + max(4, sweep_seeds)))
        for aid in building_grammar.list_archetype_ids():
            districts = building_grammar.list_district_styles(aid)
            if not districts:
                continue
            sweep = grammar_build_set.grammar_eval_sweep(
                archetype_id=aid,
                district_style=districts[0],
                seeds=seeds,
            )
            sweeps.append(
                {
                    "archetype_id": aid,
                    "district_style": districts[0],
                    "green": sweep.get("green"),
                    "massing_histogram": sweep.get("massing_histogram"),
                    "errors": sweep.get("errors"),
                }
            )

    schema_fail = [r for r in grammar_checks if not r.get("schema_ok")]
    generate_fail = [r for r in grammar_checks if r.get("schema_ok") and not r.get("generate_ok")]
    sweep_fail = [s for s in sweeps if not s.get("green")]

    tier = str(tier_body.get("tier") or "G0")
    green = not schema_fail and not generate_fail and (mode == "fast" or not sweep_fail)
    if tier in ("G3", "G4"):
        green = green and bool(coverage.get("green")) and bool(parity.get("green"))

    body: dict[str, Any] = {
        "task_id": "DESIGNER-GRAMMAR-QUALITY-LOOP-001",
        "ok": True,
        "green": green,
        "mode": mode,
        "tier": tier,
        "tier_detail": {
            "archetype_count": tier_body.get("archetype_count"),
            "reasons": tier_body.get("reasons"),
            "grammar_files": tier_body.get("grammar_files"),
        },
        "brief_green": brief.get("green"),
        "brief_gaps": brief.get("gaps") or [],
        "coverage_green": coverage.get("green"),
        "coverage_errors": (coverage.get("errors") or [])[:6],
        "parity_green": parity.get("green"),
        "parity_errors": (parity.get("errors") or [])[:4],
        "grammar_checks": grammar_checks,
        "sweeps": sweeps if mode == "full" else [],
        "next_actions": _tier_next_actions(tier_body, brief),
        "cli_fast": "powershell tools/mcp/scripts/designer_grammar_iterate.ps1",
        "cli_full": "powershell tools/mcp/scripts/designer_grammar_iterate.ps1 -Mode full",
    }

    if write_witness:
        out = repo_root() / DESIGNER_GRAMMAR_LOOP_WITNESS
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
        body["witness_path"] = str(out.relative_to(repo_root())).replace("\\", "/")

    return body
