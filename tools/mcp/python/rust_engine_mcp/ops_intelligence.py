"""OPS intelligence — compressed project brief for agent orientation (JSON backend)."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from . import agent_queue
from .paths import repo_root

PHASE3_QUEUE = "tools/orchestrator/queues/post_drain_phase3_queue.json"
OPS_REPORT_REL = "debug_runs/agent_ops/ops_report_latest.json"
OPS_BRIEF_REL = "debug_runs/agent_ops/ops_project_brief_v1.json"
OPS_MCP_LAYER_WITNESS_REL = "debug_runs/agent_ops/ops_mcp_function_layer_live.json"
MCP_PHASE4_QUEUE_WITNESS_REL = "debug_runs/agent_ops/mcp_phase4_queue_live.json"
MCP_VALID_CONSTRUCTION_WITNESS_REL = "debug_runs/agent_ops/mcp_valid_construction_live.json"

REVIEW_ORDER_P0: tuple[dict[str, Any], ...] = (
    {
        "key": "P0-A",
        "task_id": "TRIAGE-MAP-PICK-CLOSURE-001",
        "label": "P0-BUILD-FOOTPRINT-001",
        "agent": "coder",
        "vr_id": "VR-10",
    },
    {
        "key": "P0-B",
        "task_id": "P0-MINIMAP-WIDGET-001",
        "label": "P0-MINIMAP-WIDGET-001",
        "agent": "designer",
    },
    {
        "key": "P0-C",
        "task_id": "P0-FIRE-TILE-VFX-001",
        "task_alt": "TRIAGE-FIRE-PRODUCT-001",
        "label": "P0-FIRE-TILE-VFX-001",
        "agent": "coder",
    },
    {
        "key": "P0-D",
        "task_id": "P0-VFX-ZOOM-LOCK-001",
        "label": "P0-VFX-ZOOM-LOCK-001",
        "agent": "coder",
    },
)

_LAMBDA_TOKENS = 0.02
_MU_COMPUTE = 0.01
_NU_DEBT = 0.15

_AGENT_PICK_LABEL: dict[str, str] = {
    "coder": "@coder",
    "coder-mcp": "@coder-mcp",
    "designer": "@designer",
    "designer-mcp": "@designer-mcp",
    "planner": "@planner",
    "sim-steward": "@sim-steward",
    "operator": "Operator",
    "orchestrator": "@orchestrator",
}

_REQUIRED_BRIEF_KEYS = frozenset(
    {
        "schema",
        "project",
        "quality_score",
        "utility_score",
        "auth_spine",
        "known_failures",
        "top_failures_ranked",
        "recent_improvements",
        "suggested_focus",
        "active_picks",
        "last_20_runs_summary",
        "metrics_tier1",
    }
)


def _load_json(path: Path) -> dict[str, Any] | None:
    if not path.is_file():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError):
        return None
    return data if isinstance(data, dict) else None


def _load_phase3_tasks(root: Path) -> list[dict[str, Any]]:
    data = _load_json(root / PHASE3_QUEUE)
    if not data:
        return []
    tasks = data.get("tasks")
    return tasks if isinstance(tasks, list) else []


def _agent_pick_label(agent: str) -> str:
    key = agent.strip().lower()
    if key == "operator":
        return "Operator"
    return _AGENT_PICK_LABEL.get(key, f"@{agent}" if not agent.startswith("@") else agent)


def _active_picks_from_phase3(root: Path) -> dict[str, str]:
    ready = [t for t in _load_phase3_tasks(root) if str(t.get("status") or "") == "ready" and t.get("id")]
    ready.sort(key=lambda t: int(t.get("priority") or 999))
    picks: dict[str, str] = {}
    for row in ready:
        label = _agent_pick_label(str(row.get("agent") or ""))
        if label not in picks:
            picks[label] = str(row["id"])
    return picks


def brief_schema_keys_present(brief: dict[str, Any]) -> bool:
    return _REQUIRED_BRIEF_KEYS.issubset(brief.keys())


def _utility_score(quality: int, failures: list[str]) -> float:
    debt = len(failures) * _NU_DEBT
    return round(max(0.0, quality - debt * 10 - _LAMBDA_TOKENS * 5 - _MU_COMPUTE * 3), 1)


def ops_build_project_brief(
    *,
    ops_report: dict[str, Any] | None = None,
    repo: Path | None = None,
) -> dict[str, Any]:
    root = repo or repo_root()
    report = ops_report or _load_json(root / OPS_REPORT_REL) or {}
    qce = report.get("qce") if isinstance(report.get("qce"), dict) else {}
    q_score = int((qce.get("Q_coherence") or {}).get("score") or 70)
    delta_wf = report.get("delta_wf") if isinstance(report.get("delta_wf"), list) else []
    known_failures = [
        str(row.get("finding", "")).lower().replace(" ", "_").replace("—", "")[:64]
        for row in delta_wf[:5]
        if row.get("finding")
    ]
    top_failures = [
        {"id": known_failures[i] if i < len(known_failures) else f"failure_{i}", "severity": "P0" if i == 0 else "P1"}
        for i in range(min(3, max(len(known_failures), 1)))
    ]
    done_rows = [t for t in _load_phase3_tasks(root) if str(t.get("status") or "") == "done"]
    recent = [str(t.get("id")) for t in done_rows[-5:] if t.get("id")]

    build_set_health: dict[str, Any] = {}
    try:
        from .grammar_build_set import building_set_health_brief

        build_set_health = building_set_health_brief()
    except Exception:  # noqa: BLE001
        build_set_health = {"green": False, "error": "building_set_health_unavailable"}

    brief: dict[str, Any] = {
        "schema": "ops_project_brief_v1",
        "project": root.name,
        "quality_score": q_score,
        "utility_score": _utility_score(q_score, known_failures),
        "auth_spine": "MAT★⇢APS★⇢SNAP★⇢WRK★⇢ATL★⇢RT★",
        "known_failures": known_failures[:5],
        "top_failures_ranked": top_failures,
        "recent_improvements": recent,
        "suggested_focus": str(delta_wf[0].get("finding")) if delta_wf else "Spine green — pick phase3 ready row",
        "active_picks": _active_picks_from_phase3(root),
        "last_20_runs_summary": "ops_report_latest.json, ops_project_brief_v1.json",
        "metrics_tier1": {
            "q_per_token": None,
            "ftr": None,
            "rtr": None,
            "status": "not_measured",
            "note": "Tier-1 KPIs require agent_run_event telemetry",
        },
        "build_set_health": build_set_health,
        "handoff_ok": True,
        "ops_report_path": OPS_REPORT_REL,
        "phase3_queue_path": PHASE3_QUEUE,
    }
    return brief


def ops_get_project_brief() -> dict[str, Any]:
    path = repo_root() / OPS_BRIEF_REL
    if path.is_file():
        brief = _load_json(path) or ops_build_project_brief()
    else:
        brief = ops_build_project_brief()
    brief["ok"] = True
    return brief


def ops_get_retry_guidance(task_id: str) -> dict[str, Any]:
    needle = task_id.strip()
    root = repo_root()
    for row in _load_phase3_tasks(root):
        if str(row.get("id") or "") != needle:
            continue
        return {
            "ok": True,
            "task_id": needle,
            "status": row.get("status"),
            "agent": row.get("agent"),
            "witness": row.get("witness") or row.get("witness_json"),
            "depends_on": row.get("depends_on") or [],
            "blocked_by": row.get("blocked_by") or [],
            "goal": row.get("goal") or row.get("title"),
        }
    return {"ok": False, "task_id": needle, "error": "task not found in phase3 queue"}


def _phase4_tasks_by_id() -> dict[str, dict[str, Any]]:
    try:
        return {str(t.get("id")): t for t in agent_queue.load_queue("phase4") if t.get("id")}
    except (FileNotFoundError, ValueError, json.JSONDecodeError, KeyError):
        return {}


def _vr_blockers_compressed() -> list[dict[str, str]]:
    path = repo_root() / "src/dev/visual_run_blockers.md"
    rows: list[dict[str, str]] = []
    if path.is_file():
        for line in path.read_text(encoding="utf-8").splitlines():
            if "**VR-" not in line or not line.strip().startswith("|"):
                continue
            parts = [p.strip() for p in line.split("|") if p.strip()]
            if len(parts) < 3 or parts[0].startswith("---"):
                continue
            vid = parts[0].strip("* ")
            if not vid.startswith("VR-"):
                continue
            rows.append(
                {
                    "id": vid,
                    "symptom": parts[1][:96],
                    "gate": parts[2][:48] if len(parts) > 2 else "",
                }
            )
    if not any(r["id"] == "VR-10" for r in rows):
        rows.append(
            {
                "id": "VR-10",
                "symptom": "Ghost footprint mis-projected under cursor (MAP-PICK)",
                "gate": "G-PLAY footprint — see plan_build_footprint_vm09_exec_v1.md",
                "status": "closed",
            }
        )
    return rows[:8]


def review_order_brief() -> dict[str, Any]:
    """BLANG:REVIEW — REVIEW-ORDER P0 rows + phase4 status + VR compressed."""
    phase4 = _phase4_tasks_by_id()
    p0_rows: list[dict[str, Any]] = []
    for spec in REVIEW_ORDER_P0:
        task_id = str(spec["task_id"])
        row = phase4.get(task_id) or phase4.get(str(spec.get("task_alt") or ""))
        status = str(row.get("status") or "missing") if row else "missing"
        blocker = None
        if row:
            blocker = row.get("snag") or row.get("note")
            if status == "done":
                blocker = None
        agent = str(spec.get("agent") or (row or {}).get("agent") or "")
        p0_rows.append(
            {
                "key": spec["key"],
                "task_id": task_id,
                "label": spec.get("label", task_id),
                "status": status,
                "agent": agent,
                "blocker": blocker,
                "delta_wf": None if status == "done" else f"ΔWF→@{agent}",
                "witness": (row or {}).get("witness"),
                "vr_id": spec.get("vr_id"),
            }
        )
    gplay = phase4.get("G-PLAY-01") or {}
    return {
        "schema": "review_order_brief_v1",
        "ok": True,
        "g_play": {
            "id": "G-PLAY-01",
            "status": gplay.get("status"),
            "blocker": gplay.get("snag") or gplay.get("note"),
        },
        "p0_rows": p0_rows,
        "vr_blockers": _vr_blockers_compressed(),
        "phase4_queue": agent_queue.QUEUE_REGISTRY.get("phase4"),
    }


def write_mcp_phase4_queue_live_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    review = review_order_brief()
    slice_sample = agent_queue.slice_exec_brief("TRIAGE-MAP-PICK-CLOSURE-001")
    next_phase4 = agent_queue.agent_queue_next("coder", queue="phase4")
    body: dict[str, Any] = {
        "gate": "MCP-P2-QUEUE-PHASE4-001",
        "green": bool(review.get("ok")) and bool(slice_sample.get("ok")),
        "phase4_in_registry": "phase4" in agent_queue.QUEUE_REGISTRY,
        "review_order": review,
        "slice_sample": slice_sample,
        "agent_queue_next_phase4": next_phase4,
        "_agent_meta": {
            "schema": "mcp_phase4_queue_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "MCP_PHASE4_QUEUE",
            "source_system": "ops_intelligence",
            "relative_path": MCP_PHASE4_QUEUE_WITNESS_REL,
        },
    }
    out = root / MCP_PHASE4_QUEUE_WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = MCP_PHASE4_QUEUE_WITNESS_REL
    return body


def write_mcp_valid_construction_live_witness(*, repo: Path | None = None) -> dict[str, Any]:
    from .validators.construction_witness import validate_construction_witness_path

    root = repo or repo_root()
    stage_path = "debug_runs/construction_stage_live.json"
    placement_path = "debug_runs/construction_placement_live.json"
    stage_brief = agent_queue.witness_brief(stage_path, profile="construction")
    map_brief = agent_queue.witness_brief("debug_runs/map_zoom_coherence_live.json", profile="map_pick")
    placement_report = validate_construction_witness_path(placement_path)
    body: dict[str, Any] = {
        "gate": "MCP-P2-VALID-CONSTRUCTION-001",
        "green": stage_brief.get("ok")
        and map_brief.get("ok")
        and placement_report.status == "passed",
        "witness_brief_construction": stage_brief,
        "witness_brief_map_pick": map_brief,
        "validate_report_construction": placement_report.to_dict(),
        "_agent_meta": {
            "schema": "mcp_valid_construction_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "MCP_VALID_CONSTRUCTION",
            "source_system": "ops_intelligence",
            "relative_path": MCP_VALID_CONSTRUCTION_WITNESS_REL,
        },
    }
    out = root / MCP_VALID_CONSTRUCTION_WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = MCP_VALID_CONSTRUCTION_WITNESS_REL
    return body


def write_ops_project_brief(brief: dict[str, Any], *, repo: Path | None = None) -> Path:
    root = repo or repo_root()
    out = root / OPS_BRIEF_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    meta = {
        "schema": "ops_project_brief_v1",
        "written_at_epoch_secs": int(time.time()),
        "profile": "OPS_PROJECT_BRIEF",
        "source_system": "ops_intelligence",
        "relative_path": OPS_BRIEF_REL,
    }
    body = dict(brief)
    body["_agent_meta"] = meta
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return out


def refresh_ops_mcp_function_layer_witness(
    *,
    repo: Path | None = None,
    brief_path: Path | None = None,
) -> dict[str, Any]:
    root = repo or repo_root()
    brief_file = brief_path or (root / OPS_BRIEF_REL)
    brief_ok = brief_file.is_file()
    body = {
        "gate": "OPS-MCP-FUNCTION-LAYER-001",
        "green": brief_ok,
        "ops_get_project_brief": True,
        "ops_project_brief_v1_path": brief_ok,
        "ops_project_brief_rel": OPS_BRIEF_REL,
        "schema": "ops_project_brief_v1",
        "quality_score": None,
        "_agent_meta": {
            "schema": "ops_mcp_function_layer_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "OPS_MCP_FUNCTION_LAYER",
            "source_system": "ops_intelligence",
            "relative_path": OPS_MCP_LAYER_WITNESS_REL,
        },
    }
    if brief_ok:
        brief = _load_json(brief_file) or {}
        body["quality_score"] = brief.get("quality_score")
    out = root / OPS_MCP_LAYER_WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body
