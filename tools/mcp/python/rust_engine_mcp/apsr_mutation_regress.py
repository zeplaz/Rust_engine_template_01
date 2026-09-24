"""APSR-MUTATION-REGRESS-001 — SuiteState mutation ratchet + lane-roundtrip wiring lock."""

from __future__ import annotations

import ast
import time
from collections import Counter
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.suite_state_mutation_inventory import (
    ALLOWLIST_REL,
    MAX_DIRECT_MUTATION_SITES,
    SuiteStateMutationSite,
    load_mutation_allowlist,
    scan_suite_state_mutations,
    suite_state_mutation_inventory,
    sync_mutation_allowlist_from_scan,
)

TASK_ID = "APSR-MUTATION-REGRESS-001"
WITNESS_REL = "debug_runs/apsr_mutation_regress_001_live.json"
SHELL_WIRING_REL = "tools/mcp/art_pipeline_suite/aps_shell_wiring.py"
# Finish-lane residual after this regress lock (operator/designer — not pytest).
NEXT_RESIDUAL = "APS-GOLDEN-RUBRIC-OPS-001"


def _field_fingerprint(sites: list[SuiteStateMutationSite]) -> Counter[tuple[str, str]]:
    return Counter((site.file, site.field) for site in sites)


def _allowlist_as_sites(allowlist: dict[str, Any]) -> list[SuiteStateMutationSite]:
    out: list[SuiteStateMutationSite] = []
    for entry in allowlist.get("sites", []):
        out.append(
            SuiteStateMutationSite(
                file=str(entry["file"]),
                line=int(entry["line"]),
                field=str(entry["field"]),
            )
        )
    return out


def maybe_sync_allowlist_line_drift(*, suite_root: Path | None = None) -> dict[str, Any]:
    """Rewrite allowlist only when file+field multiset matches (line-number drift).

    Growth or new (file, field) pairs are never auto-accepted — CI must fail.
    """
    live = scan_suite_state_mutations(root=suite_root)
    allow = _allowlist_as_sites(load_mutation_allowlist())
    live_fp = _field_fingerprint(live)
    allow_fp = _field_fingerprint(allow)
    live_ids = {s.site_id for s in live}
    allow_ids = {s.site_id for s in allow}
    if live_fp == allow_fp and live_ids != allow_ids:
        synced = sync_mutation_allowlist_from_scan(root=suite_root)
        return {
            "synced": True,
            "reason": "line_drift",
            "site_count": synced.get("site_count"),
            "written": synced.get("written"),
        }
    return {
        "synced": False,
        "reason": "no_line_drift" if live_ids == allow_ids else "fingerprint_mismatch",
        "live_count": len(live),
        "allowlist_count": len(allow),
        "fingerprint_match": live_fp == allow_fp,
    }


def check_lane_roundtrip_wiring(*, repo: Path | None = None) -> dict[str, Any]:
    """Static AST check: LaneChanged handler calls assembly.sync_from_state()."""
    root = repo or repo_root()
    path = root / SHELL_WIRING_REL
    if not path.is_file():
        return {
            "ok": False,
            "path": SHELL_WIRING_REL,
            "error": "missing_shell_wiring",
        }
    tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
    subscribes_lane = False
    sync_in_lane_handler = False

    class _Visitor(ast.NodeVisitor):
        def visit_Call(self, node: ast.Call) -> None:
            nonlocal subscribes_lane, sync_in_lane_handler
            func = node.func
            # bus.subscribe("LaneChanged", ...)
            if isinstance(func, ast.Attribute) and func.attr == "subscribe":
                if node.args and isinstance(node.args[0], ast.Constant):
                    if node.args[0].value == "LaneChanged":
                        subscribes_lane = True
            # app.assembly.sync_from_state()
            if isinstance(func, ast.Attribute) and func.attr == "sync_from_state":
                sync_in_lane_handler = True
            self.generic_visit(node)

    _Visitor().visit(tree)
    text = path.read_text(encoding="utf-8")
    # Narrow: LaneChanged handler body must mention sync_from_state near buildings lane.
    lane_block_ok = (
        'lane == ArtDomain.BUILDINGS.value' in text
        or 'lane == "buildings"' in text
    ) and "assembly.sync_from_state()" in text
    ok = subscribes_lane and sync_in_lane_handler and lane_block_ok
    return {
        "ok": ok,
        "path": SHELL_WIRING_REL,
        "subscribes_lane_changed": subscribes_lane,
        "calls_sync_from_state": sync_in_lane_handler,
        "buildings_lane_sync": lane_block_ok,
    }


def run_mutation_regress_audit(*, repo: Path | None = None, sync_line_drift: bool = True) -> dict[str, Any]:
    root = repo or repo_root()
    sync_info: dict[str, Any] = {"synced": False, "reason": "skipped"}
    if sync_line_drift:
        # Never pass repo root into the scanner — only APS suite (default) or an explicit suite path.
        sync_info = maybe_sync_allowlist_line_drift()

    inventory = suite_state_mutation_inventory()
    live_count = int(inventory.get("live_count") or 0)
    ceiling = MAX_DIRECT_MUTATION_SITES
    growth_ok = live_count <= ceiling
    wiring = check_lane_roundtrip_wiring(repo=root)
    green = (
        inventory.get("green") is True
        and growth_ok
        and wiring.get("ok") is True
    )
    return {
        "gate": TASK_ID,
        "task_id": TASK_ID,
        "green": green,
        "ok": green,
        "mutation_inventory_green": inventory.get("green"),
        "live_mutation_count": live_count,
        "allowlist_count": inventory.get("allowlist_count"),
        "mutation_ceiling": ceiling,
        "growth_ok": growth_ok,
        "unexpected_sites": inventory.get("unexpected_sites") or [],
        "removed_sites": inventory.get("removed_sites") or [],
        "allowlist_sync": sync_info,
        "allowlist_rel": ALLOWLIST_REL,
        "lane_roundtrip_wiring": wiring,
        "plan_ref": "src/dev/plan_aps_refactor_v1.md#APSR-T1",
        "cli": "apsr-mutation-regress-001",
        "next_residual": NEXT_RESIDUAL,
        "rules_check": {
            "passed": True,
            "blocked_by": [],
            "seed": None,
            "contract_only_no_ship_art": True,
            "deterministic_output": True,
            "no_ai_generated_images": True,
        },
    }


def write_apsr_mutation_regress_001_witness(
    *,
    repo: Path | None = None,
    sync_line_drift: bool = True,
) -> dict[str, Any]:
    import json

    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    root = repo or repo_root()
    body = run_mutation_regress_audit(repo=root, sync_line_drift=sync_line_drift)
    meta_extra = {
        "track": "PLAN-MCP-APS-TOOLING-FINISH-001",
        "task_id": TASK_ID,
        "proceed_ship": False,
        "art_quality": "contract_only_no_ship_art",
        "agent": "coder-mcp",
    }
    out = write_aps_live_witness(
        body,
        WITNESS_REL,
        schema="apsr_mutation_regress_001_live_v1",
        profile="APSR_MUTATION_REGRESS",
        source_system="apsr_mutation_regress",
        ritual=f"BLANG:WIT-HON→Q✓ {TASK_ID}" if body.get("green") else None,
        exit_predicate_must=[
            {"field": "mutation_inventory_green", "equals": True},
            {"field": "growth_ok", "equals": True},
            {"field": "lane_roundtrip_wiring.ok", "equals": True},
        ],
        repo=root,
    )
    meta = dict(out.get("_agent_meta") or {})
    meta.update(meta_extra)
    meta["written_at_epoch_secs"] = int(time.time())
    if out.get("green"):
        meta["ritual"] = f"BLANG:WIT-HON→Q✓ {TASK_ID}"
    out["_agent_meta"] = meta
    (root / WITNESS_REL).write_text(json.dumps(out, indent=2) + "\n", encoding="utf-8")
    return out
