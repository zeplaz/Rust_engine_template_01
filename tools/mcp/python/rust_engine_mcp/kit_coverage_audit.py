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
