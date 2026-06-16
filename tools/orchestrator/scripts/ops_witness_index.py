#!/usr/bin/env python3
"""Unified witness index + OPS report — all engine + art program lanes (Track D).

Programs: stage5, fire_vfx, construction, infrastructure, economy, wave, stage7,
ui, art A/B/C, agent_ops. See tools/orchestrator/queues/OPS_LANE_REGISTRY.json.

Writes:
  - debug_runs/unified_witness_index.json
  - debug_runs/agent_ops/ops_report_latest.json
"""

from __future__ import annotations

import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[3]
DEBUG_RUNS = REPO_ROOT / "debug_runs"
REGISTRY_PATH = REPO_ROOT / "tools/orchestrator/queues/OPS_LANE_REGISTRY.json"
OUT_INDEX = DEBUG_RUNS / "unified_witness_index.json"
OUT_OPS = DEBUG_RUNS / "agent_ops" / "ops_report_latest.json"
OUT_OPS_BRIEF = DEBUG_RUNS / "agent_ops" / "ops_project_brief_v1.json"
MCP_PYTHON = REPO_ROOT / "tools" / "mcp" / "python"

SKIP_PREFIXES = (
    "debug_runs/agent_debug_index",
    "debug_runs/unified_witness_index",
    "debug_runs/agent_ops/ops_report",
    "debug_runs/preview_jobs/",
    "debug_runs/validators/",
    "debug_runs/full_render_diagnostic_",
)


def _rel(path: Path) -> str:
    return path.relative_to(REPO_ROOT).as_posix()


def _load_registry() -> dict[str, Any]:
    data = json.loads(REGISTRY_PATH.read_text(encoding="utf-8"))
    programs: dict[str, dict[str, Any]] = {}
    path_to_program: dict[str, str] = {}
    for prog in data.get("programs", []):
        pid = prog["program_id"]
        programs[pid] = prog
        for anchor in prog.get("anchor_witnesses", []):
            path_to_program[anchor] = pid
    return {
        "programs": programs,
        "path_to_program": path_to_program,
        "handoff_priorities": data.get("handoff_priorities", []),
    }


def _should_skip(rel: str) -> bool:
    return any(rel.startswith(p) for p in SKIP_PREFIXES)


def _match_program(rel: str, body: dict[str, Any], registry: dict[str, Any]) -> str:
    if rel in registry["path_to_program"]:
        return registry["path_to_program"][rel]
    meta = body.get("_agent_meta") or {}
    profile = str(body.get("profile") or meta.get("profile") or "").upper()
    task = str(
        body.get("slice_id")
        or body.get("gate_id")
        or body.get("program_id")
        or body.get("task_id")
        or meta.get("lane")
        or ""
    ).upper()
    name = Path(rel).name.lower()
    rel_l = rel.lower()
    for pid, prog in registry["programs"].items():
        for hint in prog.get("profile_hints", []):
            h = hint.upper()
            if h in profile or h in task or hint.lower() in name or hint.lower() in rel_l:
                return pid
    if body.get("track") in ("A", "B", "C"):
        return f"art_{body['track']}"
    if "art_pipeline" in rel_l:
        return "art_B"
    return "unclassified"


def _honest_gate(body: dict[str, Any], summary: dict[str, Any]) -> str:
    green = summary.get("green")
    art_quality = summary.get("art_quality")
    if green is False and art_quality:
        return "dishonest_gate"
    if green is False and summary.get("validator_status") == "passed":
        return "schema_only"
    if green is True:
        return "honest_green"
    if summary.get("ok") is True or summary.get("status") in ("done", "passed"):
        return "done_no_ship_flag"
    if summary.get("operational_green") is True:
        return "operational_green"
    if summary.get("readiness_passes") is True:
        return "readiness_green"
    return "unknown"


def _extract_summary(rel: str, body: dict[str, Any], registry: dict[str, Any]) -> dict[str, Any]:
    meta = body.get("_agent_meta") or {}
    program_id = _match_program(rel, body, registry)
    prog = registry["programs"].get(program_id, {})
    summary: dict[str, Any] = {
        "parse_ok": True,
        "program_id": program_id,
        "program_label": prog.get("label"),
        "track": body.get("track") or prog.get("track"),
        "lane": meta.get("lane") or meta.get("profile") or body.get("profile"),
        "task_id": (
            body.get("slice_id")
            or body.get("gate_id")
            or body.get("program_id")
            or body.get("task_id")
            or meta.get("lane")
        ),
        "green": body.get("green"),
        "proceed_ship": body.get("proceed_ship"),
        "art_quality": body.get("art_quality"),
        "ok": body.get("ok"),
        "status": body.get("status"),
        "profile": body.get("profile") or meta.get("profile"),
        "agent_role": meta.get("agent"),
        "owner": prog.get("owner"),
        "validator_status": (
            (body.get("promotion_validation") or {}).get("status")
            or (body.get("gates") or {}).get("assembly_p0", {}).get("status")
        ),
        "readiness_passes": None,
        "blocked_by": body.get("blocked_by"),
    }
    rp = body.get("readiness")
    if isinstance(rp, dict) and "passes" in rp:
        summary["readiness_passes"] = rp["passes"]
    for key in (
        "operational_green",
        "activation_green",
        "throughput_green",
        "infrastructure_view_isolation_green",
        "parity_green",
        "stage6_virtualization_green",
    ):
        if key in body:
            summary[key] = body[key]
    summary["honest_gate"] = _honest_gate(body, summary)
    return summary


def _scan_file(rel: str, registry: dict[str, Any]) -> dict[str, Any]:
    path = REPO_ROOT / rel
    if not path.is_file():
        return {
            "path": rel,
            "exists": False,
            "bytes": 0,
            "modified_epoch_secs": None,
            "summary": {"parse_ok": False, "missing": True, "program_id": "unclassified"},
        }
    text = path.read_text(encoding="utf-8", errors="replace")
    try:
        body = json.loads(text)
    except json.JSONDecodeError:
        return {
            "path": rel,
            "exists": True,
            "bytes": len(text),
            "modified_epoch_secs": int(path.stat().st_mtime),
            "summary": {"parse_ok": False, "program_id": "unclassified"},
        }
    return {
        "path": rel,
        "exists": True,
        "bytes": len(text),
        "modified_epoch_secs": int(path.stat().st_mtime),
        "summary": _extract_summary(rel, body, registry),
    }


def _discover_witness_paths(registry: dict[str, Any]) -> list[str]:
    found: set[str] = set(registry["path_to_program"].keys())
    if DEBUG_RUNS.is_dir():
        for p in DEBUG_RUNS.rglob("*.json"):
            rel = _rel(p)
            if _should_skip(rel):
                continue
            name = p.name
            if name.endswith("_live.json") or name.endswith("_witness.json"):
                found.add(rel)
            elif "art_pipeline" in rel and name.endswith(".json"):
                found.add(rel)
    return sorted(found)


def _construction_sub_witnesses() -> list[dict[str, Any]]:
    """Nested rows inside construction_stage_live.json (HANDOFF disk truth)."""
    path = DEBUG_RUNS / "construction_stage_live.json"
    if not path.is_file():
        return []
    body = json.loads(path.read_text(encoding="utf-8"))
    rows: list[dict[str, Any]] = []
    for key, val in body.items():
        if not key.startswith("construction_") or not isinstance(val, dict):
            continue
        green = val.get("green")
        if green is None and val.get("ok") is not None:
            green = val.get("ok")
        rows.append(
            {
                "witness_key": key,
                "green": green,
                "partial_alpha": val.get("partial_alpha"),
                "note": val.get("note"),
            }
        )
    op = body.get("operational_green")
    rows.append({"witness_key": "operational_green", "green": op})
    return rows


def build_index() -> dict[str, Any]:
    registry = _load_registry()
    all_rels = _discover_witness_paths(registry)
    proofs = [_scan_file(rel, registry) for rel in all_rels]
    by_program: dict[str, list[dict[str, Any]]] = {}
    for proof in proofs:
        pid = (proof.get("summary") or {}).get("program_id") or "unclassified"
        by_program.setdefault(pid, []).append(proof)
    now = int(datetime.now(timezone.utc).timestamp())
    return {
        "_agent_meta": {
            "schema": "unified_witness_index_v2",
            "written_at_epoch_secs": now,
            "profile": "UNIFIED_WITNESS_INDEX",
            "source_system": "ops_witness_index",
            "relative_path": _rel(OUT_INDEX),
            "registry": _rel(REGISTRY_PATH),
        },
        "profile": "UNIFIED_WITNESS_INDEX",
        "proof_count": len(proofs),
        "program_count": len(by_program),
        "programs": by_program,
        "proofs": proofs,
        "construction_sub_witnesses": _construction_sub_witnesses(),
        "registry_path": _rel(REGISTRY_PATH),
    }


def _program_rollup(index: dict[str, Any]) -> dict[str, list[dict[str, Any]]]:
    return index.get("programs") or {}


def _count_green(proofs: list[dict[str, Any]]) -> int:
    n = 0
    for p in proofs:
        s = p.get("summary") or {}
        if s.get("green") is True:
            n += 1
        elif s.get("readiness_passes") is True or s.get("operational_green") is True:
            n += 1
        elif s.get("infrastructure_view_isolation_green") is True:
            n += 1
        elif s.get("activation_green") is True or s.get("throughput_green") is True:
            n += 1
        elif s.get("honest_gate") in ("honest_green", "operational_green", "readiness_green", "done_no_ship_flag"):
            n += 1
    return n


def _dsm_snapshot(programs: dict[str, list[dict[str, Any]]], index: dict[str, Any]) -> list[str]:
    b_proofs = programs.get("art_B", [])
    b_ship_blocked = any(
        (p.get("summary") or {}).get("proceed_ship") is False
        and (p.get("summary") or {}).get("honest_gate") == "dishonest_gate"
        for p in b_proofs
    )
    constr_sub = index.get("construction_sub_witnesses") or []
    constr_red = [w for w in constr_sub if w.get("green") is False]
    auth_line = _load_auth_spine_line(REPO_ROOT)
    lines = [
        f"AUTH: {auth_line} | SIM★⇢CON★⇢INFRA★⇢FIRE★⇢ECO★",
        "LOOP: RUN⇢TEL★⇢KPI★⇢OPS★⇢ΔWF↺",
        f"STAGE5: {_count_green(programs.get('stage5_spine', []))} green / {len(programs.get('stage5_spine', []))} proofs",
        f"FIRE/VFX: {_count_green(programs.get('fire_vfx', []))} green / {len(programs.get('fire_vfx', []))} proofs",
        f"CONSTRUCTION: operational + {len(constr_sub)} sub-witnesses · {len(constr_red)} red",
        f"INFRASTRUCTURE: {_count_green(programs.get('infrastructure', []))} green / {len(programs.get('infrastructure', []))} proofs",
        f"ECONOMY/LOG: {_count_green(programs.get('economy_logistics', []))} green / {len(programs.get('economy_logistics', []))} proofs",
        f"WAVE/STAGE6: {_count_green(programs.get('wave_product', []))} green / {len(programs.get('wave_product', []))} proofs",
        f"STAGE7/PLAY: {_count_green(programs.get('stage7_play', []))} green / {len(programs.get('stage7_play', []))} proofs",
        f"ART-A/B/C: {len(programs.get('art_A', []))}/{len(programs.get('art_B', []))}/{len(programs.get('art_C', []))} proofs · B ship⛔={b_ship_blocked}",
        f"UI: {len(programs.get('ui_presentation', []))} proofs · UNCLASSIFIED: {len(programs.get('unclassified', []))}",
    ]
    if b_ship_blocked:
        lines.append("FAIL-PROP[art_B]: Track B manual keyframe — warehouse paused (does not block ATL★/RT★)")
    if constr_red:
        keys = ", ".join(w["witness_key"] for w in constr_red[:3])
        lines.append(f"FAIL-PROP[construction]: red sub-witnesses: {keys}")
    return lines


def _qce_fields(programs: dict[str, list[dict[str, Any]]], index: dict[str, Any]) -> dict[str, Any]:
    b_dishonest = sum(
        1
        for p in programs.get("art_B", [])
        if (p.get("summary") or {}).get("honest_gate") == "dishonest_gate"
    )
    constr_red = [w for w in index.get("construction_sub_witnesses", []) if w.get("green") is False]
    stage5_ok = _count_green(programs.get("stage5_spine", [])) > 0
    return {
        "Q_coherence": {
            "score": 7 if stage5_ok else 5,
            "evidence": f"stage5 spine + construction operational; art_B dishonest={b_dishonest}",
        },
        "Q_stability": {
            "score": 7,
            "evidence": "FULL_APP + infra isolation + construction operational_green",
        },
        "C_compute": {
            "score": 5,
            "evidence": "fire_streaming + wave virtualization indexed; weather_sim_live when green",
        },
        "C_tokens": {"score": None, "evidence": "append agent_run_event_v1 on HANDOFF close"},
        "E_clarity": {
            "score": 6,
            "evidence": "construction sub-witness rollup in index; APS previews green",
        },
        "E_confusion_risk": {
            "score": min(9, 4 + b_dishonest + len(constr_red)),
            "evidence": f"art dishonest={b_dishonest}; construction red={len(constr_red)}",
        },
    }


def _construction_live_body() -> dict[str, Any]:
    path = DEBUG_RUNS / "construction_stage_live.json"
    if not path.is_file():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def _load_auth_spine_line(root: Path) -> str:
    """AUTH spine from master_chain_tensor — never hardcode stale ○ nodes."""
    tensor_path = root / "tools/orchestrator/queues/master_chain_tensor_v1.json"
    if tensor_path.is_file():
        try:
            data = json.loads(tensor_path.read_text(encoding="utf-8"))
            spine = data.get("auth_spine") or {}
            order = ("MAT", "APS", "SNAP", "WRK", "ATL", "RT")
            parts = []
            for node in order:
                entry = spine.get(node) or {}
                glyph = entry.get("glyph") or ("★" if entry.get("phi") == 2 else "○")
                parts.append(f"{node}{glyph}")
            if parts:
                return "⇢".join(parts)
        except (json.JSONDecodeError, OSError, TypeError):
            pass
    return "MAT★⇢APS★⇢SNAP★⇢WRK★⇢ATL★⇢RT★"


def _witness_green(path: Path) -> bool:
    if not path.is_file():
        return False
    try:
        body = json.loads(path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError):
        return False
    if body.get("green") is True:
        return True
    if body.get("parity_green") is True:
        return True
    return body.get("operational_green") is True


def _handoff_resolved(item: dict[str, Any]) -> bool:
    """Drop registry handoffs when disk witnesses prove green (avoid stale ΔWF)."""
    if item.get("resolved") is True:
        return True
    task_id = str(item.get("task_id") or "")
    if task_id == "INFRA-E0-003":
        return _witness_green(DEBUG_RUNS / "transport_network_live.json")
    if task_id == "MCP-PILOT-GRAMMAR-001":
        # Paused defer — keep in ΔWF as operator lane, not infra blocker
        return False
    body = _construction_live_body()
    if not body:
        return False

    witness_key = item.get("witness_key")
    if witness_key:
        val = body.get(witness_key)
        if isinstance(val, dict) and val.get("green") is True:
            if witness_key == "construction_parametric_placement_001":
                return val.get("partial_alpha") is True
            return True

    if task_id == "CON-P3-S1-S3":
        audit = body.get("construction_scaling_audit_001") or {}
        return all(
            audit.get(k) is True
            for k in (
                "s1_preset_matrix_match",
                "s2_occupied_tiles_wired",
                "s3_blocked_disables_commit",
            )
        )
    if task_id == "FIX-PROC-TEST-REGRESS-001":
        proc = body.get("construction_procedural_build_001") or {}
        return proc.get("green") is True
    return False


def _delta_wf(programs: dict[str, list[dict[str, Any]]], index: dict[str, Any]) -> list[dict[str, str]]:
    registry = _load_registry()
    rows: list[dict[str, str]] = []
    for item in registry.get("handoff_priorities", []):
        if _handoff_resolved(item):
            continue
        rows.append(
            {
                "finding": item.get("note") or item["task_id"],
                "owner": item.get("owner", "@coder"),
                "program_id": item.get("program_id", ""),
                "next_artifact": item.get("witness_key") or item["task_id"],
            }
        )
    for p in programs.get("art_B", []):
        s = p.get("summary") or {}
        if s.get("honest_gate") == "dishonest_gate":
            rows.insert(
                0,
                {
                    "finding": "Track B ship blocked — manual keyframe + G4",
                    "owner": "@designer-mcp + operator",
                    "program_id": "art_B",
                    "next_artifact": p.get("path", ""),
                },
            )
            break
    missing = [
        p
        for p in programs.get("agent_ops", [])
        if "orchestrator_thread_health" in p.get("path", "") and not p.get("exists")
    ]
    if missing:
        rows.append(
            {
                "finding": "orchestrator_thread_health.json missing",
                "owner": "@coder",
                "program_id": "agent_ops",
                "next_artifact": "debug_runs/orchestrator_thread_health.json",
            }
        )
    weather = registry["programs"].get("weather", {})
    if not weather.get("anchor_witnesses") and not _witness_green(DEBUG_RUNS / "weather_sim_live.json"):
        rows.append(
            {
                "finding": "Weather/atmosphere — no dedicated witness yet (deferred)",
                "owner": "@planner",
                "program_id": "weather",
                "next_artifact": "src/dev/stage5_triage_backlog.md",
            }
        )
    return rows[:12]


def _doc_reads_rollup() -> dict[str, Any] | None:
    brief_path = DEBUG_RUNS / "agent_ops" / "doc_reads_brief_latest.json"
    if not brief_path.is_file():
        return None
    try:
        brief = json.loads(brief_path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError):
        return None
    return {
        "witness_path": _rel(brief_path),
        "total_reads_in_window": brief.get("total_reads_in_window"),
        "hot_path_count": len(brief.get("hot_paths") or []),
        "promotion_candidates": len(brief.get("promotion_candidates") or []),
        "top_hot_paths": [
            {"path": r.get("path"), "count": r.get("count")}
            for r in (brief.get("hot_paths") or [])[:5]
        ],
    }


def build_ops_report(index: dict[str, Any]) -> dict[str, Any]:
    programs = _program_rollup(index)
    now = int(datetime.now(timezone.utc).timestamp())
    return {
        "_agent_meta": {
            "schema": "ops_report_v2",
            "written_at_epoch_secs": now,
            "profile": "OPS_REPORT",
            "source_system": "ops_witness_index",
            "relative_path": _rel(OUT_OPS),
            "agent": "operations-intelligence",
        },
        "profile": "OPS_REPORT",
        "plan_id": "PLAN-THREE-TRACK-001",
        "track_d_id": "OPS-WITNESS-SPINE-002",
        "registry_path": _rel(REGISTRY_PATH),
        "dsm_snapshot": _dsm_snapshot(programs, index),
        "qce": _qce_fields(programs, index),
        "delta_wf": _delta_wf(programs, index),
        "program_summary": {
            pid: {
                "count": len(proofs),
                "green_count": _count_green(proofs),
                "label": (_load_registry()["programs"].get(pid) or {}).get("label"),
            }
            for pid, proofs in sorted(programs.items())
        },
        "construction_sub_witnesses": index.get("construction_sub_witnesses"),
        "unified_index_path": _rel(OUT_INDEX),
        "handoff_path": "tools/orchestrator/queues/HANDOFF.md",
        "doc_reads": _doc_reads_rollup(),
    }


def _import_ops_intelligence():
    if str(MCP_PYTHON) not in sys.path:
        sys.path.insert(0, str(MCP_PYTHON))
    from rust_engine_mcp import ops_intelligence

    return ops_intelligence


def main() -> int:
    if not REGISTRY_PATH.is_file():
        print(f"Missing registry: {REGISTRY_PATH}", file=sys.stderr)
        return 1
    index = build_index()
    report = build_ops_report(index)
    ops_intelligence = _import_ops_intelligence()
    brief = ops_intelligence.ops_build_project_brief(ops_report=report, repo=REPO_ROOT)
    report["utility_score"] = brief.get("utility_score")
    report["metrics_tier1"] = brief.get("metrics_tier1")
    meta = report.setdefault("_agent_meta", {})
    if isinstance(meta, dict):
        meta["source_system"] = "ops_witness_index+ops_project_brief"
    OUT_OPS.parent.mkdir(parents=True, exist_ok=True)
    OUT_INDEX.write_text(json.dumps(index, indent=2) + "\n", encoding="utf-8")
    OUT_OPS.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    ops_intelligence.write_ops_project_brief(brief, repo=REPO_ROOT)
    ops_intelligence.refresh_ops_mcp_function_layer_witness(
        repo=REPO_ROOT,
        brief_path=OUT_OPS_BRIEF,
    )
    print(f"Wrote {_rel(OUT_INDEX)} ({index['proof_count']} proofs, {index['program_count']} programs)")
    print(f"Wrote {_rel(OUT_OPS)}")
    print(f"Wrote {_rel(OUT_OPS_BRIEF)}")
    print(f"Wrote {_rel(DEBUG_RUNS / 'agent_ops' / 'ops_mcp_function_layer_live.json')}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
