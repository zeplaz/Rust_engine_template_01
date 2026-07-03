"""Agent continuation queues — drain-ready next slice without wait-only turns."""

from __future__ import annotations

import json
import re
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from .paths import repo_root

QUEUE_REGISTRY: dict[str, str] = {
    "multi_parallel": "tools/orchestrator/queues/multi_parallel_home_queues_v1.json",
    "grammar": "tools/orchestrator/queues/grammar_continuation_queue.json",
    "aps_grammar": "tools/orchestrator/queues/aps_grammar_evolution_queue.json",
    "continuation": "tools/orchestrator/queues/continuation_queue.json",
    "simulation": "tools/orchestrator/queues/simulation_continuation_queue.json",
    "phase4": "tools/orchestrator/queues/post_drain_phase4_queue.json",
    "power_ux": "tools/orchestrator/queues/power_grid_construction_ux_queue.json",
    "aps_presence": "tools/orchestrator/queues/aps_presence_correction_queue.json",
}

MULTI_PARALLEL_DISPATCH = "tools/orchestrator/queues/multi_parallel_tracks_dispatch_v1.json"
DESIGNER_ACTIVE_QUEUE = "tools/orchestrator/queues/designer_active_queue.json"

PICK_STATUS = frozenset({"ready", "in_progress"})
WAIT_STATUS = frozenset({"blocked", "paused", "deferred"})

AGENT_ALIASES: dict[str, str] = {
    "planner-mcp": "planner",
    "coder-mcp": "coder-mcp",
    "designer-mcp": "designer-mcp",
    "orchestrator-mcp": "orchestrator-mcp",
    "@planner": "planner",
    "@coder": "coder",
    "@designer": "designer",
    "@coder-mcp": "coder-mcp",
    "@designer-mcp": "designer-mcp",
    "@operator": "operator",
    "@sim-steward": "sim-steward",
    "coder a": "coder_a",
    "coder b": "coder_b",
    "coder c": "coder_c",
    "coder_a": "coder_a",
    "coder_b": "coder_b",
    "coder_c": "coder_c",
    "@coder a": "coder_a",
    "@coder b": "coder_b",
    "@coder c": "coder_c",
    "operator": "operator",
    "sim-steward": "sim-steward",
    "orchestrator": "orchestrator",
}

WITNESS_PROFILES = frozenset({"map_pick", "construction", "fire_product", "honesty"})

VALID_STATUS = frozenset({"ready", "blocked", "in_progress", "done", "deferred", "cancelled", "reopened"})


def _normalize_agent(agent: str) -> str:
    key = agent.strip().lower()
    return AGENT_ALIASES.get(key, key)


def queue_path(queue: str) -> Path:
    rel = QUEUE_REGISTRY.get(queue)
    if not rel:
        raise KeyError(f"unknown queue: {queue!r}; known: {sorted(QUEUE_REGISTRY)}")
    return repo_root() / rel


def _read_queue_file(path: Path) -> tuple[list[dict[str, Any]], dict[str, Any] | None]:
    data = json.loads(path.read_text(encoding="utf-8"))
    if isinstance(data, list):
        return data, None
    if isinstance(data, dict):
        for k in ("drain", "tasks", "multi_parallel_ready", "active"):
            rows = data.get(k)
            if isinstance(rows, list) and rows and isinstance(rows[0], dict):
                return rows, data
            if isinstance(rows, list) and not rows:
                return rows, data
    raise ValueError(f"queue must be a JSON array or object with drain[]/tasks[]: {path}")


def _row_agent(row: dict[str, Any]) -> str:
    return _normalize_agent(str(row.get("owner") or row.get("agent") or ""))


def _priority_num(row: dict[str, Any]) -> int:
    pri = row.get("priority")
    if isinstance(pri, str) and pri.upper().startswith("P") and len(pri) > 1 and pri[1].isdigit():
        return int(pri[1])
    try:
        return int(pri)  # type: ignore[arg-type]
    except (TypeError, ValueError):
        return 9


def _pick_sort_key(row: dict[str, Any]) -> tuple[int, int, str]:
    wave = int(row.get("wave") or 0)
    return (wave, _priority_num(row), str(row.get("id") or ""))


def _load_designer_parallel_ready() -> list[dict[str, Any]]:
    path = repo_root() / DESIGNER_ACTIVE_QUEUE
    if not path.is_file():
        return []
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return []
    rows = data.get("multi_parallel_ready")
    if not isinstance(rows, list):
        return []
    return [r for r in rows if isinstance(r, dict) and r.get("id")]


def load_multi_parallel_items(*, include_designer_mirror: bool = True) -> list[dict[str, Any]]:
    """Merged work rows from multi_parallel home queue (+ designer mirror for specs)."""
    items = load_queue("multi_parallel")
    by_id = {str(r["id"]): r for r in items if r.get("id")}
    if include_designer_mirror:
        for row in _load_designer_parallel_ready():
            sid = str(row.get("id") or "")
            if not sid:
                continue
            if sid not in by_id:
                by_id[sid] = row
            elif str(by_id[sid].get("status")) in WAIT_STATUS and str(row.get("status")) == "ready":
                by_id[sid] = {**by_id[sid], **row}
    return list(by_id.values())


def _track_filter(row: dict[str, Any], track: str) -> bool:
    if not track:
        return True
    t = track.strip().upper()
    row_track = str(row.get("track") or row.get("track_id") or "").upper()
    if t in row_track:
        return True
    if t.startswith("T") and t[1:].isdigit():
        return row_track == t or row_track.endswith(t)
    return t in row_track


def _estimate_slice_minutes(row: dict[str, Any]) -> int:
    agent = _row_agent(row)
    sid = str(row.get("id") or "")
    if agent == "operator" or row.get("needs_display"):
        return 15
    if agent in ("designer", "designer-mcp"):
        return 20 if "DES-" in sid or "DMCP-" in sid else 25
    if agent == "coder-mcp":
        return 30 if row.get("territory") and "app.py" in str(row.get("territory")) else 25
    if agent in ("coder", "coder_a", "coder_b", "coder_c"):
        return 25
    if agent == "sim-steward":
        return 15
    return 20


def _demand_plan_minutes(row: dict[str, Any]) -> int:
    """Planning estimate for session lists — shorter than execution estimate to fit ~4–6 slices/hour."""
    return min(15, max(10, _estimate_slice_minutes(row) // 2))


def _compress_work_row(row: dict[str, Any]) -> dict[str, Any]:
    keep = (
        "id",
        "title",
        "goal",
        "priority",
        "owner",
        "agent",
        "track",
        "track_id",
        "wave",
        "status",
        "deliverable",
        "witness",
        "plan",
        "depends_on",
        "territory",
        "verify",
        "home_queue",
        "parallel_ok",
        "needs_display",
        "note",
    )
    out = {k: row[k] for k in keep if k in row and row[k] not in (None, "", [])}
    if "goal" in row and "title" not in out:
        out["title"] = str(row.get("goal") or "")[:120]
    return out


def _tracks_summary(items: list[dict[str, Any]], agent: str) -> dict[str, int]:
    counts: dict[str, int] = {}
    for row in items:
        if _row_agent(row) != agent:
            continue
        if str(row.get("status")) not in PICK_STATUS:
            continue
        key = str(row.get("track") or row.get("track_id") or "?")
        counts[key] = counts.get(key, 0) + 1
    return counts


def agent_queue_demand(
    agent: str,
    *,
    minutes: int = 60,
    max_slices: int = 8,
    track_rotate: bool = True,
    track: str = "",
) -> dict[str, Any]:
    """Build an hour-scale session todo list from ready rows (cross-track)."""
    norm = _normalize_agent(agent)
    items = load_multi_parallel_items()
    ready = [
        r
        for r in items
        if _row_agent(r) == norm
        and str(r.get("status")) == "ready"
        and _deps_satisfied(r, _by_id(items))
        and _track_filter(r, track)
    ]
    ready.sort(key=_pick_sort_key)

    if not ready:
        return {
            "schema": "agent_queue_demand_v1",
            "ok": True,
            "agent": norm,
            "minutes_budget": minutes,
            "action": "idle",
            "demand_todos": [],
            "hint": "No ready rows — try get-que with cross_drain or another agent lane",
        }

    plan: list[dict[str, Any]] = []
    used_tracks: set[str] = set()
    budget = max(15, minutes)
    spent = 0
    pool = list(ready)

    while pool and len(plan) < max_slices and spent < budget:
        pick_idx = 0
        if track_rotate and len(pool) > 1:
            for i, row in enumerate(pool):
                tr = str(row.get("track") or row.get("track_id") or "")
                if tr and tr not in used_tracks:
                    pick_idx = i
                    break
        row = pool.pop(pick_idx)
        est = _demand_plan_minutes(row)
        if spent + est > budget and plan:
            break
        tr = str(row.get("track") or row.get("track_id") or "")
        used_tracks.add(tr)
        spent += est
        plan.append(
            {
                "n": len(plan) + 1,
                "id": row.get("id"),
                "track": tr,
                "wave": row.get("wave"),
                "est_minutes": est,
                "title": (row.get("goal") or row.get("title") or "")[:80],
                "deliverable": row.get("deliverable"),
                "witness": row.get("witness"),
            }
        )

    return {
        "schema": "agent_queue_demand_v1",
        "ok": True,
        "agent": norm,
        "minutes_budget": minutes,
        "minutes_estimated": spent,
        "action": "work",
        "demand_todos": plan,
        "session_loop": [
            "For each todo n: slice_exec_brief(id) → work → WIT-HON → dual Q✓",
            "Blocked mid-slice? get-que again — cross-drain to next todo",
            f"Regression after every 2 coder slices",
        ],
        "hint": f"Demand plan: {len(plan)} slices · ~{spent}m — say 'get que' between slices",
    }


def agent_get_que(
    agent: str,
    *,
    track: str = "",
    build_list: bool = False,
    minutes: int = 60,
    mark_in_progress: bool = False,
) -> dict[str, Any]:
    """
    BLANG:Q+ for multi-parallel tracks — 'get que' entry point.
    Returns next slice + ready board + optional hour-scale demand plan.
    """
    norm = _normalize_agent(agent)
    items = load_multi_parallel_items()
    by_id = _by_id(items)
    work, blocked_primary, reason = _pick_next_multi(items, norm, track=track)

    if work and mark_in_progress and str(work.get("status")) == "ready":
        _mark_slice_in_progress(str(work["id"]))

    mine_ready = [
        r
        for r in items
        if _row_agent(r) == norm
        and str(r.get("status")) == "ready"
        and _deps_satisfied(r, by_id)
        and _track_filter(r, track)
    ]
    mine_ready.sort(key=_pick_sort_key)

    cross_drain = [r for r in mine_ready if work and r.get("id") != work.get("id")][:8]

    out: dict[str, Any] = {
        "schema": "agent_get_que_v1",
        "ok": True,
        "agent": norm,
        "queue": "multi_parallel",
        "action": "work" if work else "idle",
        "drain_reason": reason,
        "tracks_open": _tracks_summary(items, norm),
        "ready_count": len(mine_ready),
        "next": _compress_work_row(work) if work else None,
        "drain_todos": [_compress_work_row(r) for r in mine_ready[:12]],
        "cross_drain": [_compress_work_row(r) for r in cross_drain],
        "board_counts_agent": _status_counts(items, norm),
        "queue_path": QUEUE_REGISTRY["multi_parallel"],
        "dispatch_path": MULTI_PARALLEL_DISPATCH,
        "plan_doc": "src/dev/plan_multi_parallel_tracks_v1.md",
        "prompts_doc": "src/dev/multi_parallel_agent_prompts_v1.md",
        "session_loop": [
            "BLANG:PRE → get-que <agent> → slice_exec_brief(next.id)",
            "work → validate-report witness_honesty → agent-queue-update --enforce (dual Q✓)",
            "blocked? get-que again (cross_drain picks another track)",
            "hour session? get-que <agent> --demand --minutes 60",
        ],
        "aliases": ["get que", "get-que", "agent-get-que", "agent_queue_next --queue multi_parallel"],
    }
    if blocked_primary:
        out["blocked_primary"] = {
            "id": blocked_primary.get("id"),
            "status": blocked_primary.get("status"),
            "depends_on": blocked_primary.get("depends_on"),
        }
    if build_list:
        out["demand"] = agent_queue_demand(
            norm, minutes=minutes, track=track
        )
    if work:
        brief = slice_exec_brief(str(work["id"]), queue="multi_parallel")
        if brief.get("ok"):
            out["slice_brief"] = {
                "files": brief.get("files"),
                "witness": brief.get("witness"),
                "exit_predicate": brief.get("exit_predicate"),
            }
    return out


def _pick_next_multi(
    items: list[dict[str, Any]],
    agent: str,
    *,
    track: str = "",
) -> tuple[dict[str, Any] | None, dict[str, Any] | None, str]:
    by_id = _by_id(items)
    mine = [r for r in items if _row_agent(r) == agent and _track_filter(r, track)]
    mine.sort(key=_pick_sort_key)

    blocked_primary: dict[str, Any] | None = None
    for row in mine:
        st = str(row.get("status") or "")
        if st not in PICK_STATUS:
            continue
        if st == "ready" and _deps_satisfied(row, by_id):
            return row, None, "next_ready"
        if st in ("ready", "blocked", "in_progress") and not _deps_satisfied(row, by_id):
            blocked_primary = row
            break

    if blocked_primary:
        for row in mine:
            if row.get("id") == blocked_primary.get("id"):
                continue
            if str(row.get("status")) != "ready":
                continue
            if not _deps_satisfied(row, by_id):
                continue
            return row, blocked_primary, f"cross_drain:{blocked_primary.get('id')}"

    for row in mine:
        if str(row.get("status")) == "ready" and _deps_satisfied(row, by_id):
            return row, None, "next_ready"

    return None, blocked_primary, "lane_idle"


def _mark_slice_in_progress(slice_id: str) -> None:
    items = load_queue("multi_parallel")
    for row in items:
        if str(row.get("id")) == slice_id and str(row.get("status")) == "ready":
            row["status"] = "in_progress"
            row["started_at"] = datetime.now(timezone.utc).isoformat()
            break
    save_queue("multi_parallel", items)


def _save_multi_parallel_drain(items: list[dict[str, Any]]) -> Path:
    return save_queue("multi_parallel", items)


def resolve_agent_queue(agent: str, queue: str) -> str:
    """Default auto → multi_parallel home queue; legacy grammar/phase4 when explicitly requested."""
    if queue != "auto":
        return queue
    norm = _normalize_agent(agent)
    productive = {
        "planner",
        "coder",
        "coder-mcp",
        "designer",
        "designer-mcp",
        "coder_a",
        "coder_b",
        "coder_c",
        "operator",
        "sim-steward",
        "orchestrator-mcp",
    }
    if norm in productive:
        return "multi_parallel"
    if norm == "coder":
        try:
            tasks = {str(t.get("id")): t for t in load_queue("phase4")}
            gplay = tasks.get("G-PLAY-01")
            if gplay and str(gplay.get("status") or "") in ("blocked", "ready", "in_progress"):
                return "phase4"
        except (KeyError, FileNotFoundError, ValueError, json.JSONDecodeError):
            pass
    return "grammar"


def load_queue(queue: str) -> list[dict[str, Any]]:
    path = queue_path(queue)
    if not path.is_file():
        raise FileNotFoundError(f"queue file missing: {path}")
    items, _ = _read_queue_file(path)
    return items


def save_queue(queue: str, items: list[dict[str, Any]]) -> Path:
    path = queue_path(queue) if queue != "designer_active" else repo_root() / DESIGNER_ACTIVE_QUEUE
    path.parent.mkdir(parents=True, exist_ok=True)
    if queue in ("phase4", "multi_parallel", "aps_presence", "aps_grammar") and path.is_file():
        try:
            raw = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            raw = {}
        if isinstance(raw, dict):
            key = "drain" if queue in ("multi_parallel", "aps_presence", "aps_grammar") else "tasks"
            raw[key] = items
            path.write_text(json.dumps(raw, indent=2) + "\n", encoding="utf-8")
            return path
    if queue == "designer_active" and path.is_file():
        raw = json.loads(path.read_text(encoding="utf-8"))
        if isinstance(raw, dict):
            raw["multi_parallel_ready"] = items
            path.write_text(json.dumps(raw, indent=2) + "\n", encoding="utf-8")
            return path
    path.write_text(json.dumps(items, indent=2) + "\n", encoding="utf-8")
    return path


def _sync_dispatch_row(slice_id: str, status: str, note: str) -> dict[str, Any] | None:
    """Dual Q✓ — mirror status into multi_parallel dispatch rollup when row exists."""
    path = repo_root() / MULTI_PARALLEL_DISPATCH
    if not path.is_file():
        return None
    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return None
    drain = raw.get("drain")
    if not isinstance(drain, list):
        return None
    synced = False
    for row in drain:
        if str(row.get("id")) != slice_id:
            continue
        row["status"] = status
        if note:
            row["note"] = note
        row["updated_at"] = datetime.now(timezone.utc).isoformat()
        synced = True
        break
    if synced:
        raw["drain"] = drain
        path.write_text(json.dumps(raw, indent=2) + "\n", encoding="utf-8")
        return {"dispatch_synced": True, "path": MULTI_PARALLEL_DISPATCH}
    return None


def find_queue_task(slice_id: str, *, queue: str | None = None) -> tuple[str, dict[str, Any]]:
    needle = slice_id.strip()
    order = [queue] if queue else ["multi_parallel", *QUEUE_REGISTRY.keys()]
    seen: set[str] = set()
    for q in order:
        if not q or q in seen:
            continue
        seen.add(q)
        try:
            for row in load_queue(q):
                if str(row.get("id") or "") == needle:
                    return q, row
        except (FileNotFoundError, ValueError, json.JSONDecodeError, KeyError):
            continue
    for row in _load_designer_parallel_ready():
        if str(row.get("id") or "") == needle:
            return "designer_active", row
    raise KeyError(f"slice_id not in queues: {needle}")


def slice_exec_brief(slice_id: str, *, queue: str | None = None) -> dict[str, Any]:
    """BLANG:SLICE — one queue row: exit, witness, exec docs, do_not_pick."""
    try:
        q, row = find_queue_task(slice_id, queue=queue)
    except KeyError as exc:
        return {"ok": False, "schema": "slice_exec_brief_v1", "id": slice_id, "error": str(exc)}

    docs = [str(d) for d in (row.get("docs") or []) if d]
    deliverable = str(row.get("deliverable") or "")
    if deliverable and deliverable not in docs:
        docs.insert(0, deliverable)
    files = [d for d in docs if "/" in d or d.endswith((".md", ".rs", ".json"))]
    status = str(row.get("status") or "ready")
    agent = str(row.get("owner") or row.get("agent") or "")
    witness_rel = str(row.get("witness") or row.get("witness_json") or "")
    return {
        "ok": True,
        "schema": "slice_exec_brief_v1",
        "queue": q,
        "id": slice_id,
        "title": str(row.get("goal") or row.get("title") or ""),
        "status": status,
        "agent": agent,
        "lane": row.get("lane"),
        "exit": str(row.get("exit") or ""),
        "witness": witness_rel,
        "exit_predicate": row.get("exit_predicate") if isinstance(row.get("exit_predicate"), dict) else None,
        "witness_honesty": _slice_witness_honesty_brief(row, witness_rel),
        "files": files,
        "exec_docs": docs,
        "do_not_pick": status in ("done", "cancelled"),
        "blocked_by": list(row.get("blocked_by") or row.get("depends_on") or []),
        "delta_wf": None if status == "done" else f"ΔWF→@{agent}",
        "also_known_as": row.get("also_known_as"),
    }


def _by_id(items: list[dict[str, Any]]) -> dict[str, dict[str, Any]]:
    return {str(x["id"]): x for x in items if x.get("id")}


def _deps_satisfied(item: dict[str, Any], by_id: dict[str, dict[str, Any]]) -> bool:
    for dep in item.get("depends_on") or []:
        row = by_id.get(str(dep))
        if row is None:
            continue
        if str(row.get("status")) != "done":
            return False
    return True


def _status_counts(items: list[dict[str, Any]], agent: str | None = None) -> dict[str, int]:
    counts: dict[str, int] = {}
    for row in items:
        if agent and _row_agent(row) != agent:
            continue
        st = str(row.get("status") or "ready")
        counts[st] = counts.get(st, 0) + 1
    return counts


def _compress_slice(row: dict[str, Any]) -> dict[str, Any]:
    keep = (
        "id",
        "title",
        "priority",
        "agent",
        "lane",
        "status",
        "stop_point",
        "deliverable",
        "witness",
        "commands",
        "docs",
        "playbook",
        "blocked_by",
        "fallback_when_blocked",
        "note",
    )
    return {k: row[k] for k in keep if k in row and row[k] not in (None, "", [])}


def _pick_next(
    items: list[dict[str, Any]],
    agent: str,
) -> tuple[dict[str, Any] | None, dict[str, Any] | None, str]:
    """
    Returns (work_slice, blocked_primary, drain_reason).
    blocked_primary is the stop-point slice we could not start (if any).
    """
    norm = _normalize_agent(agent)
    by_id = _by_id(items)
    mine = [
        x
        for x in items
        if _normalize_agent(str(x.get("agent") or "")) == norm
    ]
    mine.sort(key=lambda x: int(x.get("priority") or 999))

    stop_points = [x for x in mine if x.get("stop_point")]
    for sp in stop_points:
        st = str(sp.get("status") or "ready")
        if st == "done":
            continue
        if st == "ready" and _deps_satisfied(sp, by_id):
            return sp, None, "stop_point_ready"
        if st in ("ready", "blocked", "in_progress"):
            blocked_primary = sp
            fallback_id = sp.get("fallback_when_blocked")
            if fallback_id:
                fb = by_id.get(str(fallback_id))
                if fb and str(fb.get("status")) == "ready" and _deps_satisfied(fb, by_id):
                    return fb, blocked_primary, f"drain_fallback:{fallback_id}"
            # try any other ready item for this agent
            for row in mine:
                if row["id"] == sp["id"]:
                    continue
                if str(row.get("status")) != "ready":
                    continue
                if not _deps_satisfied(row, by_id):
                    continue
                return row, blocked_primary, f"drain_while_blocked:{sp['id']}"

    for row in mine:
        if str(row.get("status")) != "ready":
            continue
        if not _deps_satisfied(row, by_id):
            continue
        return row, None, "next_ready"

    return None, None, "lane_idle"


def agent_queue_next(
    agent: str,
    *,
    queue: str = "auto",
    mark_in_progress: bool = False,
) -> dict[str, Any]:
    """Next drainable slice for an agent — never returns wait-only without a drain alternative."""
    resolved_queue = resolve_agent_queue(agent, queue)
    norm = _normalize_agent(agent)

    if resolved_queue == "multi_parallel":
        items = load_multi_parallel_items()
        work, blocked_primary, reason = _pick_next_multi(items, norm)
        if work and mark_in_progress and str(work.get("status")) == "ready":
            _mark_slice_in_progress(str(work["id"]), load_queue("multi_parallel"))
        board_lines = [
            f"{row.get('id')}|{row.get('status')}|{_row_agent(row)}|{row.get('track', '')}"
            for row in sorted(items, key=_pick_sort_key)
            if _row_agent(row) == norm
        ]
        out: dict[str, Any] = {
            "queue": resolved_queue,
            "queue_requested": queue,
            "agent": norm,
            "action": "work" if work else "idle",
            "drain_reason": reason,
            "board_counts": _status_counts(items),
            "board_counts_agent": _status_counts(items, norm),
            "board_lines": board_lines,
            "tracks_open": _tracks_summary(items, norm),
            "token_policy": [
                "Use validate_*_report compress=4 — not raw cargo/blender logs",
                "Use witness_brief / handoff_brief — not full JSON/markdown dumps",
                "On exit: agent_queue_update(slice_id, done|blocked) + dual Q✓ dispatch",
            ],
            "queue_path": QUEUE_REGISTRY["multi_parallel"],
            "get_que_hint": f"get-que {norm} --demand --minutes 60 for hour-scale todo list",
        }
        if work:
            out["slice"] = _compress_work_row(work)
        if blocked_primary:
            out["blocked_primary"] = {
                "id": blocked_primary.get("id"),
                "status": blocked_primary.get("status"),
                "depends_on": blocked_primary.get("depends_on"),
            }
        return out

    items = load_queue(resolved_queue)
    work, blocked_primary, reason = _pick_next(items, norm)

    if work and mark_in_progress and str(work.get("status")) == "ready":
        by_id = _by_id(items)
        row = by_id[str(work["id"])]
        row["status"] = "in_progress"
        row["started_at"] = datetime.now(timezone.utc).isoformat()
        save_queue(resolved_queue, items)

    board_lines = [
        f"{row.get('id')}|{row.get('status')}|{row.get('agent')}"
        for row in sorted(items, key=lambda x: int(x.get("priority") or 999))
    ]

    out = {
        "queue": resolved_queue,
        "queue_requested": queue,
        "agent": norm,
        "action": "work" if work else "idle",
        "drain_reason": reason,
        "board_counts": _status_counts(items),
        "board_counts_agent": _status_counts(items, norm),
        "board_lines": board_lines,
        "token_policy": [
            "Use validate_*_report compress=4 — not raw cargo/blender logs",
            "Use witness_brief / handoff_brief — not full JSON/markdown dumps",
            "On exit: agent_queue_update(slice_id, done|blocked) + HANDOFF.md one paragraph",
        ],
        "queue_path": str(queue_path(resolved_queue).relative_to(repo_root())).replace("\\", "/"),
    }
    if work:
        out["slice"] = _compress_slice(work)
    if blocked_primary:
        out["blocked_primary"] = {
            "id": blocked_primary.get("id"),
            "status": blocked_primary.get("status"),
            "blocked_by": blocked_primary.get("blocked_by"),
            "title": blocked_primary.get("title"),
        }
    return out


def agent_queue_update(
    slice_id: str,
    status: str,
    *,
    note: str = "",
    queue: str = "auto",
    enforce: bool = False,
) -> dict[str, Any]:
    st = status.strip().lower()
    if st not in VALID_STATUS:
        raise ValueError(f"invalid status {status!r}; use one of {sorted(VALID_STATUS)}")

    if queue == "auto":
        queue, _ = find_queue_task(slice_id)
    else:
        queue = queue

    if queue == "designer_active":
        path = repo_root() / DESIGNER_ACTIVE_QUEUE
        raw = json.loads(path.read_text(encoding="utf-8"))
        rows = raw.get("multi_parallel_ready")
        if not isinstance(rows, list):
            raise KeyError(f"slice_id not in designer_active mirror: {slice_id}")
        items = rows
    else:
        items = load_queue(queue)

    found = False
    target_row: dict[str, Any] | None = None
    for row in items:
        if str(row.get("id")) != slice_id:
            continue
        found = True
        target_row = row
        if st == "done" and enforce:
            from rust_engine_mcp.validators.queue_integrity import check_row_done_allowed

            ok, reason = check_row_done_allowed(row)
            if not ok:
                return {
                    "ok": False,
                    "enforce": True,
                    "slice_id": slice_id,
                    "status": st,
                    "queue": queue,
                    "error": reason,
                }
        row["status"] = st
        if note:
            row["note"] = note
        row["updated_at"] = datetime.now(timezone.utc).isoformat()
        break
    if not found:
        raise KeyError(f"slice_id not in queue: {slice_id}")

    save_queue(queue, items)
    dispatch = _sync_dispatch_row(slice_id, st, note) if queue in ("multi_parallel", "designer_active") else None
    return {
        "ok": True,
        "slice_id": slice_id,
        "status": st,
        "queue": queue,
        "enforce": bool(enforce),
        "had_exit_predicate": isinstance((target_row or {}).get("exit_predicate"), dict),
        "dispatch_sync": dispatch,
    }


def agent_queue_board(
    *,
    queue: str = "grammar",
    agent: str = "",
) -> dict[str, Any]:
    items = load_queue(queue)
    norm = _normalize_agent(agent) if agent else ""
    lines = []
    for row in sorted(items, key=lambda x: int(x.get("priority") or 999)):
        if norm and _normalize_agent(str(row.get("agent") or "")) != norm:
            continue
        lines.append(
            f"{row.get('id')}\t{row.get('status')}\t{row.get('agent')}\t{row.get('title', '')[:60]}"
        )
    return {
        "queue": queue,
        "agent_filter": norm or None,
        "counts": _status_counts(items, norm or None),
        "lines": lines,
    }


AGENT_META_BRIEF_KEYS = (
    "schema",
    "profile",
    "source_system",
    "relative_path",
    "written_at_epoch_secs",
    "agent_commands",
    "related_proofs",
    "orchestrator",
    "docs",
    "agent",
    "lane",
    "program_id",
    "task_id",
)


def _witness_brief_construction(data: dict[str, Any], rel: str) -> dict[str, Any]:
    param = data.get("construction_parametric_placement_001")
    if not isinstance(param, dict):
        param = {}
    mp = data.get("map_pick_closure_001")
    if not isinstance(mp, dict):
        mp = {}
    return {
        "profile": "construction",
        "path": rel,
        "green": data.get("green"),
        "operational_green": data.get("operational_green"),
        "footprint_projection_ok": data.get("footprint_projection_ok", mp.get("footprint_projection_ok")),
        "construction_parametric_placement_001": {"green": param.get("green")},
        "map_pick_closure_001": {"green": mp.get("green")},
    }


def _witness_brief_map_pick(data: dict[str, Any], rel: str) -> dict[str, Any]:
    mzc = data.get("map_zoom_coherence_001")
    if not isinstance(mzc, dict):
        mzc = {}
    pick_tile = data.get("pick_tile") or data.get("ghost_tile") or data.get("action_tile")
    ghost_origin = data.get("ghost_origin") or data.get("ghost_tile")
    return {
        "profile": "map_pick",
        "path": rel,
        "footprint_projection_ok": data.get("footprint_projection_ok", mzc.get("green")),
        "cursor_delta_px": data.get("cursor_delta_px") or data.get("cursor_reproject_delta_px"),
        "gpu_path_active": data.get("gpu_path_active", data.get("gpu_footprint_active")),
        "authority_drift": data.get("authority_drift", False),
        "pick_tile": pick_tile,
        "ghost_origin": ghost_origin,
        "pick_delta_world_max": mzc.get("pick_delta_world_max"),
        "ghost_screen_delta_px_max": mzc.get("ghost_screen_delta_px_max"),
        "map_pick_closure_math_ok": data.get("map_pick_closure_math_ok"),
    }


def _witness_brief_fire_product(data: dict[str, Any], rel: str) -> dict[str, Any]:
    fire = data.get("triage_fire_product_001") or data.get("fire_product_001")
    if not isinstance(fire, dict):
        fire = {}
    return {
        "profile": "fire_product",
        "path": rel,
        "green": data.get("green"),
        "triage_fire_product_001": {"green": fire.get("green", data.get("green"))},
        "operational_spark_rows_gt_0": fire.get("operational_spark_rows_gt_0", data.get("operational_spark_rows_gt_0")),
        "fire_inst": data.get("fire_inst"),
    }


def _witness_brief_honesty(data: dict[str, Any], rel: str) -> dict[str, Any]:
    """MCP-WIT-030 — failed rule ids only (compress 4)."""
    from rust_engine_mcp.validators.witness_honesty import load_witness_integrity_catalog, validate_witness_honesty

    catalog = load_witness_integrity_catalog()
    report = validate_witness_honesty(
        data,
        witness_rel=rel,
        catalog=catalog,
        compression_level=3,
    )
    failed_rule_ids = sorted(
        {
            str(issue.symbol or issue.kind)
            for issue in report.errors
            if issue.severity == "error" and (issue.symbol or issue.kind)
        }
    )
    warning_rule_ids = sorted(
        {
            str(issue.symbol or issue.kind)
            for issue in report.errors
            if issue.severity == "warning" and (issue.symbol or issue.kind)
        }
    )
    return {
        "profile": "honesty",
        "path": rel,
        "status": report.status,
        "failed_rule_ids": failed_rule_ids,
        "warning_rule_ids": warning_rule_ids,
        "q_forbidden": report.status != "passed",
        "blang": "BLANG:WIT-HON",
    }


def _last_witness_integrity_scan_summary() -> dict[str, Any]:
    """Last ops/post_build scan rollup for slice_exec_brief."""
    ops_path = repo_root() / "debug_runs/mcp_witness_integrity_ops_live.json"
    if ops_path.is_file():
        try:
            body = json.loads(ops_path.read_text(encoding="utf-8"))
            cache = body.get("integrity_cache") or {}
            return {
                "source": str(ops_path.relative_to(repo_root())).replace("\\", "/"),
                "fail_count": cache.get("fail_count"),
                "inflated_green_count": cache.get("inflated_green_count"),
                "queue_contradiction_count": cache.get("queue_contradiction_count"),
                "queue_stale_count": cache.get("queue_stale_count"),
                "enforce_mode": body.get("enforce_mode"),
            }
        except (OSError, json.JSONDecodeError):
            pass
    try:
        from rust_engine_mcp.witness_honesty_lib import build_integrity_cache

        cache = build_integrity_cache(compression_level=3)
        return {
            "source": "live_scan",
            "fail_count": cache.get("fail_count"),
            "inflated_green_count": cache.get("inflated_green_count"),
            "queue_contradiction_count": cache.get("queue_contradiction_count"),
            "queue_stale_count": cache.get("queue_stale_count"),
        }
    except Exception as exc:  # noqa: BLE001
        return {"source": "unavailable", "error": str(exc)}


def _slice_witness_honesty_brief(row: dict[str, Any], witness_rel: str) -> dict[str, Any]:
    """MCP-WIT-031 — honesty block for slice_exec_brief before BLANG:Q✓."""
    out: dict[str, Any] = {"last_scan": _last_witness_integrity_scan_summary()}
    if witness_rel.endswith(".json"):
        brief = witness_brief(witness_rel, profile="honesty")
        out["witness"] = brief.get("brief") if brief.get("ok") else {"error": brief.get("error")}
        out["q_forbidden"] = bool((brief.get("brief") or {}).get("q_forbidden"))
    elif isinstance(row.get("exit_predicate"), dict):
        out["q_forbidden"] = True
        out["note"] = "exit_predicate on row; validate witness JSON for WIT-HON"
    else:
        out["q_forbidden"] = None
    return out


def witness_brief(path: str, *, profile: str | None = None, max_list_items: int = 8) -> dict[str, Any]:
    """Compressed witness JSON — green flag + key fields; optional profile dispatch."""
    p = Path(path)
    if not p.is_absolute():
        p = repo_root() / path
    if not p.is_file():
        return {"ok": False, "error": f"missing: {p}"}

    data = json.loads(p.read_text(encoding="utf-8"))
    rel = str(p.relative_to(repo_root())).replace("\\", "/")

    if profile:
        prof = profile.strip().lower()
        if prof not in WITNESS_PROFILES:
            return {"ok": False, "error": f"unknown profile: {profile!r}; use one of {sorted(WITNESS_PROFILES)}"}
        if prof == "construction":
            brief = _witness_brief_construction(data, rel)
        elif prof == "map_pick":
            brief = _witness_brief_map_pick(data, rel)
        elif prof == "honesty":
            brief = _witness_brief_honesty(data, rel)
        else:
            brief = _witness_brief_fire_product(data, rel)
        return {"ok": True, "brief": brief}

    brief: dict[str, Any] = {
        "path": rel,
        "green": data.get("green"),
        "status": data.get("status"),
        "proceed_ship": data.get("proceed_ship"),
        "art_quality": data.get("art_quality"),
        "summary": data.get("summary"),
    }
    for key in ("errors", "blockers", "missing_variant_keys", "known_fixes"):
        val = data.get(key)
        if isinstance(val, list) and val:
            brief[key] = val[:max_list_items]
            if len(val) > max_list_items:
                brief[f"{key}_truncated"] = len(val) - max_list_items
    meta = data.get("_agent_meta")
    if isinstance(meta, dict):
        brief["_agent_meta"] = {k: meta[k] for k in AGENT_META_BRIEF_KEYS if k in meta}
        if not brief["_agent_meta"]:
            brief["_agent_meta"] = meta
    return {"ok": True, "brief": brief}


def handoff_brief() -> dict[str, Any]:
    """Parse HANDOFF.md — AUTH block + agent drain picks (not legacy Goal/Blockers sections)."""
    path = repo_root() / "tools" / "orchestrator" / "queues" / "HANDOFF.md"
    if not path.is_file():
        return {"ok": False, "error": "HANDOFF.md missing", "hint": "tools/orchestrator/queues/HANDOFF.template.md"}

    text = path.read_text(encoding="utf-8")
    auth_spine = ""
    agent_picks: dict[str, str] = {}
    in_drain = False

    for line in text.splitlines():
        if "AUTH:" in line and "MAT" in line:
            auth_spine = line.split("AUTH:", 1)[-1].strip()
        low = line.lower()
        if "## agent drain" in low:
            in_drain = True
            continue
        if in_drain and line.startswith("## "):
            break
        if not in_drain or not line.startswith("|"):
            continue
        if "BLANG:Q+" in line or "Agent |" in line or ":---" in line:
            continue
        parts = [p.strip() for p in line.split("|") if p.strip()]
        if len(parts) < 2:
            continue
        agent_cell = parts[0].strip("* ").strip()
        pick_cell = parts[1].strip()
        if agent_cell and pick_cell and not pick_cell.lower().startswith("do not"):
            agent_picks[agent_cell] = pick_cell

    return {
        "ok": True,
        "path": str(path.relative_to(repo_root())),
        "auth_spine": auth_spine,
        "agent_picks": agent_picks,
        "active_queue": "tools/orchestrator/queues/post_drain_phase3_queue.json",
    }


CODER_ACTIVE_QUEUE = "tools/orchestrator/queues/coder_active_queue.json"

_CODER_KEYS: dict[str, str] = {
    "a": "coder_a",
    "b": "coder_b",
    "c": "coder_c",
    "coder-a": "coder_a",
    "coder-b": "coder_b",
    "coder-c": "coder_c",
    "coder_a": "coder_a",
    "coder_b": "coder_b",
    "coder_c": "coder_c",
}


def _load_coder_active_queue() -> dict[str, Any]:
    path = repo_root() / CODER_ACTIVE_QUEUE
    if not path.is_file():
        return {"ok": False, "error": f"missing: {CODER_ACTIVE_QUEUE}"}
    return json.loads(path.read_text(encoding="utf-8"))


def _collect_closed_ids(data: dict[str, Any]) -> set[str]:
    closed: set[str] = set()
    for key in (
        "closed_through_phase_6",
        "closed_2026_06_02",
        "closed_2026_06_03_coder_a",
    ):
        for item in data.get("construction_program", {}).get(key) or []:
            closed.add(str(item))
    for block in data.get("coder_a", {}).get("done_2026_06_02") or []:
        if isinstance(block, dict):
            closed.add(str(block.get("id", "")))
    for block in data.get("coder_b", {}).get("done_2026_06_03") or []:
        if isinstance(block, dict):
            closed.add(str(block.get("id", "")))
    for block in data.get("coder_c", {}).get("done") or []:
        if isinstance(block, dict):
            closed.add(str(block.get("id", "")))
    return {x for x in closed if x}


def simulation_queue_brief() -> dict[str, Any]:
    """MCP-SIM-QUEUE-001 — weather sim train open/blocked rows from simulation_continuation_queue."""
    try:
        items = load_queue("simulation")
    except (FileNotFoundError, ValueError) as exc:
        return {"schema": "simulation_queue_brief_v1", "ok": False, "error": str(exc)}

    open_rows: list[dict[str, Any]] = []
    done_rows: list[str] = []
    for row in sorted(items, key=lambda x: int(x.get("priority") or 999)):
        st = str(row.get("status") or "ready")
        sid = str(row.get("id") or "")
        if st == "done":
            done_rows.append(sid)
        elif st in ("ready", "in_progress"):
            open_rows.append(
                {
                    "id": sid,
                    "status": st,
                    "agent": row.get("agent"),
                    "title": (row.get("title") or "")[:72],
                    "regression": row.get("regression"),
                }
            )

    recommend = open_rows[0]["id"] if open_rows else None
    return {
        "schema": "simulation_queue_brief_v1",
        "ok": True,
        "recommend_next": recommend,
        "open": open_rows,
        "done": done_rows,
        "regression_default": "cargo test -p proc_A_dine01 --lib weather",
        "hint": f"BLANG:Q+ agent_queue_next('coder', queue='simulation') — drain {recommend or 'none'}",
    }


MCP_ACTIVE_QUEUE = "tools/orchestrator/queues/mcp_active_queue.json"

# Authoritative drain order when multiple queues disagree (lower = sooner).
_CODER_MCP_DRAIN_ORDER: tuple[str, ...] = (
    "MCP-P2-KIT002-PLAN",
    "MCP-P2-RUN-EVENT-001",
    "MCP-P2-HONEST-BAKE-001",
    "MCP-PROD-B2",
    "MCP-PROD-C-PILOT",
    "MCP-ATLAS-BRIEF-001",
    "MCP-SPINE-CHAIN-001",
    "MCP-PROD-TILE-VAL",
    "MCP-PROD-INDEX",
    "MCP-HONEST-BAKE-001",
    "MCP-OPS-REPORT-001",
)


def _load_mcp_active_queue() -> dict[str, Any]:
    path = repo_root() / MCP_ACTIVE_QUEUE
    if not path.is_file():
        return {"tasks": []}
    return json.loads(path.read_text(encoding="utf-8"))


def _coder_mcp_rows_from_grammar() -> list[dict[str, Any]]:
    try:
        items = load_queue("grammar")
    except (FileNotFoundError, ValueError, json.JSONDecodeError):
        return []
    out: list[dict[str, Any]] = []
    for row in items:
        agent = str(row.get("agent") or "")
        if _normalize_agent(agent) != "coder-mcp":
            continue
        st = str(row.get("status") or "ready")
        if st == "done":
            continue
        out.append(
            {
                "id": str(row.get("id") or ""),
                "status": st,
                "source": "grammar",
                "priority": int(row.get("priority") or 999),
                "title": (row.get("title") or "")[:80],
                "blocked_by": row.get("blocked_by") or row.get("depends_on"),
                "witness": row.get("witness"),
            }
        )
    return out


def _mcp_active_open_rows(*, agent: str | None = None) -> list[dict[str, Any]]:
    data = _load_mcp_active_queue()
    out: list[dict[str, Any]] = []
    for bucket in ("p2_tasks", "tasks"):
        for row in data.get(bucket) or []:
            row_agent = str(row.get("agent") or "")
            if agent is not None and row_agent != agent:
                continue
            st = str(row.get("status") or "ready")
            if st in ("done", "cancelled"):
                continue
            out.append(
                {
                    "id": str(row.get("id") or ""),
                    "status": st,
                    "agent": row_agent,
                    "source": "mcp_active",
                    "priority": {"P0": 0, "P1": 1, "P2": 2, "P3": 3}.get(
                        str(row.get("priority") or "P9"), 9
                    ),
                    "title": (row.get("goal") or row.get("title") or "")[:80],
                    "blocked_by": row.get("blocked_by") or row.get("depends_on"),
                    "witness": row.get("witness"),
                }
            )
    return out


def _coder_mcp_rows_from_mcp_active() -> list[dict[str, Any]]:
    return [r for r in _mcp_active_open_rows(agent="coder-mcp")]


def _drain_sort_key(row: dict[str, Any]) -> tuple[int, int, str]:
    sid = str(row.get("id") or "")
    try:
        order = _CODER_MCP_DRAIN_ORDER.index(sid)
    except ValueError:
        order = 999
    st_rank = {"ready": 0, "in_progress": 1, "blocked": 2, "deferred": 3}.get(
        str(row.get("status") or ""), 4
    )
    return (st_rank, order, sid)


def coder_mcp_drain_brief() -> dict[str, Any]:
    """MCP-CODER-MCP-DRAIN-001 — all open @coder-mcp slices across queues + drain order."""
    merged: dict[str, dict[str, Any]] = {}
    for row in _coder_mcp_rows_from_grammar() + _coder_mcp_rows_from_mcp_active():
        sid = str(row.get("id") or "")
        if not sid:
            continue
        if sid not in merged:
            merged[sid] = row
            continue
        prev = merged[sid]
        if prev.get("status") != "ready" and row.get("status") == "ready":
            merged[sid] = {**prev, **row}

    open_rows = sorted(merged.values(), key=_drain_sort_key)
    ready = [r for r in open_rows if r.get("status") == "ready"]
    blocked = [r for r in open_rows if r.get("status") == "blocked"]
    deferred = [r for r in open_rows if r.get("status") == "deferred"]

    drain_todos = [
        {
            "n": i + 1,
            "id": r["id"],
            "status": r["status"],
            "title": r.get("title"),
            "witness": r.get("witness"),
            "source": r.get("source"),
        }
        for i, r in enumerate(open_rows)
    ]

    recommend = ready[0]["id"] if ready else (open_rows[0]["id"] if open_rows else None)

    return {
        "schema": "coder_mcp_drain_brief_v1",
        "ok": True,
        "recommend_next": recommend,
        "ready": [r["id"] for r in ready],
        "blocked": [r["id"] for r in blocked],
        "deferred": [r["id"] for r in deferred],
        "drain_todos": drain_todos,
        "session_loop": [
            "BLANG:PRE",
            "coder_mcp_drain_brief()",
            "agent_queue_next('coder-mcp')",
            "work one slice",
            "BLANG:PY -k aps",
            "BLANG:WIT",
            "BLANG:Q✓ + agent_run_append",
        ],
        "regression": "pytest tools/mcp/python/tests/ -k 'aps or material_brief or mcp_productivity'",
        "maintain": [
            "HANDOFF.md stale ○ rows → 🟢 via BLANG:Q✓",
            "MICRO_TOOLS_REGISTRY_v1.md after each new tool",
            "operator APS 960×600 sign-off (manual)",
        ],
        "hint": f"Drain {recommend} first — one slice per session",
    }


def orchestrator_mcp_lane_brief() -> dict[str, Any]:
    """@orchestrator-mcp begin-work — P2 lane order + recommend_next (SYMLANG packet fields)."""
    data = _load_mcp_active_queue()
    order_rel = str(data.get("lane_order") or "tools/orchestrator/queues/mcp_lane_order_v1.md")
    order_path = repo_root() / order_rel
    drain = data.get("coder_mcp_drain") or {}

    p2_rows = sorted(
        [r for r in _mcp_active_open_rows() if r.get("source") == "mcp_active"],
        key=_drain_sort_key,
    )
    ready = [r for r in p2_rows if r.get("status") == "ready"]
    recommend = drain.get("orchestrator_mcp_pick") or (ready[0]["id"] if ready else None)

    by_agent: dict[str, list[str]] = {}
    for row in p2_rows:
        by_agent.setdefault(str(row.get("agent") or ""), []).append(str(row.get("id") or ""))

    return {
        "schema": "orchestrator_mcp_lane_brief_v1",
        "ok": True,
        "program_id": data.get("program_id"),
        "prior_program_id": data.get("prior_program_id"),
        "lane_order_path": order_rel,
        "lane_order_exists": order_path.is_file(),
        "auth_spine": "MAT★ ⇢ APS★ ⇢ SNAP★ ⇢ WRK★ ⇢ ATL★ ⇢ RT★",
        "recommend_next": recommend,
        "ready_ids": [r["id"] for r in ready],
        "p2_tasks": p2_rows,
        "by_agent": by_agent,
        "paused": ["MCP-PILOT-GRAMMAR-001"],
        "frozen": data.get("frozen") or [],
        "session_loop": [
            "BLANG:STATS",
            "agent_session_bootstrap(agent='orchestrator-mcp')",
            "handoff_brief()",
            "orchestrator-mcp-lane-brief",
            "issue explicit order → delegate G0…G5",
            "ops_intelligence_scan on lane close",
        ],
        "explicit_order_template": (
            "⟨EXPLICIT-ORDER⟩ in tools/orchestrator/queues/mcp_lane_order_v1.md § Delegate paste"
        ),
        "hint": f"Issue ΔWF→@{_agent_for_slice(recommend, p2_rows)} {recommend}" if recommend else "P2 queue empty",
    }


def _agent_for_slice(slice_id: str | None, rows: list[dict[str, Any]]) -> str:
    if not slice_id:
        return "planner-mcp"
    for row in rows:
        if row.get("id") == slice_id:
            return str(row.get("agent") or "planner-mcp")
    return "planner-mcp"


def coder_drain_brief(coder: str = "c") -> dict[str, Any]:
    """MCP-CODER-DRAIN-001 — active vs closed slices for @coder A/B/C; flags stale paste dispatches."""
    data = _load_coder_active_queue()
    if data.get("ok") is False:
        return data

    key = _CODER_KEYS.get(coder.strip().lower(), coder.strip().lower())
    if key not in ("coder_a", "coder_b", "coder_c"):
        return {
            "ok": False,
            "error": f"unknown coder {coder!r}; use a|b|c or coder_a|coder_b|coder_c",
        }

    closed_ids = _collect_closed_ids(data)
    lane = data.get(key) or {}
    active = lane.get("active") or []
    stale: list[str] = []
    open_rows: list[dict[str, Any]] = []
    for row in active:
        sid = str(row.get("id") or "")
        st = str(row.get("status") or "ready")
        if sid in closed_ids or st == "done":
            stale.append(sid)
        elif st in ("ready", "in_progress"):
            open_rows.append(
                {
                    "id": sid,
                    "status": st,
                    "program": row.get("program"),
                    "exit": (row.get("exit") or "")[:120],
                }
            )

    program_next: list[str] = []
    if key == "coder_a":
        program_next = list(data.get("construction_program", {}).get("coder_a_horizon") or [])
        infra = list(data.get("infrastructure_program", {}).get("coder_a_next") or [])
        if not open_rows and infra:
            program_next = infra
    elif key == "coder_b":
        program_next = list(data.get("construction_program", {}).get("coder_b_next") or [])
        if not program_next:
            program_next = list(data.get("construction_program", {}).get("after_p6") or [])
    elif key == "coder_c":
        program_next = list(data.get("weather_program", {}).get("coder_c_next") or [])

    recommend = open_rows[0]["id"] if open_rows else (program_next[0] if program_next else None)

    drain_ref = "docs/archive/2026-06-src-dev/plans/construction_coder_drain_order_v1.md"
    if key == "coder_c":
        drain_ref = data.get("weather_program", {}).get("plan_doc", "docs/archive/2026-06-src-dev/plans/plan_weather_parallel_lane_v1.md")

    hint = "BLANG:Q+ for grammar queue; use witness_brief on exit paths"
    if stale:
        hint = (
            f"Stale dispatch — {', '.join(stale)} closed on disk; do not re-implement. "
            f"Pick {recommend or 'horizon slice'}."
        )

    return {
        "schema": "coder_drain_brief_v1",
        "ok": True,
        "coder": key,
        "active_open": open_rows,
        "stale_in_active": stale,
        "recommend_next": recommend,
        "program_next": program_next[:6],
        "construction_closed": sorted(closed_ids)[:24],
        "drain_ref": drain_ref,
        "regression": {
            "coder_a": "cargo test -p proc_A_dine01 --lib construction",
            "coder_b": "cargo test -p proc_A_dine01 --lib construction",
            "coder_c": "cargo test -p proc_A_dine01 --lib weather",
        }.get(key),
        "hint": hint,
    }


def file_digest(path: str, *, max_lines: int = 40) -> dict[str, Any]:
    """Head of file + line count — avoid full-file Read for large sources."""
    p = Path(path)
    if not p.is_absolute():
        p = repo_root() / path
    if not p.is_file():
        return {"ok": False, "error": f"missing: {p}"}
    lines = p.read_text(encoding="utf-8", errors="replace").splitlines()
    return {
        "ok": True,
        "path": str(p.relative_to(repo_root())).replace("\\", "/"),
        "total_lines": len(lines),
        "head": lines[: max(1, min(max_lines, 200))],
        "truncated": len(lines) > max_lines,
    }


def orchestrator_brief(*, use_cached: bool = True) -> dict[str, Any]:
    """Last cargo orchestrator summary — not full diagnostic dump."""
    path = repo_root() / "tools" / "orchestrator" / "state" / "last_run.json"
    if use_cached and path.is_file():
        data = json.loads(path.read_text(encoding="utf-8"))
        return {
            "ok": True,
            "source": "last_run.json",
            "status": data.get("status"),
            "error_count": data.get("error_count"),
            "warning_count": data.get("warning_count"),
            "migration_buckets": list((data.get("migration_buckets") or {}).keys())[:12],
            "hint": "Use validate_cargo_report(use_cached=true, compress=4) for details",
        }
    return {"ok": False, "hint": "Run cargo orchestrate or validate_cargo_report"}


def token_savings_guide() -> dict[str, Any]:
    """Static policy — tools to use instead of raw logs / full files."""
    return {
        "validators": {
            "cargo": "validate_cargo_report(compress=4, use_cached=true)",
            "bevy": "validate_bevy_report(compress=4)",
            "glb": "validate_asset_report(path, compress=4)",
            "mcp_job": "validate_report('mcp_job', path, compress=4)",
        },
        "briefs": {
            "witness": "witness_brief('debug_runs/...json', profile='construction'|'map_pick'|'fire_product'|'honesty')",
            "handoff": "handoff_brief()",
            "review_order": "review_order_brief()",
            "slice_exec": "slice_exec_brief('TRIAGE-MAP-PICK-CLOSURE-001')",
            "ops_project": "ops_get_project_brief()",
            "ops_retry": "ops_get_retry_guidance('<task_id>')",
            "ops_blockers": "ops_get_active_blockers()",
            "landscape_presets": "landscape_grammar_presets_witness()",
            "orchestrator": "orchestrator_brief()",
            "file_peek": "file_digest('src/.../file.rs', max_lines=40)",
            "snapshot": "snapshot_digest('assets/staging/assemblies/<id>.json')",
            "p0_plain": "validate_p0_gate_plain('<snapshot>.json')",
            "coder_drain": "coder_drain_brief('a'|'b'|'c') — before pasting @coder dispatch blocks",
            "simulation": "simulation_queue_brief() — weather train open rows",
            "coder_mcp_drain": "coder_mcp_drain_brief() — full @coder-mcp drain board",
            "orchestrator_mcp_lane": "orchestrator-mcp-lane-brief — @orchestrator-mcp P2 pick + explicit order",
            "get_que": "get_que('<agent>', demand=true, minutes=60) — multi-parallel Q+ + hour todo list",
            "agent_queue_demand": "agent_queue_demand('<agent>', minutes=60) — ordered session slices only",
        },
        "witness_integrity": {
            "blang": "BLANG:WIT-HON",
            "witness_brief": "witness_brief(path, profile='honesty') — failed rule ids only",
            "validate_witness": "validate-report witness_honesty <path> --compress 3",
            "validate_scan": "validate-report witness_honesty --scan debug_runs --compress 3",
            "validate_queue": "validate-report queue_integrity --compress 3",
            "ops_witness": "debug_runs/mcp_witness_integrity_ops_live.json",
            "enforce_env": "RUST_ENGINE_WITNESS_INTEGRITY_ENFORCE=1",
            "hook_disable": "RUST_ENGINE_WITNESS_HONESTY_HOOK=0",
            "agent_rule": "BLANG:Q✓ forbidden when witness_honesty FAIL on row witness + rollup parents",
        },
        "session_start": [
            "agent_doc_reads_brief(min_reads=2)",
            "agent_session_bootstrap('<agent>')",
            "token_savings_guide()",
            "pipeline_preflight()",
            "agent_queue_next('<agent>')",
        ],
        "blang": {
            "spec": "src/dev/agent_lang_v1.md",
            "flow": "src/dev/a2c_commit_flow_v1.md",
            "session": "BLANG:PRE → BLANG:Q+ → L4 work → L5 tools → L6 WIT → BLANG:Q✓",
            "tokens": {
                "BLANG:PRE": "pipeline_preflight()",
                "BLANG:Q+": "agent_queue_next('<agent>')",
                "BLANG:Q✓": "agent_queue_update(id, 'done', note=witness_path)",
                "BLANG:HO": "handoff_brief()",
                "BLANG:WIT": "witness_brief('debug_runs/...json', profile='construction'|'map_pick'|'fire_product')",
                "BLANG:WIT-HON": "witness_brief(path, profile='honesty') | validate-report witness_honesty <path>|--scan debug_runs | validate-report queue_integrity",
                "BLANG:REVIEW": "review_order_brief()",
                "BLANG:OPS": "ops_get_project_brief() — delta_wf + active_blockers composed",
                "BLANG:OPS-RETRY": "ops_get_retry_guidance('<task_id>')",
                "BLANG:OPS-BLOCK": "ops_get_active_blockers()",
                "BLANG:SLICE": "slice_exec_brief('<slice_id>')",
                "BLANG:PLACE": "validate_report('construction', path, compress=3)",
                "BLANG:DIGEST": "snapshot_digest(path) — includes arch_dna block",
                "BLANG:DNA": "arch_dna_snapshot_brief(path)",
                "BLANG:DIFF": "snapshot_diff_brief(before, after)",
                "BLANG:P0": "validate_p0_gate_plain(path)",
                "BLANG:CARGO": "validate_cargo_report(compress=4, use_cached=true)",
                "BLANG:BEVY": "validate_bevy_report(compress=4)",
                "BLANG:MARK": "agent_marker_append(slice_id, marker, note)",
                "BLANG:PY": "pytest tools/mcp/python/tests/ -k <filter>",
                "BLANG:S5": "cargo test -p proc_A_dine01 --lib stage5",
                "BLANG:ORCH": "cargo orchestrate",
                "BLANG:DOC": "agent_doc_touch(path, intent='ref|orient|implement')",
                "BLANG:STATS": "agent_doc_reads_brief(min_reads=2)",
                "BLANG:BOOT": "agent_session_bootstrap('<agent>')",
                "BLANG:CACHE": "agent_doc_digest_cached(path) — before re-touch",
                "BLANG:PROMOTE": "agent_doc_promote_hot_reads(min_reads=3)",
                "BLANG:RUN": "agent_run_append({slice_id, agent, tools_called, witness})",
            },
            "by_agent": {
                "orchestrator": ["BLANG:PRE", "BLANG:HO", "BLANG:Q+", "BLANG:WIT", "BLANG:Q✓", "BLANG:RUN"],
                "planner-mcp": ["BLANG:HO", "BLANG:Q+", "BLANG:DOC", "BLANG:Q✓"],
                "coder-mcp": ["BLANG:PRE", "BLANG:Q+", "BLANG:REVIEW", "BLANG:OPS", "BLANG:OPS-RETRY", "BLANG:SLICE", "BLANG:PLACE", "BLANG:DIGEST", "BLANG:P0", "BLANG:PY", "BLANG:WIT", "BLANG:WIT-HON", "BLANG:Q✓", "BLANG:RUN"],
                "coder": ["BLANG:Q+", "BLANG:REVIEW", "BLANG:SLICE", "BLANG:PLACE", "BLANG:CARGO", "BLANG:BEVY", "BLANG:S5", "BLANG:WIT", "BLANG:WIT-HON", "BLANG:Q✓"],
            },
            "commits": {
                "SPEC": "planner — $ref:exec.md",
                "WIT": "implementer — witness path",
                "OPS": "operator — checklist row",
            },
        },
        "blang_session_loop": [
            "BLANG:PRE → BLANG:Q+ → work → BLANG:WIT-HON → BLANG:WIT → BLANG:Q✓",
            "Orient/ref: BLANG:DOC — not raw Read",
            "End: BLANG:RUN — agent_run_append telemetry",
        ],
        "artifact_touch": [
            "agent_session_bootstrap(agent) — SESSION-START brief stack + ledger",
            "agent_doc_reads_brief() — hot re-read rollup before blaming drift",
            "agent_doc_promote_hot_reads() — repeated paths → tools/mcp/cache/agent_doc_digests/",
            "agent_doc_digest_cached(path) — prefer cache over re-touch",
            "agent_doc_touch(path, intent='ref|orient|implement') — NOT Read unless implement",
            "snapshot_digest(path) — NOT Read(full snapshot JSON)",
            "material_profile_brief(profile_id) — NOT catalog/registry Read",
            "material_catalog_brief() — MAT node roll-up only",
            "snapshot_diff_brief(before, after) — NOT two full snapshots in chat",
            "validate_p0_gate_plain(path) — NOT validate-report + parse hints",
            "witness_brief(path) — NOT Read(witness json)",
        ],
        "queues": {
            "next_slice": "agent_queue_next('coder', queue='auto')",
            "phase4": "agent_queue_next('coder', queue='phase4')",
            "checkpoint": "agent_queue_update(slice_id, 'done', note='...', queue='phase4')",
            "board": "agent_queue_board(queue='phase4', agent='coder')",
        },
        "never": [
            "Paste full cargo check output into chat",
            "Read entire blend/log files",
            "Re-read AGENTS.md + plan docs every turn without agent_doc_touch",
            "End turn with only 'waiting on X' — call agent_queue_next and drain",
        ],
        "grammar_tools": [
            "grammar_iterate(request_path) — MCP parity with CLI grammar-iterate",
        ],
        "collective_ritual": [
            "agent-markers-brief — ⟨BP:MIRROR⟩ prior honest reflections",
            "agent-marker-append — ⟨BP:SHARE⟩ leave mirror/scan/why/joint for next agent",
            "On idle/blocked: ⟨BP:COLLECT⟩ → ⟨BP:MIRROR⟩ → ⟨BP:SCAN⟩ → ⟨BP:RESUME⟩",
            "$ref:docs/archive/2026-06-src-dev/plans/agent_collective_ritual_v1.md",
        ],
    }
