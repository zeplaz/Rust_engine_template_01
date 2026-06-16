"""PLAN-MCP-AGENT-LANG-001 — doc read tracker + run telemetry + snapshot diff brief."""

from __future__ import annotations

import hashlib
import json
import re
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from . import agent_queue
from .grammar_iterate import compute_snapshot_diff, run_grammar_iterate
from .paths import repo_root

DOC_READS_LEDGER = "debug_runs/agent_ops/doc_reads.jsonl"
RUN_EVENTS_LEDGER = "debug_runs/agent_ops/run_events.jsonl"
AGENT_MARKERS_LEDGER = "debug_runs/agent_ops/agent_markers.jsonl"
AGENT_DOC_READ_WITNESS = "debug_runs/agent_doc_read_001_live.json"
AGENT_RUN_APPEND_WITNESS = "debug_runs/agent_run_append_001_live.json"
DOC_READS_BRIEF_WITNESS = "debug_runs/agent_ops/doc_reads_brief_latest.json"
DIGEST_CACHE_DIR = "tools/mcp/cache/agent_doc_digests"

CANONICAL_SESSION_PATHS: tuple[str, ...] = (
    "prompts/llm_agent_brief.md",
    "docs/archive/2026-06-src-dev/plans/agent_meta_grammar_v3_lattice.md",
    ".cursor/skills/agent-lang/SKILL.md",
    "src/dev/agent_lang_v1.md",
)

AGENT_ROLE_READS: dict[str, list[str]] = {
    "orchestrator": [
        "tools/orchestrator/NEXT.md",
        "tools/orchestrator/queues/agent_queue.md",
    ],
    "orchestrator-mcp": [
        "tools/mcp/README.md",
        "tools/mcp/MICRO_TOOLS_REGISTRY_v1.md",
    ],
    "planner": ["prompts/llm_agent_brief.md"],
    "planner-mcp": [
        "docs/archive/2026-06-src-dev/plans/plan_mcp_agent_lang_program_v1.md",
        "tools/mcp/MICRO_TOOLS_REGISTRY_v1.md",
    ],
    "coder": [
        ".cursor/skills/bevy-simulation-grade/SKILL.md",
        ".cursor/skills/validation-first/SKILL.md",
        "docs/archive/2026-06-src-dev/plans/agent_mcp_consumer_guide_v1.md",
    ],
    "coder-mcp": [
        "tools/mcp/MICRO_TOOLS_REGISTRY_v1.md",
        "docs/archive/2026-06-src-dev/plans/plan_mcp_agent_lang_program_v1.md",
    ],
    "designer": ["prompts/guides/ui_boundary_guide_v1.md"],
    "designer-mcp": ["tools/mcp/README.md"],
    "sim-steward": [
        ".cursor/skills/bevy-simulation-grade/SKILL.md",
        ".cursor/skills/debug-intelligence/SKILL.md",
        ".cursor/skills/cleanup-completion-intelligence/SKILL.md",
    ],
    "main-thread-orchestrator": [
        "tools/orchestrator/queues/HANDOFF.md",
        ".cursor/skills/bevy-simulation-grade/SKILL.md",
    ],
    "debug-intelligence": [".cursor/skills/debug-intelligence/SKILL.md"],
    "cleanup-intelligence": [".cursor/skills/cleanup-completion-intelligence/SKILL.md"],
    "operations-intelligence": [
        "src/dev/plan_agent_operations_intelligence_v1.md",
        "tools/orchestrator/queues/OPS_WITNESS_SPINE.md",
    ],
    "coparent-orchestrator": [
        "tools/orchestrator/queues/HANDOFF.md",
        ".cursor/skills/bevy-simulation-grade/SKILL.md",
    ],
    "_default": ["AGENTS.md"],
}

_VALID_INTENTS = frozenset({"ref", "orient", "implement"})


def _resolve_path(path: str | Path) -> Path:
    p = Path(path)
    if not p.is_absolute():
        p = repo_root() / p
    return p


def _append_jsonl(rel_path: str, row: dict[str, Any]) -> Path:
    out = repo_root() / rel_path
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(row, ensure_ascii=False) + "\n")
    return out


def _iso_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()


def agent_doc_touch(
    path: str | Path,
    *,
    agent: str = "coder-mcp",
    intent: str = "ref",
    max_lines: int = 40,
    session_hint: str = "",
) -> dict[str, Any]:
    """MCP-DOC-READ-001 — ledger doc read + return file_digest (not full Read)."""
    intent_norm = intent.strip().lower()
    if intent_norm not in _VALID_INTENTS:
        return {
            "schema": "agent_doc_touch_v1",
            "ok": False,
            "path": str(path),
            "error": f"intent must be one of {sorted(_VALID_INTENTS)}",
        }

    resolved = _resolve_path(path)
    try:
        rel = str(resolved.relative_to(repo_root())).replace("\\", "/")
    except ValueError:
        rel = str(resolved)

    digest = agent_queue.file_digest(rel, max_lines=max(1, min(max_lines, 200)))
    if not digest.get("ok"):
        return {
            "schema": "agent_doc_touch_v1",
            "ok": False,
            "path": rel,
            "intent": intent_norm,
            "error": digest.get("error", "missing file"),
        }

    ledger_row = {
        "ts": _iso_now(),
        "agent": agent,
        "path": rel,
        "intent": intent_norm,
        "session_hint": session_hint or None,
    }
    ledger_path = _append_jsonl(DOC_READS_LEDGER, ledger_row)

    hint = "Use digest; full Read only when intent=implement"
    if intent_norm == "orient":
        hint = "Digest + witness_brief if path is a witness JSON"
    elif intent_norm == "implement":
        hint = "Full Read/edit allowed for implementation"

    return {
        "schema": "agent_doc_touch_v1",
        "ok": True,
        "path": rel,
        "intent": intent_norm,
        "agent": agent,
        "digest": {
            "total_lines": digest.get("total_lines", 0),
            "head": digest.get("head") or [],
            "truncated": bool(digest.get("truncated")),
        },
        "ledger_appended": True,
        "ledger_path": str(ledger_path.relative_to(repo_root())).replace("\\", "/"),
        "hint": hint,
    }


def _path_slug(rel_path: str) -> str:
    safe = re.sub(r"[^a-zA-Z0-9._-]+", "_", rel_path.replace("\\", "/").strip("/"))
    digest = hashlib.sha256(rel_path.encode("utf-8")).hexdigest()[:12]
    return f"{safe[:80]}_{digest}"


def _load_ledger_rows(*, tail_rows: int = 500) -> list[dict[str, Any]]:
    ledger = repo_root() / DOC_READS_LEDGER
    if not ledger.is_file():
        return []
    lines = ledger.read_text(encoding="utf-8").splitlines()
    rows: list[dict[str, Any]] = []
    for line in lines[-max(1, tail_rows) :]:
        line = line.strip()
        if not line:
            continue
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError:
            continue
    return rows


def agent_doc_reads_brief(
    *,
    min_reads: int = 2,
    tail_rows: int = 500,
    write_witness: bool = True,
) -> dict[str, Any]:
    """MCP-DOC-READ-003 — aggregate doc_reads ledger: hot paths, repeats, promotion hints."""
    rows = _load_ledger_rows(tail_rows=tail_rows)
    by_path: dict[str, dict[str, Any]] = {}
    repeat_in_session: list[dict[str, Any]] = []
    session_path_counts: Counter[tuple[str, str]] = Counter()

    for row in rows:
        path = str(row.get("path") or "")
        if not path:
            continue
        agent = str(row.get("agent") or "unknown")
        intent = str(row.get("intent") or "ref")
        session_hint = str(row.get("session_hint") or "")
        ts = str(row.get("ts") or "")

        bucket = by_path.setdefault(
            path,
            {
                "path": path,
                "count": 0,
                "agents": Counter(),
                "intents": Counter(),
                "last_ts": "",
                "session_hints": Counter(),
            },
        )
        bucket["count"] += 1
        bucket["agents"][agent] += 1
        bucket["intents"][intent] += 1
        if session_hint:
            bucket["session_hints"][session_hint] += 1
            session_path_counts[(path, session_hint)] += 1
        if ts >= bucket["last_ts"]:
            bucket["last_ts"] = ts

    hot_paths: list[dict[str, Any]] = []
    promotion_candidates: list[dict[str, Any]] = []
    for path, bucket in sorted(by_path.items(), key=lambda kv: kv[1]["count"], reverse=True):
        count = bucket["count"]
        if count < min_reads:
            continue
        cache_path = repo_root() / DIGEST_CACHE_DIR / f"{_path_slug(path)}.json"
        entry = {
            "path": path,
            "count": count,
            "agents": dict(bucket["agents"].most_common(5)),
            "intents": dict(bucket["intents"]),
            "last_ts": bucket["last_ts"],
            "cached": cache_path.is_file(),
            "cache_path": str(cache_path.relative_to(repo_root())).replace("\\", "/"),
        }
        hot_paths.append(entry)
        if count >= max(min_reads + 1, 3):
            promotion_candidates.append(entry)

    for (path, session_hint), count in session_path_counts.most_common(12):
        if count < 2:
            continue
        repeat_in_session.append(
            {"path": path, "session_hint": session_hint, "count": count}
        )

    total_reads = len(rows)
    unique_paths = len(by_path)
    payload: dict[str, Any] = {
        "schema": "agent_doc_reads_brief_v1",
        "ok": True,
        "ledger_path": DOC_READS_LEDGER,
        "tail_rows_scanned": tail_rows,
        "total_reads_in_window": total_reads,
        "unique_paths_in_window": unique_paths,
        "min_reads_threshold": min_reads,
        "hot_paths": hot_paths[:20],
        "promotion_candidates": promotion_candidates[:12],
        "repeat_in_session": repeat_in_session[:12],
        "hint": (
            "Run agent_doc_promote_hot_reads() when promotion_candidates non-empty; "
            "prefer agent_doc_digest_cached(path) over re-touch"
        ),
    }
    if write_witness:
        out = repo_root() / DOC_READS_BRIEF_WITNESS
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
        payload["witness_path"] = DOC_READS_BRIEF_WITNESS
    return payload


def agent_doc_digest_cached(
    path: str | Path,
    *,
    max_lines: int = 120,
    force_refresh: bool = False,
) -> dict[str, Any]:
    """Return MCP digest cache entry for path if source mtime unchanged."""
    resolved = _resolve_path(path)
    try:
        rel = str(resolved.relative_to(repo_root())).replace("\\", "/")
    except ValueError:
        rel = str(resolved)

    if not resolved.is_file():
        return {
            "schema": "agent_doc_digest_cached_v1",
            "ok": False,
            "path": rel,
            "error": "missing file",
        }

    source_mtime = resolved.stat().st_mtime
    cache_file = repo_root() / DIGEST_CACHE_DIR / f"{_path_slug(rel)}.json"
    if cache_file.is_file() and not force_refresh:
        try:
            cached = json.loads(cache_file.read_text(encoding="utf-8"))
            if cached.get("source_mtime") == source_mtime:
                return {
                    "schema": "agent_doc_digest_cached_v1",
                    "ok": True,
                    "path": rel,
                    "cache_hit": True,
                    "cache_path": str(cache_file.relative_to(repo_root())).replace("\\", "/"),
                    "read_count_at_promote": cached.get("read_count"),
                    "digest": cached.get("digest"),
                    "hint": "Use cached digest — skip agent_doc_touch unless implement",
                }
        except (json.JSONDecodeError, OSError):
            pass

    digest = agent_queue.file_digest(rel, max_lines=max(1, min(max_lines, 200)))
    return {
        "schema": "agent_doc_digest_cached_v1",
        "ok": bool(digest.get("ok")),
        "path": rel,
        "cache_hit": False,
        "digest": digest if digest.get("ok") else None,
        "hint": "No fresh cache — run agent_doc_promote_hot_reads() or agent_doc_touch()",
    }


def agent_doc_promote_hot_reads(
    *,
    min_reads: int = 3,
    max_promote: int = 8,
    max_lines: int = 120,
) -> dict[str, Any]:
    """MCP-DOC-READ-004 — promote hot ledger paths into tools/mcp/cache/agent_doc_digests/."""
    brief = agent_doc_reads_brief(min_reads=min_reads, write_witness=True)
    cache_dir = repo_root() / DIGEST_CACHE_DIR
    cache_dir.mkdir(parents=True, exist_ok=True)

    promoted: list[dict[str, Any]] = []
    skipped: list[dict[str, Any]] = []

    for row in brief.get("promotion_candidates") or []:
        if len(promoted) >= max(1, max_promote):
            break
        rel = str(row.get("path") or "")
        if not rel:
            continue
        resolved = _resolve_path(rel)
        if not resolved.is_file():
            skipped.append({"path": rel, "reason": "missing"})
            continue

        source_mtime = resolved.stat().st_mtime
        cache_file = cache_dir / f"{_path_slug(rel)}.json"
        if cache_file.is_file():
            try:
                existing = json.loads(cache_file.read_text(encoding="utf-8"))
                if existing.get("source_mtime") == source_mtime:
                    skipped.append({"path": rel, "reason": "cache_fresh"})
                    continue
            except (json.JSONDecodeError, OSError):
                pass

        digest = agent_queue.file_digest(rel, max_lines=max(1, min(max_lines, 200)))
        if not digest.get("ok"):
            skipped.append({"path": rel, "reason": digest.get("error", "digest_failed")})
            continue

        payload = {
            "schema": "agent_doc_digest_cache_v1",
            "source_path": rel,
            "source_mtime": source_mtime,
            "read_count": row.get("count"),
            "promoted_at": _iso_now(),
            "max_lines": max_lines,
            "digest": digest,
        }
        cache_file.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
        promoted.append(
            {
                "path": rel,
                "read_count": row.get("count"),
                "cache_path": str(cache_file.relative_to(repo_root())).replace("\\", "/"),
            }
        )

    return {
        "schema": "agent_doc_promote_hot_reads_v1",
        "ok": True,
        "promoted_count": len(promoted),
        "promoted": promoted,
        "skipped": skipped[:8],
        "cache_dir": DIGEST_CACHE_DIR,
        "brief_witness": DOC_READS_BRIEF_WITNESS,
        "hint": "Next session: agent_doc_digest_cached(path) before agent_doc_touch",
    }


def agent_session_bootstrap(
    agent: str,
    *,
    session_hint: str = "SESSION-START",
    max_lines: int = 60,
    touch_role_reads: bool = False,
) -> dict[str, Any]:
    """MCP-DOC-READ-005 — session start: stats + canonical brief stack via BLANG:DOC ledger."""
    agent_norm = agent.strip() or "coder"
    stats = agent_doc_reads_brief(min_reads=2, write_witness=True)

    canonical: list[dict[str, Any]] = []
    for idx, rel in enumerate(CANONICAL_SESSION_PATHS):
        intent = "orient" if idx == 0 else "ref"
        touch = agent_doc_touch(
            rel,
            agent=agent_norm,
            intent=intent,
            max_lines=max_lines,
            session_hint=session_hint,
        )
        cached = agent_doc_digest_cached(rel, max_lines=max_lines)
        canonical.append(
            {
                "path": rel,
                "intent": intent,
                "ok": touch.get("ok"),
                "digest_lines": (touch.get("digest") or {}).get("total_lines"),
                "cache_hit": cached.get("cache_hit"),
            }
        )

    role_paths = AGENT_ROLE_READS.get(agent_norm, AGENT_ROLE_READS["_default"])
    role_touches: list[dict[str, Any]] = []
    if touch_role_reads:
        for rel in role_paths:
            touch = agent_doc_touch(
                rel,
                agent=agent_norm,
                intent="ref",
                max_lines=max_lines,
                session_hint=session_hint,
            )
            role_touches.append({"path": rel, "ok": touch.get("ok")})

    return {
        "schema": "agent_session_bootstrap_v1",
        "ok": True,
        "agent": agent_norm,
        "session_hint": session_hint,
        "canonical_touches": canonical,
        "role_reads": role_paths,
        "role_touches": role_touches or None,
        "hot_paths": stats.get("hot_paths", [])[:5],
        "repeat_in_session": stats.get("repeat_in_session", [])[:3],
        "promotion_hint": (
            "agent_doc_promote_hot_reads()"
            if stats.get("promotion_candidates")
            else "no promotion needed"
        ),
        "next_steps": [
            "BLANG:ROLE — agent_doc_touch each role_reads path",
            "BLANG:PRE — pipeline_preflight()",
            "BLANG:Q+ — agent_queue_next(agent) or handoff_brief()",
        ],
        "fragment": ".cursor/agents/_fragments/session_bootstrap_v1.md",
    }


def agent_run_append(
    event: dict[str, Any],
    *,
    agent: str | None = None,
) -> dict[str, Any]:
    """MCP-DOC-READ-002 — append session telemetry to run_events.jsonl."""
    if not isinstance(event, dict):
        return {"schema": "agent_run_append_v1", "ok": False, "error": "event must be object"}

    row = dict(event)
    row.setdefault("ts", _iso_now())
    if agent:
        row.setdefault("agent", agent)
    row.setdefault("schema", "agent_run_event_v1")

    ledger_path = _append_jsonl(RUN_EVENTS_LEDGER, row)
    return {
        "schema": "agent_run_append_v1",
        "ok": True,
        "ledger_path": str(ledger_path.relative_to(repo_root())).replace("\\", "/"),
        "appended": row,
    }


def agent_marker_append(
    *,
    agent: str,
    slice_id: str,
    mirror: str = "",
    scan: str = "",
    why: str = "",
    joint: str = "",
    dim: list[str] | None = None,
    breakpoint: str = "⟨BP:SHARE⟩",
    prior_writer: str = "",
    prior_ref: str = "",
    delta_wf: str = "",
    session_hint: str = "",
) -> dict[str, Any]:
    """AGENT-COLLECTIVE-RITUAL-001 — honest reflection marker for next agent."""
    row: dict[str, Any] = {
        "schema": "agent_marker_v1",
        "ts": _iso_now(),
        "agent": agent,
        "slice_id": slice_id,
        "breakpoint": breakpoint,
        "dim": (dim or [])[:3],
        "mirror": mirror or None,
        "scan": scan or None,
        "why": why or None,
        "joint": joint or None,
        "prior_writer": prior_writer or None,
        "prior_ref": prior_ref or None,
        "delta_wf": delta_wf or None,
        "session_hint": session_hint or None,
    }
    row = {k: v for k, v in row.items() if v is not None}
    ledger_path = _append_jsonl(AGENT_MARKERS_LEDGER, row)
    return {
        "schema": "agent_marker_append_v1",
        "ok": True,
        "ledger_path": str(ledger_path.relative_to(repo_root())).replace("\\", "/"),
        "appended": row,
        "hint": "Next agent: ⟨BP:MIRROR⟩ read tail of agent_markers.jsonl before work",
    }


def agent_markers_brief(*, tail: int = 8) -> dict[str, Any]:
    """Compressed tail of honest markers — for ⟨BP:MIRROR⟩."""
    path = repo_root() / AGENT_MARKERS_LEDGER
    if not path.is_file():
        return {"ok": True, "markers": [], "hint": "no markers yet"}
    lines = path.read_text(encoding="utf-8").splitlines()
    rows: list[dict[str, Any]] = []
    for line in lines[-max(1, tail) :]:
        line = line.strip()
        if not line:
            continue
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError:
            continue
    brief = []
    for r in rows:
        brief.append(
            {
                "ts": r.get("ts"),
                "agent": r.get("agent"),
                "slice_id": r.get("slice_id"),
                "dim": r.get("dim"),
                "mirror": (r.get("mirror") or "")[:200],
                "joint": (r.get("joint") or "")[:160],
                "delta_wf": r.get("delta_wf"),
            }
        )
    return {"ok": True, "count": len(brief), "markers": brief, "ledger_path": AGENT_MARKERS_LEDGER}


def snapshot_diff_brief(
    before_path: str | Path,
    after_path: str | Path,
) -> dict[str, Any]:
    """MCP-SNAPSHOT-DIFF-001 — compact placement/footprint diff between two snapshots."""
    before_p = _resolve_path(before_path)
    after_p = _resolve_path(after_path)
    if not before_p.is_file():
        return {"schema": "snapshot_diff_brief_v1", "ok": False, "error": f"missing before: {before_p}"}
    if not after_p.is_file():
        return {"schema": "snapshot_diff_brief_v1", "ok": False, "error": f"missing after: {after_p}"}

    before = json.loads(before_p.read_text(encoding="utf-8"))
    after = json.loads(after_p.read_text(encoding="utf-8"))
    diff = compute_snapshot_diff(before, after)
    bfp = before.get("footprint") or {}
    afp = after.get("footprint") or {}

    def _rel(p: Path) -> str:
        try:
            return str(p.relative_to(repo_root())).replace("\\", "/")
        except ValueError:
            return str(p)

    summary = (
        f"+{diff.get('cells_added', 0)} −{diff.get('cells_removed', 0)} "
        f"~{diff.get('cells_changed', 0)} · layers: {', '.join(diff.get('layers_touched') or []) or '—'}"
    )
    return {
        "schema": "snapshot_diff_brief_v1",
        "ok": True,
        "before": _rel(before_p),
        "after": _rel(after_p),
        "footprint_before": f"{bfp.get('width', '?')}x{bfp.get('depth', '?')}x{bfp.get('floors', '?')}",
        "footprint_after": f"{afp.get('width', '?')}x{afp.get('depth', '?')}x{afp.get('floors', '?')}",
        "summary": summary,
        "diff": diff,
        "hint": "Use with grammar_iterate result — avoid pasting full snapshot JSON in chat",
    }


def grammar_iterate_mcp(
    request_path: str | Path,
    *,
    write_snapshot: bool = False,
    write_witness: str | None = None,
) -> dict[str, Any]:
    """MCP-GRAMMAR-ITER-TOOL — server/CLI parity wrapper over run_grammar_iterate."""
    return run_grammar_iterate(
        request_path,
        write_snapshot=write_snapshot,
        write_witness=write_witness,
    )


def refresh_agent_doc_read_witness() -> bool:
    touch = agent_doc_touch("docs/archive/2026-06-src-dev/plans/plan_mcp_agent_lang_program_v1.md", intent="ref", agent="witness")
    ledger = repo_root() / DOC_READS_LEDGER
    green = bool(touch.get("ok") and touch.get("ledger_appended") and ledger.is_file())
    payload = {
        "gate_id": "MCP-DOC-READ-001",
        "ok": green,
        "green": green,
        "touch_ok": touch.get("ok"),
        "ledger_path": touch.get("ledger_path"),
        "intent": touch.get("intent"),
    }
    out = repo_root() / AGENT_DOC_READ_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return green


def refresh_agent_run_append_witness() -> bool:
    append = agent_run_append(
        {
            "slice_id": "MCP-DOC-READ-002-IMPL",
            "tools_called": ["agent_doc_touch", "agent_run_append"],
            "witness": AGENT_RUN_APPEND_WITNESS,
        },
        agent="witness",
    )
    ledger = repo_root() / RUN_EVENTS_LEDGER
    green = bool(append.get("ok") and ledger.is_file())
    payload = {
        "gate_id": "MCP-DOC-READ-002",
        "ok": green,
        "green": green,
        "append_ok": append.get("ok"),
        "ledger_path": append.get("ledger_path"),
    }
    out = repo_root() / AGENT_RUN_APPEND_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return green
