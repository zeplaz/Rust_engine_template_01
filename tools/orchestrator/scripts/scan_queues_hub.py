#!/usr/bin/env python3
"""Scan orchestrator queues for hub compilation."""
from __future__ import annotations

import json
from collections import defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
QUEUES = REPO / "tools/orchestrator/queues"
SKIP = {
    "designer_signoff_registry.json",
    "master_chain_tensor_v1.json",
    "defer_registry.json",
    "parallel_wave_aps_veg_dispatch_v1.json",
    "agent_hub_queue_v1.json",
}

OWNER_NORM = {
    "coder-mcp": "coder-mcp",
    "coder_mcp": "coder-mcp",
    "designer-mcp": "designer-mcp",
    "designer_mcp": "designer-mcp",
    "coder_a": "coder_a",
    "coder b": "coder_b",
    "coder_b": "coder_b",
    "coder a": "coder_a",
    "designer": "designer",
    "planner-mcp": "planner-mcp",
    "planner_mcp": "planner-mcp",
    "planner": "planner",
    "orchestrator-mcp": "orchestrator-mcp",
    "orchestrator": "orchestrator-mcp",
    "sim-steward": "sim-steward",
    "operator": "operator",
    "coder": "coder",
    "operations-intelligence": "operations-intelligence",
}


def norm_owner(raw: str) -> str:
    s = raw.lower().replace("@", "").strip()
    for k, v in OWNER_NORM.items():
        if k in s:
            return v
    return s or "?"


def row_lists(obj: dict) -> list[tuple[str, list]]:
    out: list[tuple[str, list]] = []
    for k in ("drain", "tasks", "p2_tasks", "parallel", "items", "queue", "active", "optional_designer"):
        if isinstance(obj.get(k), list):
            out.append((k, obj[k]))
    # nested agent buckets in coder_active_queue
    for k in ("coder_a", "coder_b", "coder_c", "planner", "designer", "operator"):
        sub = obj.get(k)
        if isinstance(sub, dict):
            for sk, sv in sub.items():
                if isinstance(sv, list) and sv and isinstance(sv[0], dict) and "id" in sv[0]:
                    out.append((f"{k}.{sk}", sv))
    return out


def main() -> None:
    done_st = {"done", "closed", "signed", "lib_done"}
    pick_st = {"ready", "active", "in_progress", "open", "reopened"}
    wait_st = {"blocked", "paused", "deferred"}

    by_pick: dict[str, list] = defaultdict(list)
    by_wait: dict[str, list] = defaultdict(list)
    stats: list[dict] = []

    for fp in sorted(QUEUES.glob("*.json")):
        if fp.name in SKIP:
            continue
        try:
            obj = json.loads(fp.read_text(encoding="utf-8"))
        except json.JSONDecodeError as e:
            stats.append({"queue": fp.name, "error": str(e)[:80]})
            continue
        if isinstance(obj, list):
            lists = [("root", obj)]
            meta = {}
        else:
            meta = obj.get("_meta") or {}
            lists = row_lists(obj)
        if not lists:
            stats.append({"queue": fp.name, "error": "no_row_list"})
            continue
        d = r = b = o = 0
        for _, rows in lists:
            for row in rows:
                if not isinstance(row, dict) or "id" not in row:
                    continue
                st = row.get("status", "?")
                if st in done_st:
                    d += 1
                    continue
                owner = norm_owner(str(row.get("owner") or row.get("agent") or row.get("co_owner") or "?"))
                dep = row.get("depends_on") or row.get("blocked_by") or []
                if not isinstance(dep, list):
                    dep = [dep]
                entry = {
                    "id": row["id"],
                    "status": st,
                    "owner": owner,
                    "queue": fp.name,
                    "program": meta.get("program_id", row.get("program", "")),
                    "depends_on": dep[:4],
                    "priority": row.get("priority", ""),
                }
                if st in pick_st:
                    r += 1
                    by_pick[owner].append(entry)
                elif st in wait_st:
                    b += 1
                    by_wait[owner].append(entry)
                else:
                    o += 1
        stats.append({"queue": fp.name, "done": d, "pick": r, "blocked": b, "other": o})

    hub = {
        "schema_version": "agent_hub_queue_v1",
        "generated_from": "tools/orchestrator/scripts/scan_queues_hub.py",
        "queue_stats": stats,
        "pick_now": {k: v for k, v in sorted(by_pick.items())},
        "blocked_fallback": {k: v for k, v in sorted(by_wait.items())},
        "rule": "If primary pick is blocked, choose another pick_now row for same agent or parallel_ok lane",
    }
    out = REPO / "tools/orchestrator/queues/agent_hub_queue_v1.json"
    out.write_text(json.dumps(hub, indent=2), encoding="utf-8")
    print(f"Wrote {out}")
    print(f"Queues scanned: {len(stats)}")
    for agent, items in sorted(by_pick.items()):
        print(f"  PICK {agent}: {len(items)}")
    for agent, items in sorted(by_wait.items()):
        print(f"  WAIT {agent}: {len(items)}")


if __name__ == "__main__":
    main()
