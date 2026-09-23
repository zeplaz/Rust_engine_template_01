"""Scan orchestrator queues for automatable (non-operator) open slices."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
QUEUE_DIR = ROOT / "tools" / "orchestrator" / "queues"

AUTO_STATUS = {
    "open",
    "queued",
    "ready",
    "pending",
    "in_progress",
    "todo",
    "active",
    "queued_after_phase0",
    "queued_after_rpc1",
}
OP_NEEDLES = (
    "operator",
    "pixel",
    "checklist",
    "g-play-operator",
    "human 10",
    "manual sign",
    "aps-g4 pixel",
)


def collect_items(data: object) -> list[dict]:
    items: list[dict] = []
    if isinstance(data, list):
        items.extend([x for x in data if isinstance(x, dict)])
        return items
    if not isinstance(data, dict):
        return items
    for k in ("drain", "slices", "items", "queue", "rows", "active", "open", "tracks"):
        v = data.get(k)
        if isinstance(v, list):
            items.extend([x for x in v if isinstance(x, dict)])
        if isinstance(v, dict):
            for vv in v.values():
                if isinstance(vv, list):
                    items.extend([x for x in vv if isinstance(x, dict)])
    # multi_parallel style
    for k, v in data.items():
        if k in ("_meta", "meta", "schema"):
            continue
        if isinstance(v, list):
            items.extend([x for x in v if isinstance(x, dict)])
        elif isinstance(v, dict):
            for kk in ("open", "active", "queued", "todo", "blocked", "done", "drain"):
                vv = v.get(kk)
                if isinstance(vv, list):
                    items.extend([x for x in vv if isinstance(x, dict)])
    return items


def main() -> None:
    rows: list[dict] = []
    for p in sorted(QUEUE_DIR.glob("*.json")):
        try:
            data = json.loads(p.read_text(encoding="utf-8"))
        except Exception:
            continue
        for it in collect_items(data):
            sid = str(it.get("id") or it.get("slice_id") or "")
            if not sid:
                continue
            st = str(it.get("status") or "").lower()
            if st in {"done", "closed", "decided", "cancelled", "deferred", "paused"}:
                continue
            owner = str(it.get("owner") or it.get("agent") or "?")
            goal = str(it.get("goal") or it.get("title") or it.get("notes") or "")[:100]
            blob = f"{sid} {owner} {goal} {st}".lower()
            op = any(n in blob for n in OP_NEEDLES) or owner.lower() in {"operator", "human"}
            if st not in AUTO_STATUS and st != "blocked" and "queue" not in st:
                continue
            rows.append(
                {
                    "file": p.name,
                    "id": sid,
                    "owner": owner,
                    "status": st,
                    "op": op,
                    "goal": goal,
                }
            )

    # de-dupe by id keep first
    seen: set[str] = set()
    uniq: list[dict] = []
    for r in rows:
        if r["id"] in seen:
            continue
        seen.add(r["id"])
        uniq.append(r)

    auto = [r for r in uniq if not r["op"] and r["status"] != "blocked"]
    blocked = [r for r in uniq if not r["op"] and r["status"] == "blocked"]
    ops = [r for r in uniq if r["op"]]

    out = {
        "auto_count": len(auto),
        "blocked_non_op_count": len(blocked),
        "operator_count": len(ops),
        "auto": auto[:50],
        "blocked_non_op": blocked[:25],
        "operator_only": ops[:15],
    }
    dest = ROOT / "debug_runs" / "agent_ops" / "auto_fleet_inventory_live.json"
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_text(json.dumps(out, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"wrote": str(dest), "auto": len(auto), "blocked": len(blocked), "op": len(ops)}, indent=2))
    for r in auto[:30]:
        print(f"{r['id']}\t{r['owner']}\t{r['status']}\t{r['file']}")


if __name__ == "__main__":
    main()
