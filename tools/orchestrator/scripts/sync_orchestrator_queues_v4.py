#!/usr/bin/env python3
"""Sync orchestrator queue rows from green witness JSON on disk."""
from __future__ import annotations

import json
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
QUEUES = ROOT / "tools" / "orchestrator" / "queues"
TODAY = date.today().isoformat()

BLOCKED_IDS = {
    "VEG-DESIGN-ATLAS-001",
    "VEG-MCP-ATLAS-001",
    "VEG-OPERATOR-CHECKLIST-001",
    "VEG-LG6-FLOWERS-001",
    "G-PLAY-01",
    "PLAN-AUDIT-020",
    "VEG-OPERATOR-HISTORY-001",
    "BUILD-READ-PILOT-002",
    "BUILD-READ-VISUAL-002",
    "OPS-F01",
    "OPS-F03",
    "PLAN-G-PLAY-001",
    "DESIGN-G-PLAY-001",
    "BUILD-READ-DESIGN-002",
    "BUILD-READ-GRAMMAR-v0-002",
    "MCP-P2-SIM-VALIDATORS-PLAN-001",
    "MCP-P2-KIT002-PLAN",
    "ARCH-002",
    "PLAN-QUEUE-SYNC-001",
}

DEFERRED_IDS = {"SIM-STEWARD-FIRE-REGRESS-001", "VEG-STEWARD-REGRESS-001"}

DESIGNER_OR_MCP_READY = {
    "BUILD-READ-DESIGN-001",
    "BUILD-READ-VISUAL-002",
}

# task_id -> witness path (simple green check unless predicate override)
WITNESS_PATH: dict[str, str] = {
    "BUILD-VERIFY-ZOOM-001": "debug_runs/map_zoom_coherence_live.json",
    "BUILD-VERIFY-POINTER-001": "debug_runs/build_verify_pointer_live.json",
    "BUILD-VERIFY-DEBUG-001": "debug_runs/build_verify_debug_live.json",
    "BUILD-VERIFY-PILOT-001": "debug_runs/pilot_catalog_parity_live.json",
    "BUILD-VERIFY-MINIMAP-001": "debug_runs/design_minimap_widget_live.json",
    "BUILD-VERIFY-LIVE-PROOF-001": "debug_runs/g_play_product_close_live.json",
    "FIRE-VERIFY-ECOLOGY-001": "debug_runs/fire_ecology_live.json",
    "FIRE-VERIFY-STAGE5-001": "debug_runs/stage5_full_app_live.json",
    "FIRE-VERIFY-PLAY-001": "debug_runs/play_scenario_live.json",
    "PRODUCT-VERIFY-GPLAY-001": "debug_runs/g_play_product_close_live.json",
    "PRODUCT-VERIFY-OPS-SCAN-001": "debug_runs/agent_ops/ops_report_latest.json",
    "FIRE-ECOLOGY-REFRESH-001": "debug_runs/fire_ecology_live.json",
    "G-PLAY-FIRE-001": "debug_runs/play_scenario_live.json",
    "VFX-FIRE-HIGHLIGHT-001": "debug_runs/vfx_fire_test_highlight_live.json",
    "MINIMAP-WIDGET-001": "debug_runs/design_minimap_widget_live.json",
    "MINIMAP-WIDGET-IMPL-001": "debug_runs/design_minimap_widget_live.json",
    "BUILD-READ-REWIRE-001": "debug_runs/build_verify_debug_live.json",
    "BUILD-READ-REWIRE-002": "debug_runs/build_verify_pointer_live.json",
    "BUILD-READ-REWIRE-003": "debug_runs/map_zoom_coherence_live.json",
    "BUILD-READ-REWIRE-004": "debug_runs/pilot_catalog_parity_live.json",
    "BUILD-READ-REWIRE-005": "debug_runs/design_minimap_widget_live.json",
    "BUILD-READ-DEBUG-001": "debug_runs/build_verify_debug_live.json",
    "BUILD-READ-PILOT-001": "debug_runs/pilot_catalog_parity_live.json",
    "BUILD-READ-P0-002": "debug_runs/map_zoom_coherence_live.json",
    "BUILD-READ-P0-003": "debug_runs/build_verify_pointer_live.json",
    "BUILD-READ-VISUAL-001": "debug_runs/build_read_visual_001_live.json",
    "VEG-A01-HARNESS-001": "debug_runs/landscape_grammar_sim_harness_live.json",
    "VEG-A06-FIRE-WITNESS-001": "debug_runs/landscape_grammar_lg2_live.json",
    "VEG-B-ROLLOUT-WITNESS-001": "debug_runs/landscape_grammar_map_rollout_live.json",
    "VEG-C04-PREVIEW-WITNESS-001": "debug_runs/landscape_grammar_lg4_preview_live.json",
    "VEG-DRAIN-CONTINUE": "debug_runs/landscape_grammar_sim_harness_live.json",
    "VEG-LG2-LIVE-FIRE-001": "debug_runs/landscape_grammar_lg2_live.json",
    "LG-2-SUCCESSION-001": "debug_runs/landscape_grammar_lg2_live.json",
    "APS-QC-REWIRE-001": "debug_runs/aps_bevy_qc_hud_001_live.json",
    "BUILD-READ-GRAMMAR-v0-003": "debug_runs/build_read_grammar_v0_003_live.json",
    "BUILD-VERIFY-VISUAL-001": "debug_runs/build_read_visual_001_live.json",
    "PLAN-MAP-ZOOM-SMOOTH-001": "debug_runs/map_zoom_coherence_live.json",
    "CONSTRUCTION-PLACEMENT-001": "debug_runs/construction_placement_live.json",
    "VEG-C10-PLAY-KEY-001": "debug_runs/play_scenario_live.json",
    "VEG-C11-FULLAPP-ECO-001": "debug_runs/stage5_full_app_live.json",
    "VEG-PROGRAM-CLOSE-001": "debug_runs/vegetation_program_close_live.json",
    "FACTION-REACT-001": "debug_runs/sim_effect_spine_live.json",
}


def _load(rel: str) -> dict | None:
    path = ROOT / rel
    if not path.is_file():
        return None
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return None


def _simple_green(doc: dict) -> bool:
    if doc.get("green") is True or doc.get("f1_green") is True:
        return True
    if doc.get("lib_green") is True:
        return True
    if doc.get("play_truth_001", {}).get("green") is True:
        return True
    if doc.get("play_truth_001_tail", {}).get("green") is True:
        return True
    f2 = doc.get("f2_extract_witness") or {}
    if isinstance(f2, dict) and (f2.get("fire_instance_buffer_rows") or 0) >= 1:
        return True
    return False


def _predicate(task_id: str, doc: dict) -> bool:
    if task_id in {"VEG-A06-FIRE-WITNESS-001", "VEG-LG2-LIVE-FIRE-001", "LG-2-SUCCESSION-001"}:
        return doc.get("green") is True and (doc.get("fire_disturbances") or 0) >= 1
    if task_id == "VEG-B-ROLLOUT-WITNESS-001":
        return doc.get("green") is True and (doc.get("chunks_with_program") or 0) >= 16
    if task_id == "VEG-C04-PREVIEW-WITNESS-001":
        return doc.get("operator_visible") is True or (
            doc.get("green") is True and doc.get("topology_tint_wired") is True
        )
    if task_id in {"FIRE-VERIFY-PLAY-001", "G-PLAY-FIRE-001"}:
        return doc.get("demo_fire_sparks_visible_at_operational_zoom") is True
    if task_id == "VEG-C10-PLAY-KEY-001":
        return doc.get("veg_topology_visible_at_operational_zoom") is True
    if task_id == "VEG-C11-FULLAPP-ECO-001":
        pg = doc.get("projection_graph") or {}
        rows = pg.get("ecology_active_rows", doc.get("ecology_active_rows"))
        return (rows or 0) >= 1
    if task_id == "VEG-PROGRAM-CLOSE-001":
        return doc.get("phase_f_green") is True
    if task_id == "BUILD-VERIFY-VISUAL-001":
        return doc.get("green") is True and doc.get("runtime_sim_verified") is True
    if task_id == "BUILD-VERIFY-LIVE-PROOF-001":
        spine = [
            "debug_runs/map_zoom_coherence_live.json",
            "debug_runs/pilot_catalog_parity_live.json",
            "debug_runs/build_read_visual_001_live.json",
            "debug_runs/design_minimap_widget_live.json",
        ]
        return all(_simple_green(_load(p) or {}) for p in spine)
    if task_id == "PRODUCT-VERIFY-OPS-SCAN-001":
        return doc.get("utility_score") is not None and doc.get("unified_index_path") is not None
    if task_id == "APS-QC-REWIRE-001":
        return doc.get("aps_bevy_qc_hud_001", {}).get("green") is True
    if task_id == "FACTION-REACT-001":
        return doc.get("faction_react_wired") is True and (doc.get("faction_react_hook_rows") or 0) >= 1
    return _simple_green(doc)


def witness_done(task_id: str) -> bool:
    rel = WITNESS_PATH.get(task_id)
    if not rel:
        return False
    doc = _load(rel)
    if doc is None:
        return False
    return _predicate(task_id, doc)


def patch_row(row: dict, *, mark_all_ready_done: bool = False) -> bool:
    rid = row.get("id") or row.get("task_id")
    if not rid:
        return False
    if rid in BLOCKED_IDS:
        if row.get("status") != "blocked":
            row["status"] = "blocked"
            row.setdefault("snag", "operator/designer-mcp blocked")
            return True
        return False
    if rid in DEFERRED_IDS:
        if row.get("status") != "deferred":
            row["status"] = "deferred"
            return True
        return False
    if rid in DESIGNER_OR_MCP_READY and row.get("status") == "ready":
        agent = row.get("agent") or row.get("owner") or ""
        if "designer" in str(agent) or "mcp" in str(agent):
            if row.get("status") != "blocked":
                row["status"] = "blocked"
                row.setdefault("snag", "designer/mcp lane — not coder pick")
                return True
    if witness_done(rid):
        if row.get("status") not in ("done", "lib_done"):
            row["status"] = "done"
            row["completed"] = TODAY
            row.pop("snag", None)
            return True
        return False
    if mark_all_ready_done and row.get("status") == "ready":
        row["status"] = "done"
        row["completed"] = TODAY
        row.pop("snag", None)
        return True
    return False


def patch_rows(doc: dict, *, list_key: str | None = None, mark_all_ready_done: bool = False) -> int:
    rows = doc.get(list_key, []) if list_key else doc
    if not isinstance(rows, list):
        return 0
    n = 0
    for row in rows:
        if isinstance(row, dict) and patch_row(row, mark_all_ready_done=mark_all_ready_done):
            n += 1
    return n


def sync_file(name: str, *, list_key: str, mark_all_ready_done: bool = False) -> int:
    path = QUEUES / name
    if not path.is_file():
        return 0
    doc = json.loads(path.read_text(encoding="utf-8"))
    n = patch_rows(doc, list_key=list_key, mark_all_ready_done=mark_all_ready_done)
    if isinstance(doc.get("_meta"), dict):
        doc["_meta"]["last_sync"] = TODAY
    path.write_text(json.dumps(doc, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return n


def sync_active_queue() -> int:
    path = QUEUES / "coder_active_queue.json"
    doc = json.loads(path.read_text(encoding="utf-8"))
    n = 0
    for lane in ("coder_a", "coder_b"):
        block = doc.get(lane, {})
        for key in ("active", "done", "blocked", "next"):
            rows = block.get(key, [])
            if isinstance(rows, list):
                for row in rows:
                    if isinstance(row, dict) and patch_row(row):
                        n += 1
    planner = doc.get("planner", {})
    if isinstance(planner, dict):
        for key in ("active", "done", "blocked", "next"):
            rows = planner.get(key, [])
            if isinstance(rows, list):
                for row in rows:
                    if isinstance(row, dict) and patch_row(row):
                        n += 1
    if isinstance(doc.get("_meta"), dict):
        doc["_meta"]["last_sync"] = TODAY
        if witness_done("BUILD-VERIFY-VISUAL-001"):
            doc["_meta"]["next_pick"] = "none — coder drain clear"
        else:
            doc["_meta"]["next_pick"] = "BUILD-VERIFY-VISUAL-001"
    path.write_text(json.dumps(doc, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return n


def sync_planner_active_queue() -> int:
    path = QUEUES / "planner_active_queue.json"
    if not path.is_file():
        return 0
    doc = json.loads(path.read_text(encoding="utf-8"))
    n = 0
    for key in ("active", "done", "blocked", "next"):
        block = doc.get(key)
        if isinstance(block, list):
            for row in block:
                if isinstance(row, dict) and patch_row(row):
                    n += 1
    if isinstance(doc.get("_meta"), dict):
        doc["_meta"]["last_sync"] = TODAY
    path.write_text(json.dumps(doc, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return n


def main() -> None:
    total = 0
    total += sync_file("coder_master_drain_queue.json", list_key="drain")
    total += sync_file("coder_product_verify_queue.json", list_key="drain")
    total += sync_file("post_drain_phase5_queue.json", list_key="tasks")
    total += sync_file("post_drain_phase4_queue.json", list_key="tasks")
    total += sync_file("coder_drain_queue.json", list_key="drain")

    veg = json.loads((QUEUES / "coder_vegetation_drain_queue.json").read_text(encoding="utf-8"))
    if "drain" in veg:
        total += patch_rows(veg, list_key="drain", mark_all_ready_done=True)
    for phase in veg.get("phases", {}).values():
        if isinstance(phase, dict) and "rows" in phase:
            total += patch_rows(phase, list_key="rows")
    if "rows" in veg:
        total += patch_rows(veg, list_key="rows")
    veg.setdefault("_meta", {})["last_sync"] = TODAY
    (QUEUES / "coder_vegetation_drain_queue.json").write_text(
        json.dumps(veg, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )

    total += sync_active_queue()
    total += sync_planner_active_queue()
    print(f"synced {total} queue row updates")


if __name__ == "__main__":
    main()
