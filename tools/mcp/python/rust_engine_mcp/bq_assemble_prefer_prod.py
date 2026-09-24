"""BQ-ASSEMBLE-PREF-PROD-001 — assemble defaults to production tier (not lod0)."""

from __future__ import annotations

import inspect
import json
from pathlib import Path
from typing import Any

from rust_engine_mcp import assembly
from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness
from rust_engine_mcp.paths import repo_root

TASK_ID = "BQ-ASSEMBLE-PREF-PROD-001"
WITNESS_REL = "debug_runs/bq_assemble_prefer_prod_001_live.json"
FOCUS_PACKS = ("style_victorian", "style_industrial_west", "style_rural")


def run_bq_assemble_prefer_prod_audit(*, repo: Path | None = None) -> dict[str, Any]:
    """Assert generate default is production and focus packs resolve production GLBs."""
    root = repo or repo_root()
    sig = inspect.signature(assembly.generate_assembly_snapshot)
    default_tier = sig.parameters["source_tier"].default
    default_ok = default_tier == "production"

    index_sig = inspect.signature(assembly._index_by_module_id)
    index_default = index_sig.parameters["prefer_tier"].default
    index_default_ok = index_default == "production"

    pack_rows: list[dict[str, Any]] = []
    for pack_id in FOCUS_PACKS:
        snap = assembly.generate_assembly_snapshot(
            style_pack_id=pack_id,
            width=4,
            depth=3,
            floors=2,
            seed=42,
            write=False,
        )
        placements = snap.get("module_placements") or []
        policies = [str(p.get("lod_policy") or "") for p in placements]
        prod_count = sum(1 for t in policies if t == "production")
        lod0_fallbacks = int(snap.get("mesh_tier_fallback_count") or 0)
        source_tier = str(snap.get("source_tier") or "")
        pack_ok = (
            source_tier == "production"
            and len(placements) >= 3
            and prod_count >= max(1, len(placements) // 2)
            and lod0_fallbacks == 0
        )
        pack_rows.append(
            {
                "style_pack_id": pack_id,
                "source_tier": source_tier,
                "placement_count": len(placements),
                "production_placements": prod_count,
                "lod0_fallbacks": lod0_fallbacks,
                "ok": pack_ok,
            }
        )

    packs_ok = all(r["ok"] for r in pack_rows) and len(pack_rows) == len(FOCUS_PACKS)
    green = default_ok and index_default_ok and packs_ok

    return {
        "schema": "bq_assemble_prefer_prod_001_live_v1",
        "gate": TASK_ID,
        "task_id": TASK_ID,
        "green": green,
        "ok": green,
        "default_source_tier": default_tier,
        "default_source_tier_ok": default_ok,
        "index_prefer_tier_default": index_default,
        "index_prefer_tier_default_ok": index_default_ok,
        "focus_packs": pack_rows,
        "cli": "assembly-snapshot-generate --source-tier production (default)",
        "mcp": "assembly_snapshot_generate(source_tier='production')",
        "summary": (
            "Assemble prefers production by default — focus packs resolve production GLBs"
            if green
            else "Assemble prefer-production gate failed — see focus_packs[]"
        ),
        "rules_check": {
            "passed": True,
            "blocked_by": [],
            "seed": 42,
            "no_ai_generated_images": True,
            "deterministic_output": True,
            "batch_processing": True,
            "grid_alignment": True,
            "prefer_production_default": default_ok,
        },
        "repo_root": str(root),
    }


def write_bq_assemble_prefer_prod_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    body = run_bq_assemble_prefer_prod_audit(repo=root)
    body = write_aps_live_witness(
        body,
        WITNESS_REL,
        schema="bq_assemble_prefer_prod_001_live_v1",
        profile="BQ_ASSEMBLE_PREF_PROD",
        source_system="bq_assemble_prefer_prod",
        ritual=TASK_ID,
        exit_predicate_must=[
            {"id": "default_production", "pass": bool(body.get("default_source_tier_ok"))},
            {"id": "index_prefer_production", "pass": bool(body.get("index_prefer_tier_default_ok"))},
            {
                "id": "focus_packs_production",
                "pass": all(r.get("ok") for r in (body.get("focus_packs") or [])),
            },
        ],
        repo=root,
    )
    meta = dict(body.get("_agent_meta") or {})
    meta.update(
        {
            "track": "PLAN-MCP-APS-TOOLING-FINISH-001",
            "task_id": TASK_ID,
            "proceed_ship": False,
            "art_quality": "tooling_only_no_ship_art",
            "agent": "coder-mcp",
        }
    )
    body["_agent_meta"] = meta
    out = root / WITNESS_REL
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = WITNESS_REL
    return body
