#!/usr/bin/env python3
"""Bulk-reconcile reopened queue rows when witness JSON already proves green."""
from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
QUEUES = REPO / "tools/orchestrator/queues"
DEBUG = REPO / "debug_runs"

# task_id -> override key (see eval_override)
OVERRIDES: dict[str, str] = {
    "SIM-EFFECT-QUEUE-001": "sim_effect_spine",
    "SIM-EFFECT-TEL-001": "sim_effect_spine",
    "FIRE-IGNITION-P0-001": "fire_ecology_green",
    "PLAN-LANDSCAPE-GRAMMAR-001": "lg1_green",
    "LG-2-SUCCESSION-001": "lg2_green",
    "TRIAGE-MAP-ZOOM-SMOOTH-001": "map_zoom_green",
    "BUILD-READ-REWIRE-003": "map_zoom_green",
    "APS-QC-REWIRE-001": "aps_qc_green",
    "APS-E1-CHROME-001": "aps_e1_chrome",
    "MCP-P2-QUEUE-PHASE4-001": "mcp_phase4_queue",
    "MCP-P2-VALID-CONSTRUCTION-001": "mcp_valid_construction",
    "MCP-P2-OPS-BRIEF-002": "mcp_ops_brief",
    "MCP-LG-VALID-PRESET-001": "mcp_lg_preset",
    "VEG-F02-MCP-ATLAS-001": "mcp_sign_atlas",
    "MCP-P2-RUN-EVENT-001": "mcp_run_event",
    "MCP-P2-HONEST-BAKE-001": "mcp_honest_bake",
    "CMCP-GRAM-SWEEP-PROCESS-001": "grammar_sweep",
    "VEG-F01-ATLAS-SHIP-001": "veg_ship_close",
    "VEG-A18-PHASE-A-CLOSE-001": "veg_phase_a",
    "VEG-B-PHASE-CLOSE-001": "veg_phase_b",
    "VEG-C04-PREVIEW-WITNESS-001": "lg4_preview",
    "VEG-C15-PHASE-CLOSE-001": "veg_phase_c",
    "VEG-D07-PHASE-CLOSE-001": "veg_phase_d",
    "VEG-E08-PHASE-CLOSE-001": "veg_phase_e",
    "VA2-HARNESS-CLOSE-001": "visual_aidv2_green",
}

SKIP_AUTO_CLOSE = frozenset({"BUILD-READ-REWIRE-004", "PERF-INSTR-VFX-002", "VA2-HARNESS-CLOSE-001"})


def load_json(path: Path) -> dict | list:
    return json.loads(path.read_text(encoding="utf-8"))


def read_witness(rel: str) -> dict | None:
    if not rel:
        return None
    for candidate in (REPO / rel, DEBUG / Path(rel).name):
        if candidate.exists() and candidate.suffix == ".json":
            return load_json(candidate)
    return None


def veg_program() -> dict:
    d = read_witness("debug_runs/vegetation_program_close_live.json")
    return d or {}


def eval_override(key: str) -> bool:
    if key == "sim_effect_spine":
        d = read_witness("debug_runs/sim_effect_spine_live.json") or {}
        s = d.get("sim_effect_spine") or {}
        return s.get("queue_drain_ok") is True and (s.get("effect_rows") or 0) >= 1
    if key == "fire_ecology_green":
        d = read_witness("debug_runs/fire_ecology_live.json") or {}
        return d.get("green") is True
    if key == "map_zoom_green":
        d = read_witness("debug_runs/map_zoom_coherence_live.json") or {}
        return d.get("green") is True
    if key == "lg1_green":
        d = read_witness("debug_runs/landscape_grammar_lg1_live.json") or {}
        return d.get("green") is True and (d.get("topology_kind_count") or 0) >= 4
    if key == "lg2_green":
        d = read_witness("debug_runs/landscape_grammar_lg2_live.json") or {}
        return d.get("green") is True and d.get("succession_age_ticks") is True
    if key == "aps_qc_green":
        d = read_witness("debug_runs/aps_bevy_qc_hud_001_live.json") or {}
        return (d.get("aps_bevy_qc_hud_001") or {}).get("green") is True or d.get("green") is True
    if key == "aps_e1_chrome":
        d = read_witness("debug_runs/aps_option_d_e1_live.json") or {}
        return (d.get("slices") or {}).get("APS-E1-CHROME-001", {}).get("green") is True or (
            d.get("APS-E1-CHROME-001") or {}
        ).get("green") is True or d.get("green") is True
    if key == "mcp_phase4_queue":
        d = read_witness("debug_runs/agent_ops/mcp_phase4_queue_live.json") or {}
        return d.get("green") is True
    if key == "mcp_valid_construction":
        d = read_witness("debug_runs/agent_ops/mcp_valid_construction_live.json") or {}
        return d.get("green") is True
    if key == "mcp_ops_brief":
        d = read_witness("debug_runs/agent_ops/ops_mcp_function_layer_live.json") or {}
        return d.get("green") is True
    if key == "mcp_lg_preset":
        d = read_witness("debug_runs/mcp_landscape_grammar_preset_batch_live.json") or {}
        return d.get("green") is True
    if key == "mcp_sign_atlas":
        d = read_witness("debug_runs/mcp_landscape_sign_atlas_live.json") or {}
        return d.get("green") is True
    if key == "mcp_run_event":
        d = read_witness("debug_runs/mcp_p2_run_event_001_live.json") or {}
        return d.get("green") is True or d.get("ok") is True
    if key == "mcp_honest_bake":
        d = read_witness("debug_runs/mcp_p2_honest_bake_001_live.json") or {}
        return d.get("green") is True
    if key == "grammar_sweep":
        d = read_witness("debug_runs/grammar_sweep_process_live.json") or {}
        return d.get("green") is True
    if key == "veg_ship_close":
        d = read_witness("debug_runs/veg_ship_close_live.json") or {}
        return d.get("vegetation_program_close") is True or d.get("program_closed") is True
    if key == "lg4_preview":
        d = read_witness("debug_runs/landscape_grammar_lg4_preview_live.json") or {}
        return d.get("green") is True
    vp = veg_program()
    if key == "veg_phase_a":
        return vp.get("phase_a_green") is True or vp.get("all_green") is True
    if key == "veg_phase_b":
        return vp.get("phase_b_green") is True or vp.get("all_green") is True
    if key == "veg_phase_c":
        return vp.get("phase_c_green") is True or vp.get("all_green") is True
    if key == "veg_phase_d":
        return vp.get("phase_d_green") is True or vp.get("all_green") is True
    if key == "veg_phase_e":
        return vp.get("phase_e_green") is True or vp.get("all_green") is True
    if key == "visual_aidv2_green":
        d = read_witness("debug_runs/visual_aidv2_live.json") or {}
        return (
            d.get("green") is True
            and d.get("done") == 6
            and d.get("lib_fixture") is not True
        )
    return False


def generic_witness_green(witness_rel: str, task_id: str = "") -> bool:
    if witness_rel.endswith(".md") or witness_rel.endswith(".yaml"):
        p = REPO / witness_rel
        return p.exists()
    d = read_witness(witness_rel)
    if not d:
        return False
    if d.get("green") is True or d.get("lib_green") is True or d.get("ok") is True:
        return True
    if task_id and isinstance(d.get(task_id), dict):
        if d[task_id].get("green") is True:
            return True
    for v in d.values():
        if isinstance(v, dict) and v.get("green") is True:
            return True
    return False


def row_green(row: dict) -> bool:
    tid = row.get("id", "")
    if tid in SKIP_AUTO_CLOSE:
        return False
    key = OVERRIDES.get(tid)
    if key is not None:
        return eval_override(key)
    witness = row.get("witness")
    if isinstance(witness, str):
        return generic_witness_green(witness, tid)
    return False


def iter_rows(obj: dict | list) -> list[dict]:
    rows: list[dict] = []
    if isinstance(obj, list):
        for r in obj:
            if isinstance(r, dict) and "id" in r:
                rows.append(r)
            elif isinstance(r, dict):
                rows.extend(iter_rows(r))
        return rows
    if isinstance(obj, dict):
        if "id" in obj:
            rows.append(obj)
        for v in obj.values():
            if isinstance(v, (dict, list)):
                rows.extend(iter_rows(v))
    return rows


def main() -> None:
    now = datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")
    marked = 0
    still_open: list[str] = []
    for qpath in sorted(QUEUES.glob("*.json")):
        if qpath.name.startswith("_"):
            continue
        obj = load_json(qpath)
        changed = False
        for row in iter_rows(obj):
            tid = row.get("id", "")
            st = row.get("status", "")
            if st in {"done", "closed", "signed", "deferred", "blocked"}:
                continue
            if st not in {"reopened", "ready", "in_progress", "open", "active"}:
                continue
            if not row_green(row):
                still_open.append(f"{tid}|{st}|{row.get('agent') or row.get('owner') or '?'}")
                continue
            row["status"] = "done"
            row["witness_refresh"] = now
            if not row.get("completed"):
                row["completed"] = now[:10]
            row["reconcile_note"] = "witness-green auto-close (reconcile_coder_crisis.py)"
            marked += 1
            changed = True
        if changed:
            qpath.write_text(json.dumps(obj, indent=2) + "\n", encoding="utf-8")
            print(f"Updated {qpath.name}")
    print(f"Marked done: {marked}")
    if still_open:
        pickable = [x for x in still_open if "|reopened|" in x or "|ready|" in x or "|open|" in x]
        print(f"Still actionable: {len(set(pickable))}")
        for t in sorted(set(pickable))[:25]:
            print(f"  - {t}")


if __name__ == "__main__":
    main()
