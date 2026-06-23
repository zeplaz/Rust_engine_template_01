#!/usr/bin/env python3
"""Sync multi_parallel dispatch ready rows → home queue + coder_active sub-lists."""
from __future__ import annotations

import json
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
QUEUES = REPO / "tools/orchestrator/queues"

DISPATCH_PATH = QUEUES / "multi_parallel_tracks_dispatch_v1.json"
HOME_PATH = QUEUES / "multi_parallel_home_queues_v1.json"
CODER_ACTIVE_PATH = QUEUES / "coder_active_queue.json"

DONE = {"done", "closed", "signed", "lib_done"}
PICK = {"ready", "in_progress", "active", "open", "reopened"}
WAIT = {"blocked", "paused", "deferred"}

OWNER_NORM = {
    "coder-mcp": "coder-mcp",
    "coder_mcp": "coder-mcp",
    "designer-mcp": "designer-mcp",
    "coder_a": "coder_a",
    "coder b": "coder_b",
    "coder_b": "coder_b",
    "coder a": "coder_a",
    "designer": "designer",
    "operator": "operator",
    "coder": "coder",
    "sim-steward": "sim-steward",
    "planner-mcp": "planner-mcp",
}


def norm_owner(raw: str) -> str:
    s = raw.lower().replace("@", "").strip()
    for k, v in OWNER_NORM.items():
        if k in s:
            return v
    return s or "?"


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def save_json(path: Path, obj: dict) -> None:
    path.write_text(json.dumps(obj, indent=2) + "\n", encoding="utf-8")


def index_by_id(rows: list) -> dict[str, dict]:
    out: dict[str, dict] = {}
    for row in rows:
        if isinstance(row, dict) and "id" in row:
            out[row["id"]] = row
    return out


def compact_pick(row: dict, queue: str) -> dict:
    dep = row.get("depends_on") or row.get("blocked_by") or []
    if not isinstance(dep, list):
        dep = [dep]
    return {
        "id": row["id"],
        "status": row.get("status", "ready"),
        "owner": norm_owner(str(row.get("owner", "?"))),
        "track": row.get("track", ""),
        "track_id": row.get("track_id", ""),
        "wave": row.get("wave"),
        "seq": row.get("seq"),
        "priority": row.get("priority", ""),
        "goal": row.get("goal", "")[:120],
        "queue": queue,
        "depends_on": dep[:4],
        "home_queue": row.get("home_queue", "multi_parallel_home_queues_v1.json"),
    }


def agent_bucket_key(owner: str) -> str | None:
    if owner in ("coder_a", "coder_b", "coder_c"):
        return owner
    if owner == "coder":
        return "coder_b"  # default coder lane → B unless tagged coder_a
    return None


def main() -> None:
    now = datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")

    dispatch = load_json(DISPATCH_PATH)
    home = load_json(HOME_PATH)
    coder = load_json(CODER_ACTIVE_PATH)

    home_rows = home.get("drain") or []
    home_by_id = index_by_id(home_rows)

    reconciled = 0
    promoted = 0

    # Home wins for terminal states — pull dispatch down from stale "ready".
    for row in dispatch.get("drain") or []:
        if not isinstance(row, dict) or "id" not in row:
            continue
        hr = home_by_id.get(row["id"])
        if not hr:
            continue
        hs = hr.get("status", "")
        if hs in DONE and row.get("status") in PICK:
            row["status"] = hs
            if hr.get("witness"):
                row["witness"] = hr["witness"]
            row["updated_at"] = now
            reconciled += 1
        if hs in WAIT and row.get("status") == "ready":
            row["status"] = hs
            if hr.get("blocked_reason"):
                row["blocked_reason"] = hr["blocked_reason"]
            row["updated_at"] = now
            reconciled += 1

    picks_by_owner: dict[str, list] = defaultdict(list)

    for row in dispatch.get("drain") or []:
        if not isinstance(row, dict) or "id" not in row:
            continue
        st = row.get("status", "")
        if st not in PICK:
            continue
        owner = norm_owner(str(row.get("owner", "?")))
        hr = home_by_id.get(row["id"])
        if hr and hr.get("status") in DONE:
            continue
        if hr and hr.get("blocked_reason") == "NEEDS-DISPLAY":
            continue
        if hr and hr.get("status") in WAIT:
            continue
        # Promote home row to ready when dispatch says pick.
        if hr and hr.get("status") not in PICK | DONE:
            hr["status"] = "ready"
            hr["updated_at"] = now
            promoted += 1
        picks_by_owner[owner].append(compact_pick(row, "multi_parallel_tracks_dispatch_v1.json"))

    # Dedupe picks per owner by id, sort by track/wave/seq.
    for owner, items in picks_by_owner.items():
        seen: set[str] = set()
        deduped = []
        for it in items:
            if it["id"] in seen:
                continue
            seen.add(it["id"])
            deduped.append(it)
        deduped.sort(key=lambda x: (str(x.get("track", "")), x.get("wave") or 0, x.get("seq") or 0))
        picks_by_owner[owner] = deduped

    # Write multi_parallel_pick sub-lists on coder_active agent buckets.
    for owner, items in picks_by_owner.items():
        bucket = agent_bucket_key(owner)
        if bucket and bucket in coder:
            coder[bucket]["multi_parallel_pick"] = items
        elif owner == "coder-mcp":
            coder.setdefault("coder_mcp_lane", {})["multi_parallel_pick"] = items

    # Global rollup for agents without dedicated buckets.
    coder["multi_parallel_pick"] = {
        owner: items for owner, items in sorted(picks_by_owner.items()) if items
    }

    meta = coder.setdefault("_meta", {})
    meta["last_sync"] = now[:10]
    meta["last_dispatch_subqueue_sync"] = now
    meta["dispatch_pick_count"] = sum(len(v) for v in picks_by_owner.values())

    home_meta = home.setdefault("_meta", {})
    home_meta["last_dispatch_subqueue_sync"] = now

    disp_meta = dispatch.setdefault("_meta", {})
    disp_meta["last_subqueue_sync"] = now

    save_json(DISPATCH_PATH, dispatch)
    save_json(HOME_PATH, home)
    save_json(CODER_ACTIVE_PATH, coder)

    print(f"Reconciled stale dispatch rows: {reconciled}")
    print(f"Promoted home rows to ready: {promoted}")
    print(f"Pick rows by owner:")
    for owner, items in sorted(picks_by_owner.items()):
        print(f"  {owner}: {len(items)}")
        for it in items[:6]:
            print(f"    - {it['id']} ({it.get('track_id','')})")
        if len(items) > 6:
            print(f"    ... +{len(items) - 6} more")


if __name__ == "__main__":
    main()
