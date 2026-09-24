"""BQ-SMOKE-AUDIT-001 — inventory why assemble still picks lod0/smoke despite kit-fill green.

Kit-fill / BQ-K2 coverage can be green while generators still land greybox-adjacent geometry:
coverage treats any GLB-ready resolve (incl. lod0 fallback) as covered, and style-pack-scoped
pools prefer same-pack lod0 over cross-pack production.
"""

from __future__ import annotations

import json
import time
from collections import Counter
from pathlib import Path
from typing import Any

from rust_engine_mcp import assembly
from rust_engine_mcp.kit_coverage_audit import CORE_SLOTS
from rust_engine_mcp.library import load_index_json
from rust_engine_mcp.paths import repo_root

TASK_ID = "BQ-SMOKE-AUDIT-001"
GATE = "BQ-SMOKE-AUDIT-001"
WITNESS_REL = "debug_runs/bq_smoke_tier_audit_live.json"
NEXT_SLICE = "SPINE-RENDER-001"
FOCUS_PACKS = ("style_industrial_west", "style_victorian", "style_colonial")
DEFERRED_PACKS = (
    "style_rural",
    "style_modern",
    "style_military",
    "style_industrial_soviet",
)
MODULE_INDEX_REL = "assets/configs/buildings/_module_index.json"

# Promote targets for NEXT_SLICE — highest-traffic lod0 slots on focus packs.
# Priority: P0 = CORE_SLOTS · P1 = extended pack slots used by grammar generate.
PROMOTE_PRIORITY: dict[str, str] = {
    "window_1u": "P0",
    "wall_1u": "P0",
    "door_default": "P0",
    "roof_default": "P0",
    "wall_2u": "P1",
    "door_wide": "P1",
    "roof_flat": "P1",
    "corner_outer": "P1",
    "window_2u": "P1",
    "window_industrial": "P1",
    "roof_industrial": "P1",
    "prop_clutter": "P2",
}


def _tier_of(row: dict[str, Any] | None) -> str:
    if not row:
        return "missing"
    tier = str(row.get("development_tier") or "")
    batch = str(row.get("batch_id") or "")
    if tier == "smoke" or batch.startswith("kit_greybox") or batch.startswith("kit_smoke"):
        return "smoke"
    if tier in ("production", "lod0"):
        return tier
    if "production" in str(row.get("job_id") or ""):
        return "production"
    if "lod0" in str(row.get("job_id") or ""):
        return "lod0"
    return tier or "unknown"


def count_index_tiers(index: list[dict[str, Any]] | None = None) -> dict[str, Any]:
    rows = index if index is not None else load_index_json()
    tiers = Counter(str(r.get("development_tier") or "unknown") for r in rows)
    batches = Counter(str(r.get("batch_id") or "") for r in rows)
    greybox_batches = {
        bid: n
        for bid, n in batches.items()
        if bid.startswith("kit_greybox") or bid.startswith("kit_smoke")
    }
    return {
        "production": int(tiers.get("production", 0)),
        "lod0": int(tiers.get("lod0", 0)),
        "smoke": int(tiers.get("smoke", 0)),
        "unknown": int(tiers.get("unknown", 0)),
        "total_entries": len(rows),
        "greybox_batch_counts": dict(sorted(greybox_batches.items())),
        "top_batches": [{"batch_id": b, "count": n} for b, n in batches.most_common(12)],
    }


def _rows_for_module(module_id: str, index: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return [r for r in index if str(r.get("module_id") or "") == module_id]


def resolve_slot_row(
    *,
    style_pack_id: str,
    slot_key: str,
    module_id: str,
    index: list[dict[str, Any]],
    source_tier: str = "production",
) -> dict[str, Any]:
    rows = _rows_for_module(module_id, index)
    avail_tiers = sorted({str(r.get("development_tier") or "") for r in rows})
    avail_packs = sorted({str(r.get("style_pack") or "") for r in rows})
    prod_rows = [r for r in rows if str(r.get("development_tier") or "") == "production"]
    pack_prod = [
        r for r in prod_rows if str(r.get("style_pack") or "") == style_pack_id
    ]
    cross_pack_prod = [
        r for r in prod_rows if str(r.get("style_pack") or "") != style_pack_id
    ]
    resolved = assembly._resolve_module_row(
        module_id,
        index,
        style_pack_id=style_pack_id,
        source_tier=source_tier,
    )
    resolved_tier = _tier_of(resolved)
    reason = "production_hit"
    if resolved is None:
        reason = "unresolved"
    elif resolved_tier == "lod0":
        if pack_prod:
            reason = "pack_production_glb_missing"
        elif cross_pack_prod:
            reason = "style_pack_pool_prefers_same_pack_lod0_over_cross_pack_production"
        elif prod_rows:
            reason = "production_row_present_but_not_selected"
        else:
            reason = "no_production_row_lod0_fallback"
    elif resolved_tier == "smoke":
        reason = "smoke_selected"
    return {
        "slot": slot_key,
        "module_id": module_id,
        "resolved_job_id": None if not resolved else str(resolved.get("job_id") or ""),
        "resolved_tier": resolved_tier,
        "resolved_batch_id": None if not resolved else str(resolved.get("batch_id") or ""),
        "resolved_style_pack": None if not resolved else str(resolved.get("style_pack") or ""),
        "available_tiers": avail_tiers,
        "available_style_packs": avail_packs,
        "production_row_count": len(prod_rows),
        "same_pack_production_count": len(pack_prod),
        "cross_pack_production_count": len(cross_pack_prod),
        "reason": reason,
        "non_production": resolved_tier in ("lod0", "smoke", "missing"),
        "is_core_slot": slot_key in CORE_SLOTS,
    }


def audit_style_pack_tiers(
    style_pack_id: str,
    *,
    index: list[dict[str, Any]] | None = None,
) -> dict[str, Any]:
    idx = index if index is not None else load_index_json()
    pack = assembly.load_style_pack(style_pack_id)
    slots = pack.get("slots") or {}
    slot_rows = [
        resolve_slot_row(
            style_pack_id=style_pack_id,
            slot_key=str(slot),
            module_id=str(mid),
            index=idx,
        )
        for slot, mid in sorted(slots.items())
    ]
    selectable_lod0 = sorted(
        {
            r["module_id"]
            for r in slot_rows
            if r["resolved_tier"] == "lod0" and r["module_id"]
        }
    )
    selectable_smoke = sorted(
        {
            r["module_id"]
            for r in slot_rows
            if r["resolved_tier"] == "smoke" and r["module_id"]
        }
    )
    non_prod = [r for r in slot_rows if r["non_production"]]
    core_non_prod = [r for r in non_prod if r["is_core_slot"]]
    return {
        "style_pack_id": style_pack_id,
        "label": pack.get("label") or style_pack_id,
        "slot_total": len(slot_rows),
        "production_slots": sum(1 for r in slot_rows if r["resolved_tier"] == "production"),
        "lod0_slots": sum(1 for r in slot_rows if r["resolved_tier"] == "lod0"),
        "smoke_slots": sum(1 for r in slot_rows if r["resolved_tier"] == "smoke"),
        "missing_slots": sum(1 for r in slot_rows if r["resolved_tier"] == "missing"),
        "selectable_lod0_ids": selectable_lod0,
        "selectable_smoke_ids": selectable_smoke,
        "core_non_production": [
            {"slot": r["slot"], "module_id": r["module_id"], "resolved_tier": r["resolved_tier"]}
            for r in core_non_prod
        ],
        "can_pick_smoke_or_lod0_standard_slot": bool(core_non_prod),
        "slots": slot_rows,
        "production_only": not non_prod,
    }


def inventory_selectable_smoke(index: list[dict[str, Any]]) -> dict[str, Any]:
    """Smoke rows still in the index — hidden from stylepack but inventory residue."""
    smoke_rows = [r for r in index if str(r.get("development_tier") or "") == "smoke"]
    visible = [r for r in smoke_rows if r.get("stylepack_visible") is True]
    replaced = [r for r in smoke_rows if r.get("replaced_by")]
    orphan = [
        r
        for r in smoke_rows
        if not r.get("replaced_by") and r.get("stylepack_visible") is not True
    ]
    return {
        "smoke_index_count": len(smoke_rows),
        "stylepack_visible_true_count": len(visible),
        "replaced_by_count": len(replaced),
        "orphan_unreplaced_count": len(orphan),
        "selectable_smoke_ids": sorted(
            {str(r["module_id"]) for r in visible}
            | {
                # Orphans remain pickable if a caller bypasses stylepack_visible.
                str(r["module_id"])
                for r in orphan
            }
        ),
        "stylepack_visible_smoke_ids": sorted({str(r["module_id"]) for r in visible}),
        "orphan_smoke_ids": sorted({str(r["module_id"]) for r in orphan}),
        "note": (
            "Assembly _resolve_lod0_module skips smoke/greybox via stylepack_visible=false "
            "+ replaced_by; residue still inflates index and can be re-picked if resolve "
            "bypasses those guards."
        ),
    }


def build_promote_block_lists(
    per_pack: list[dict[str, Any]],
) -> dict[str, Any]:
    """Concrete lists for BQ-PROD-PROMOTE-BATCH-001 (focus packs first)."""
    promote: list[dict[str, Any]] = []
    seen: set[tuple[str, str]] = set()
    for pack_row in per_pack:
        pack_id = str(pack_row["style_pack_id"])
        if pack_id not in FOCUS_PACKS:
            continue
        for slot in pack_row.get("slots") or []:
            if not slot.get("non_production"):
                continue
            mid = str(slot["module_id"])
            key = (pack_id, mid)
            if key in seen:
                continue
            seen.add(key)
            slot_key = str(slot["slot"])
            promote.append(
                {
                    "priority": PROMOTE_PRIORITY.get(slot_key, "P2"),
                    "style_pack_id": pack_id,
                    "slot": slot_key,
                    "module_id": mid,
                    "current_job_id": slot.get("resolved_job_id"),
                    "current_tier": slot.get("resolved_tier"),
                    "current_batch_id": slot.get("resolved_batch_id"),
                    "reason": slot.get("reason"),
                    "suggested_production_job_id": f"{mid}_production_run001",
                    "action": "rebake+promote production tier for module_id; refresh module index",
                }
            )
    promote.sort(key=lambda r: (r["priority"], r["style_pack_id"], r["slot"]))

    # Block: do not promote raw greybox smoke as production; defer non-focus packs.
    block: list[dict[str, Any]] = [
        {
            "rule": "no_greybox_as_production",
            "detail": "⛔ promote kit_greybox_* / smoke-tier GLBs as production — rebake with production profiles",
        },
        {
            "rule": "ortho_not_ship",
            "detail": "⛔ ortho/lod0 headless bake as ship:true — bake_source must be keyframe_pack for production",
        },
        {
            "rule": "defer_non_focus_packs",
            "detail": (
                "Deferred packs rural/modern/military/industrial_soviet CORE lod0 handled by "
                "BQ-PROD-DEFER-PACKS-001 (promote+retire); residual non-core lod0 → SPINE-RENDER-001"
            ),
            "deferred_packs": [
                p["style_pack_id"]
                for p in per_pack
                if p["style_pack_id"] not in FOCUS_PACKS and not p.get("production_only")
            ],
        },
    ]
    # Victorian already production-only — note as no-op.
    victorian = next((p for p in per_pack if p["style_pack_id"] == "style_victorian"), None)
    if victorian and victorian.get("production_only"):
        block.append(
            {
                "rule": "victorian_already_production",
                "detail": "style_victorian standard+extended slots already resolve production — skip rebake",
            }
        )
    return {
        "promote": promote,
        "promote_count": len(promote),
        "block": block,
        "focus_packs": list(FOCUS_PACKS),
        "next_slice": NEXT_SLICE,
    }


def why_kit_fill_green_still_lod0(
    counts: dict[str, Any],
    per_pack: list[dict[str, Any]],
    smoke_inv: dict[str, Any],
) -> list[dict[str, str]]:
    focus_lod0 = sum(p.get("lod0_slots", 0) for p in per_pack if p["style_pack_id"] in FOCUS_PACKS)
    return [
        {
            "id": "KF-1",
            "cause": (
                f"Module index still majority non-production "
                f"(production={counts.get('production')} lod0={counts.get('lod0')} "
                f"smoke={counts.get('smoke')})"
            ),
        },
        {
            "id": "KF-2",
            "cause": (
                "BQ-K1 kit-fill green proves new production modules exist on disk; "
                "it does not retarget style-pack slots that still point at lod0-only module_ids"
            ),
        },
        {
            "id": "KF-3",
            "cause": (
                "BQ-K2 / kit_coverage_audit marks a slot covered when _resolve_module_row "
                "returns any GLB-ready row — including lod0 fallback after production miss"
            ),
        },
        {
            "id": "KF-4",
            "cause": (
                "Style-pack-scoped pool prefers same-pack lod0 over cross-pack production "
                "(assembly._resolve_module_row pack_rows filter)"
            ),
        },
        {
            "id": "KF-5",
            "cause": (
                f"Smoke rows remain in index ({smoke_inv.get('smoke_index_count')}) with "
                f"stylepack_visible=false; assemble skips them but index residue + orphan "
                f"ids ({smoke_inv.get('orphan_unreplaced_count')}) stay selectable if guards bypassed"
            ),
        },
        {
            "id": "KF-6",
            "cause": (
                f"Focus-pack lod0 slot hits at assemble time: {focus_lod0} "
                f"(industrial_west + colonial still select lod0 for some slots; victorian production-only)"
            ),
        },
    ]


def audit_smoke_tier(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    index_path = root / MODULE_INDEX_REL
    index = load_index_json()
    counts = count_index_tiers(index)
    packs = assembly.list_style_packs()
    per_pack = [audit_style_pack_tiers(pid, index=index) for pid in packs]
    smoke_inv = inventory_selectable_smoke(index)
    lists = build_promote_block_lists(per_pack)

    # Union of smoke ids that style packs could still name + index residue.
    per_pack_smoke: list[str] = []
    per_pack_lod0: list[str] = []
    for p in per_pack:
        per_pack_smoke.extend(p.get("selectable_smoke_ids") or [])
        per_pack_lod0.extend(p.get("selectable_lod0_ids") or [])
    selectable_smoke_ids = sorted(set(per_pack_smoke) | set(smoke_inv.get("selectable_smoke_ids") or []))
    selectable_lod0_ids = sorted(set(per_pack_lod0))

    any_standard_non_prod = any(p.get("can_pick_smoke_or_lod0_standard_slot") for p in per_pack)
    focus_rows = [p for p in per_pack if p["style_pack_id"] in FOCUS_PACKS]
    focus_production_only = all(p.get("production_only") for p in focus_rows) if focus_rows else False

    # Exit: green=false while any style pack can pick smoke/lod0 for a standard slot.
    green = not any_standard_non_prod and focus_production_only and counts.get("smoke", 0) == 0
    return {
        "gate": GATE,
        "task_id": TASK_ID,
        "program": "PLAN-MCP-APS-TOOLING-FINISH-001",
        "module_index": MODULE_INDEX_REL,
        "module_index_exists": index_path.is_file(),
        "counts": counts,
        "per_style_pack": per_pack,
        "selectable_smoke_ids": selectable_smoke_ids,
        "selectable_lod0_ids": selectable_lod0_ids,
        "smoke_inventory": smoke_inv,
        "focus_packs": {
            "ids": list(FOCUS_PACKS),
            "production_only": focus_production_only,
            "rows": [
                {
                    "style_pack_id": p["style_pack_id"],
                    "production_only": p["production_only"],
                    "lod0_slots": p["lod0_slots"],
                    "smoke_slots": p["smoke_slots"],
                    "selectable_lod0_ids": p["selectable_lod0_ids"],
                    "core_non_production": p["core_non_production"],
                }
                for p in focus_rows
            ],
        },
        "promote_block_lists": lists,
        "why_kit_fill_green_still_picks_lod0": why_kit_fill_green_still_lod0(
            counts, per_pack, smoke_inv
        ),
        "any_style_pack_picks_smoke_or_lod0_standard_slot": any_standard_non_prod,
        "green": green,
        "verdict": "PASS" if green else "FAIL",
        "honest_gate": "honest_gate",
        "proceed_ship": False,
        "art_quality": "audit_only",
        "next_residual": NEXT_SLICE,
        "notes": (
            "Audit inventory. Focus + deferred CORE slots production-only and smoke index cleared "
            f"after BQ-PROD-DEFER-PACKS-001. Residual non-core lod0 → {NEXT_SLICE}."
            if green
            else (
                "Audit only — residual: deferred CORE lod0 and/or smoke index orphans → "
                "BQ-PROD-DEFER-PACKS-001 (or refresh after promote/retire)."
                if focus_production_only
                else (
                    "Audit only — no promote. Next: BQ-PROD-PROMOTE-BATCH-001 rebakes promote[] "
                    "for industrial_west + colonial lod0 slots (victorian already production-only)."
                )
            )
        ),
    }


def write_bq_smoke_tier_audit_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    body = audit_smoke_tier(repo=root)
    body["_agent_meta"] = {
        "schema": "bq_smoke_tier_audit_live_v1",
        "written_at_epoch_secs": int(time.time()),
        "profile": "BQ_SMOKE_TIER_AUDIT",
        "source_system": "bq_smoke_tier_audit",
        "relative_path": WITNESS_REL,
        "track": "PLAN-MCP-APS-TOOLING-FINISH-001",
        "task_id": TASK_ID,
        "proceed_ship": False,
        "art_quality": "audit_only",
        "agent": "coder-mcp",
        "ritual": (
            "BLANG:WIT-HON→Q✓ BQ-SMOKE-AUDIT-001"
            if body.get("green")
            else f"BLANG:WIT-HON FAIL residual→{NEXT_SLICE}"
        ),
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = WITNESS_REL
    return body
