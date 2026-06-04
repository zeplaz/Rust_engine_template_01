"""Agent continuation queues — drain-ready next slice without wait-only turns."""

from __future__ import annotations

import json
import re
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from .paths import repo_root

QUEUE_REGISTRY: dict[str, str] = {
    "grammar": "tools/orchestrator/queues/grammar_continuation_queue.json",
    "continuation": "tools/orchestrator/queues/continuation_queue.json",
}

VALID_STATUS = frozenset({"ready", "blocked", "in_progress", "done", "deferred", "cancelled"})

AGENT_ALIASES: dict[str, str] = {
    "planner-mcp": "planner",
    "coder-mcp": "coder-mcp",
    "designer-mcp": "designer-mcp",
    "@planner": "planner",
    "@coder": "coder",
    "@designer": "designer",
    "@coder-mcp": "coder-mcp",
    "@designer-mcp": "designer-mcp",
}


def _normalize_agent(agent: str) -> str:
    key = agent.strip().lower()
    return AGENT_ALIASES.get(key, key)


def queue_path(queue: str) -> Path:
    rel = QUEUE_REGISTRY.get(queue)
    if not rel:
        raise KeyError(f"unknown queue: {queue!r}; known: {sorted(QUEUE_REGISTRY)}")
    return repo_root() / rel


def load_queue(queue: str) -> list[dict[str, Any]]:
    path = queue_path(queue)
    if not path.is_file():
        raise FileNotFoundError(f"queue file missing: {path}")
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, list):
        raise ValueError(f"queue must be a JSON array: {path}")
    return data


def save_queue(queue: str, items: list[dict[str, Any]]) -> Path:
    path = queue_path(queue)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(items, indent=2) + "\n", encoding="utf-8")
    return path


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
        if agent and _normalize_agent(str(row.get("agent") or "")) != agent:
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
    queue: str = "grammar",
    mark_in_progress: bool = False,
) -> dict[str, Any]:
    """Next drainable slice for an agent — never returns wait-only without a drain alternative."""
    items = load_queue(queue)
    norm = _normalize_agent(agent)
    work, blocked_primary, reason = _pick_next(items, norm)

    if work and mark_in_progress and str(work.get("status")) == "ready":
        by_id = _by_id(items)
        row = by_id[str(work["id"])]
        row["status"] = "in_progress"
        row["started_at"] = datetime.now(timezone.utc).isoformat()
        save_queue(queue, items)

    board_lines = [
        f"{row.get('id')}|{row.get('status')}|{row.get('agent')}"
        for row in sorted(items, key=lambda x: int(x.get("priority") or 999))
    ]

    out: dict[str, Any] = {
        "queue": queue,
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
        "queue_path": str(queue_path(queue).relative_to(repo_root())).replace("\\", "/"),
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
    queue: str = "grammar",
) -> dict[str, Any]:
    st = status.strip().lower()
    if st not in VALID_STATUS:
        raise ValueError(f"invalid status {status!r}; use one of {sorted(VALID_STATUS)}")

    items = load_queue(queue)
    found = False
    for row in items:
        if str(row.get("id")) != slice_id:
            continue
        found = True
        row["status"] = st
        if note:
            row["note"] = note
        row["updated_at"] = datetime.now(timezone.utc).isoformat()
        break
    if not found:
        raise KeyError(f"slice_id not in queue: {slice_id}")

    save_queue(queue, items)
    return {"ok": True, "slice_id": slice_id, "status": st, "queue": queue}


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


def witness_brief(path: str, *, max_list_items: int = 8) -> dict[str, Any]:
    """Compressed witness JSON — green flag + key fields only."""
    p = Path(path)
    if not p.is_absolute():
        p = repo_root() / path
    if not p.is_file():
        return {"ok": False, "error": f"missing: {p}"}

    data = json.loads(p.read_text(encoding="utf-8"))
    brief: dict[str, Any] = {
        "path": str(p.relative_to(repo_root())).replace("\\", "/"),
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
        brief["_agent_meta"] = {k: meta[k] for k in list(meta)[:6]}
    return {"ok": True, "brief": brief}


def handoff_brief() -> dict[str, Any]:
    """Parse HANDOFF.md Goal / Blockers / Next action — not full file."""
    path = repo_root() / "tools" / "orchestrator" / "queues" / "HANDOFF.md"
    if not path.is_file():
        return {"ok": False, "error": "HANDOFF.md missing", "hint": "tools/orchestrator/queues/HANDOFF.template.md"}

    text = path.read_text(encoding="utf-8")
    sections: dict[str, str] = {}
    current = ""
    buf: list[str] = []
    for line in text.splitlines():
        if line.startswith("## "):
            if current:
                sections[current] = "\n".join(buf).strip()[:1200]
            current = line[3:].strip().lower()
            buf = []
            continue
        buf.append(line)
    if current:
        sections[current] = "\n".join(buf).strip()[:1200]

    pick = ("goal", "blockers", "next action (single step)", "state", "commands")
    brief = {k: sections.get(k, "") for k in pick if k in sections}
    return {"ok": True, "path": str(path.relative_to(repo_root())), "sections": brief}


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
            "witness": "witness_brief('debug_runs/...json')",
            "handoff": "handoff_brief()",
            "orchestrator": "orchestrator_brief()",
            "file_peek": "file_digest('src/.../file.rs', max_lines=40)",
        },
        "queues": {
            "next_slice": "agent_queue_next('coder', queue='grammar')",
            "checkpoint": "agent_queue_update(slice_id, 'done', note='...')",
            "board": "agent_queue_board(queue='grammar', agent='planner')",
        },
        "never": [
            "Paste full cargo check output into chat",
            "Read entire blend/log files",
            "Re-read AGENTS.md + plan docs every turn",
            "End turn with only 'waiting on X' — call agent_queue_next and drain",
        ],
    }
