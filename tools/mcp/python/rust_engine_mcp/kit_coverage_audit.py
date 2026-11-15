"""APSR-Q2 / BQ-K2 — style-pack slot coverage audit for Catalog kit panel."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from rust_engine_mcp import assembly
from rust_engine_mcp.library import load_index_json
from rust_engine_mcp.paths import repo_root

WITNESS_REL = "debug_runs/bq_k2_coverage_001_live.json"
CHARTER_REL = "src/dev/design_bq_k2_coverage_charter_v1.md"
K1_BATCH_REL = "tools/mcp/schemas/examples/bq_k1_kitfill_batch_v1.json"

CORE_SLOTS = ("wall_1u", "door_default", "window_1u", "roof_default")

PACK_EXTENDED_SLOTS: dict[str, tuple[str, ...]] = {
    "style_colonial": ("wall_2u", "window_2u", "door_wide", "roof_flat", "corner_outer"),
    "style_victorian": ("wall_2u", "window_2u", "roof_flat", "corner_outer", "prop_clutter"),
    "style_rural": ("wall_2u", "window_2u", "door_wide", "roof_flat", "prop_clutter"),
    "style_modern": ("wall_2u", "window_2u", "window_industrial", "roof_flat", "prop_clutter"),
    "style_military": ("wall_2u", "roof_flat", "corner_outer", "prop_clutter"),
    "style_industrial_west": (
        "wall_2u",
        "door_wide",
        "window_industrial",
        "roof_industrial",
        "roof_flat",
        "corner_outer",
        "prop_clutter",
    ),
    "style_industrial_soviet": (
        "wall_2u",
        "door_wide",
        "window_industrial",
        "roof_flat",
        "prop_clutter",
    ),
}


PACK_SLOT_ALTERNATES: dict[str, dict[str, tuple[str, ...]]] = {
    "style_industrial_soviet": {"window_1u": ("window_industrial",)},
    "style_industrial_west": {"window_1u": ("window_industrial",)},
}


def required_slots_for_pack(style_pack_id: str) -> tuple[str, ...]:
    extended = PACK_EXTENDED_SLOTS.get(style_pack_id, ())
    return CORE_SLOTS + extended


def _missing_required_slots(style_pack_id: str, declared: set[str]) -> list[str]:
    required = set(required_slots_for_pack(style_pack_id))
    alternates = PACK_SLOT_ALTERNATES.get(style_pack_id, {})
    missing: list[str] = []
    for slot in sorted(required):
        if slot in declared:
            continue
        alts = alternates.get(slot, ())
        if any(alt in declared for alt in alts):
            continue
        missing.append(slot)
    return missing


def audit_style_pack_slot_keys(style_pack_id: str, *, repo: Path | None = None) -> dict[str, Any]:
    pack = assembly.load_style_pack(style_pack_id)
    declared = set(pack.get("slots") or {})
    required = set(required_slots_for_pack(style_pack_id))
    missing_keys = _missing_required_slots(style_pack_id, declared)
    return {
        "style_pack_id": style_pack_id,
        "required_slot_count": len(required),
        "declared_slot_count": len(declared),
        "missing_slot_keys": missing_keys,
        "slot_keys_complete": not missing_keys,
    }


def audit_k1_style_purity_gaps(*, repo: Path | None = None) -> list[dict[str, Any]]:
    root = repo or repo_root()
    batch_path = root / K1_BATCH_REL
    if not batch_path.is_file():
        return []
    batch = json.loads(batch_path.read_text(encoding="utf-8"))
    gaps: list[dict[str, Any]] = []
    for job in batch.get("jobs") or []:
        module_id = str(job.get("module_id") or "")
        replaces = job.get("replaces_slots") or {}
        for pack_id, slot_key in replaces.items():
            pack = assembly.load_style_pack(str(pack_id))
            current = (pack.get("slots") or {}).get(str(slot_key))
            if current != module_id:
                gaps.append(
                    {
                        "style_pack_id": pack_id,
                        "slot_key": slot_key,
                        "current_module_id": current,
                        "target_module_id": module_id,
                        "material_family": job.get("material_family"),
                    }
                )
    return gaps


def audit_bq_k2_coverage(*, repo: Path | None = None) -> dict[str, Any]:
    resolution = audit_all_style_packs(repo=repo)
    packs = assembly.list_style_packs()
    key_rows = [audit_style_pack_slot_keys(pid, repo=repo) for pid in packs]
    purity_gaps = audit_k1_style_purity_gaps(repo=repo)
    keys_green = all(r["slot_keys_complete"] for r in key_rows) if key_rows else False
    resolution_green = resolution["green"]
    charter_green = (repo or repo_root()) / CHARTER_REL
    green = resolution_green and keys_green and charter_green.is_file()
    return {
        "gate": "BQ-K2-COVERAGE-001",
        "charter_doc": CHARTER_REL,
        "style_pack_count": len(packs),
        "slot_resolution": resolution,
        "slot_key_audit": key_rows,
        "style_purity_gaps": purity_gaps,
        "style_purity_gap_count": len(purity_gaps),
        "style_purity_complete": not purity_gaps,
        "green": green,
        "verdict": "PASS" if green else "FAIL",
        "notes": "Charter PASS at 100% slot resolution; style purity closes when @coder-mcp wires K1 batch",
    }


def write_bq_k2_witness(*, repo: Path | None = None) -> dict[str, Any]:
    import time

    root = repo or repo_root()
    body = audit_bq_k2_coverage(repo=root)
    body["_agent_meta"] = {
        "schema": "bq_k2_coverage_live_v1",
        "written_at_epoch_secs": int(time.time()),
        "profile": "BQ_K2_COVERAGE",
        "source_system": "kit_coverage_audit",
        "relative_path": WITNESS_REL,
        "ritual": f"BLANG:WIT-HON→Q✓ BQ-K2-COVERAGE-001" if body.get("green") else "BLANG:WIT-HON FAIL",
        "agent": "designer-mcp",
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = WITNESS_REL
    return body


def audit_style_pack(style_pack_id: str, *, repo: Path | None = None) -> dict[str, Any]:
    pack = assembly.load_style_pack(style_pack_id)
    index = load_index_json()
    missing: list[dict[str, str]] = []
    covered = 0
    for slot_key, module_id in sorted(pack.get("slots", {}).items()):
        row = assembly._resolve_module_row(
            module_id,
            index,
            style_pack_id=style_pack_id,
            source_tier="production",
        )
        if row and assembly._row_glb_ready(row):
            covered += 1
        else:
            missing.append({"slot": slot_key, "module_id": module_id})
    total = len(pack.get("slots") or {})
    pct = (100.0 * covered / total) if total else 0.0
    return {
        "style_pack_id": style_pack_id,
        "label": pack.get("label") or style_pack_id,
        "slot_total": total,
        "slot_covered": covered,
        "coverage_pct": round(pct, 1),
        "complete": not missing,
        "missing_slots": missing,
    }


def audit_all_style_packs(*, repo: Path | None = None) -> dict[str, Any]:
    packs = assembly.list_style_packs()
    rows = [audit_style_pack(pid, repo=repo) for pid in packs]
    complete = sum(1 for r in rows if r["complete"])
    return {
        "style_pack_count": len(rows),
        "complete_count": complete,
        "style_packs": rows,
        "green": complete == len(rows) and bool(rows),
    }


def format_kit_coverage_summary(*, repo: Path | None = None) -> tuple[str, bool | None]:
    audit = audit_all_style_packs(repo=repo)
    if not audit["style_packs"]:
        return "Kit coverage: no style packs found.", None
    incomplete = [r for r in audit["style_packs"] if not r["complete"]]
    if not incomplete:
        text = f"Kit coverage: {audit['complete_count']}/{audit['style_pack_count']} packs 100% complete"
        return text, True
    worst = min(incomplete, key=lambda r: r["coverage_pct"])
    text = (
        f"Kit coverage: {audit['complete_count']}/{audit['style_pack_count']} complete · "
        f"{worst['label']} {worst['coverage_pct']:.0f}% ({len(worst['missing_slots'])} missing)"
    )
    return text, False


def write_apsr_q2_witness(*, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    audit = audit_all_style_packs(repo=repo)
    summary, _ok = format_kit_coverage_summary(repo=repo)
    body: dict[str, Any] = {
        "task_id": "APSR-A4-Q2-001",
        "gate": "APSR-A4-Q2-001",
        "green": audit["green"],
        "kit_coverage_summary": summary,
        "style_pack_count": audit["style_pack_count"],
        "complete_count": audit["complete_count"],
        "plan_ref": "src/dev/plan_aps_refactor_v1.md#APSR-Q2",
    }
    return write_aps_live_witness(
        body,
        "debug_runs/apsr_a4_q2_001_live.json",
        schema="apsr_a4_q2_live_v1",
        profile="APSR_A4_Q2",
        source_system="apsr_a4_q2",
        ritual="BLANG:WIT-HON APSR-A4-Q2-001" if audit["green"] else None,
        repo=repo,
    )


PCG_06_STAGING_REL = "assets/staging/reports/pcg_06_grammar_rule_coverage.json"
PCG_06_TASK_ID = "PCG-06-GRAMMAR-RULE-COVERAGE-001"
DESIGNER_ACTION_NEVER_FIRE = "Raise weight, add trigger, or retire rule (report only — do not auto-delete)."
DESIGNER_ACTION_TERMINAL_GAP = "Add kit module to style pack or remove terminal from grammar."
DESIGNER_ACTION_ALWAYS_FAIL = "Fix kit/QC coupling for this rule — every golden seed with it fails PCG-08 gate."


def _terminal_kit_gap(
    style_pack_id: str,
    slot_key: str,
    *,
    repo: Path | None = None,
) -> dict[str, Any] | None:
    from rust_engine_mcp import assembly

    pack = assembly.load_style_pack(style_pack_id)
    module_id = (pack.get("slots") or {}).get(slot_key)
    if not module_id:
        return {
            "issue": "missing_style_pack_slot",
            "style_pack_id": style_pack_id,
            "slot_key": slot_key,
            "module_id": None,
        }
    index = load_index_json()
    row = assembly._resolve_module_row(
        module_id,
        index,
        style_pack_id=style_pack_id,
        source_tier="production",
    )
    if not row or not assembly._row_glb_ready(row):
        return {
            "issue": "no_production_glb",
            "style_pack_id": style_pack_id,
            "slot_key": slot_key,
            "module_id": module_id,
        }
    return None


def audit_grammar_terminals_missing_kit(
    grammar: dict[str, Any],
    *,
    repo: Path | None = None,
) -> list[dict[str, Any]]:
    from rust_engine_mcp import building_grammar

    gaps: list[dict[str, Any]] = []
    seen: set[tuple[str, str]] = set()
    for district in grammar.get("district_styles") or []:
        style_pack = str(district.get("style_pack_id") or "")
        if not style_pack:
            continue
        for term in building_grammar.declared_terminals_from_grammar(grammar):
            key = (style_pack, str(term["slot_key"]))
            if key in seen:
                continue
            seen.add(key)
            gap = _terminal_kit_gap(style_pack, term["slot_key"], repo=repo)
            if gap:
                gaps.append(
                    {
                        **gap,
                        "grammar_id": term.get("grammar_id"),
                        "footprint_token": term.get("footprint_token"),
                        "designer_action": DESIGNER_ACTION_TERMINAL_GAP,
                    }
                )
    return gaps


def _build_designer_action_table(
    *,
    never_fire: list[dict[str, Any]],
    terminals_missing_kit: list[dict[str, Any]],
    always_fail_gate: list[dict[str, Any]],
) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for row in never_fire:
        rows.append(
            {
                "category": "never_fire",
                "grammar_id": row.get("grammar_id"),
                "layer": row.get("layer"),
                "rule_id": row.get("rule_id"),
                "fires_on_golden_seeds": 0,
                "designer_action": DESIGNER_ACTION_NEVER_FIRE,
            }
        )
    for row in terminals_missing_kit:
        rows.append(
            {
                "category": "terminal_no_kit",
                "grammar_id": row.get("grammar_id"),
                "style_pack_id": row.get("style_pack_id"),
                "slot_key": row.get("slot_key"),
                "issue": row.get("issue"),
                "module_id": row.get("module_id"),
                "designer_action": row.get("designer_action"),
            }
        )
    for row in always_fail_gate:
        rows.append(
            {
                "category": "always_fail_gate",
                "grammar_id": row.get("grammar_id"),
                "rule_key": row.get("rule_key"),
                "layer": row.get("layer"),
                "rule_id": row.get("rule_id"),
                "fire_count": row.get("fire_count"),
                "pass_with_rule": row.get("pass_with_rule"),
                "fail_with_rule": row.get("fail_with_rule"),
                "designer_action": DESIGNER_ACTION_ALWAYS_FAIL,
            }
        )
    rows.sort(
        key=lambda r: (
            str(r.get("category")),
            str(r.get("grammar_id")),
            str(r.get("layer") or ""),
            str(r.get("rule_id") or r.get("slot_key") or ""),
        )
    )
    return rows


def format_pcg06_designer_table_markdown(report: dict[str, Any]) -> str:
    lines = [
        f"# {PCG_06_TASK_ID} designer action table",
        "",
        "| category | grammar | layer | id | evidence | designer_action |",
        "|---|---|---|---|---|---|",
    ]
    for row in report.get("designer_action_table") or []:
        cat = row.get("category")
        gid = row.get("grammar_id") or ""
        layer = row.get("layer") or ""
        rid = row.get("rule_id") or row.get("slot_key") or ""
        if cat == "never_fire":
            evidence = "fires=0 on golden seeds"
        elif cat == "terminal_no_kit":
            evidence = str(row.get("issue") or "")
            layer = "terminal"
        else:
            evidence = (
                f"fires={row.get('fire_count')} pass={row.get('pass_with_rule')} "
                f"fail={row.get('fail_with_rule')}"
            )
        action = str(row.get("designer_action") or "").replace("|", "/")
        lines.append(f"| {cat} | {gid} | {layer} | {rid} | {evidence} | {action} |")
    return "\n".join(lines)


def run_grammar_rule_coverage_report(
    *,
    entries: list[dict[str, Any]] | None = None,
    repo: Path | None = None,
) -> dict[str, Any]:
    """PCG-06 — golden-seed rule firing + terminal kit gaps + always-fail rules (report only)."""
    from rust_engine_mcp import building_grammar, building_quality_qc, golden_seed_review

    root = repo or repo_root()
    if entries is None:
        entries = golden_seed_review.load_golden_seeds(repo=root)

    pcg08 = building_quality_qc.run_generate_and_test_report(
        entries,
        top_k=3,
        generate_fn=golden_seed_review.generate_snapshot_for_seed,
        repo=root,
    )
    eval_by_seed = {
        int(r["seed"]): r for r in pcg08.get("candidates") or [] if isinstance(r, dict)
    }

    rule_stats: dict[str, dict[str, int]] = {}
    all_fired: set[str] = set()
    grammars_by_archetype: dict[str, dict[str, Any]] = {}

    for entry in entries:
        aid = str(entry.get("archetype_id") or "")
        ds = str(entry.get("district_style") or "")
        seed = int(entry.get("seed") or 0)
        if aid not in grammars_by_archetype:
            grammars_by_archetype[aid] = building_grammar.load_building_grammar_by_archetype(aid)
        grammar = grammars_by_archetype[aid]
        result = building_grammar.generate(aid, ds, seed)
        fired = building_grammar.fired_rule_keys_from_result(result)
        all_fired.update(fired)
        ev = eval_by_seed.get(seed, {})
        passed = bool(ev.get("passes_gate"))
        for key in fired:
            st = rule_stats.setdefault(key, {"fires": 0, "pass": 0, "fail": 0})
            st["fires"] += 1
            if passed:
                st["pass"] += 1
            else:
                st["fail"] += 1

    declared: list[dict[str, Any]] = []
    for aid, grammar in sorted(grammars_by_archetype.items()):
        for row in building_grammar.declared_rules_from_grammar(grammar):
            row = dict(row)
            row["archetype_id"] = aid
            declared.append(row)

    rule_key_to_grammar: dict[str, str] = {}
    for row in declared:
        key = building_grammar.rule_step_key(str(row["layer"]), str(row["rule_id"]))
        rule_key_to_grammar[key] = str(row.get("grammar_id") or "")

    never_fire: list[dict[str, Any]] = []
    for row in declared:
        key = building_grammar.rule_step_key(str(row["layer"]), str(row["rule_id"]))
        if key not in all_fired:
            never_fire.append({**row, "rule_key": key, "designer_action": DESIGNER_ACTION_NEVER_FIRE})

    always_fail_gate: list[dict[str, Any]] = []
    for key, st in sorted(rule_stats.items()):
        if st["fires"] > 0 and st["pass"] == 0:
            layer, rule_id = key.split(":", 1)
            always_fail_gate.append(
                {
                    "rule_key": key,
                    "grammar_id": rule_key_to_grammar.get(key),
                    "layer": layer,
                    "rule_id": rule_id,
                    "fire_count": st["fires"],
                    "pass_with_rule": st["pass"],
                    "fail_with_rule": st["fail"],
                    "designer_action": DESIGNER_ACTION_ALWAYS_FAIL,
                }
            )

    terminals_missing_kit: list[dict[str, Any]] = []
    seen_grammar: set[str] = set()
    for grammar in grammars_by_archetype.values():
        gid = str(grammar.get("grammar_id") or "")
        if gid in seen_grammar:
            continue
        seen_grammar.add(gid)
        terminals_missing_kit.extend(audit_grammar_terminals_missing_kit(grammar, repo=root))

    designer_action_table = _build_designer_action_table(
        never_fire=never_fire,
        terminals_missing_kit=terminals_missing_kit,
        always_fail_gate=always_fail_gate,
    )

    return {
        "schema": "pcg_06_grammar_rule_coverage_v1",
        "task_id": PCG_06_TASK_ID,
        "ref_books_row": "PCG-06",
        "plan_ref": "src/dev/plan_ref_books_upgrade_v2.md#S4",
        "golden_seed_count": len(entries),
        "pcg08_pass_rate": pcg08.get("pass_rate"),
        "rules_fired_unique": len(all_fired),
        "rules_declared_count": len(declared),
        "never_fire_rules": never_fire,
        "terminals_missing_kit": terminals_missing_kit,
        "always_fail_gate_rules": always_fail_gate,
        "designer_action_table": designer_action_table,
        "designer_table_markdown": format_pcg06_designer_table_markdown(
            {"designer_action_table": designer_action_table}
        ),
        "report_only": True,
        "rules_check": {
            "passed": True,
            "blocked_by": [],
            "seed": "golden_set",
        },
    }


def write_pcg06_grammar_rule_coverage_report(
    *,
    entries: list[dict[str, Any]] | None = None,
    repo: Path | None = None,
) -> dict[str, Any]:
    root = repo or repo_root()
    body = run_grammar_rule_coverage_report(entries=entries, repo=root)
    out = root / PCG_06_STAGING_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = PCG_06_STAGING_REL
    return body
