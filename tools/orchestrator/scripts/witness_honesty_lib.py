#!/usr/bin/env python3
"""MCP-WIT-020 — shared witness / queue integrity engine (ops + MCP).

Imported by:
  - tools/orchestrator/scripts/ops_witness_index.py
  - rust_engine_mcp.witness_honesty_lib (bridge)
  - post_build.ps1 / ops_intelligence_scan.ps1 hooks
"""

from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[3]
MCP_PYTHON = REPO_ROOT / "tools" / "mcp" / "python"
OPS_WITNESS_REL = "debug_runs/mcp_witness_integrity_ops_live.json"

_INFLATED_RULES = frozenset(
    {
        "WIT-GREEN-TINT-ZERO",
        "WIT-OPERATOR-LIB-FIXTURE",
        "WIT-ART-DISHONEST",
        "WIT-TINY-PNG-PILOT",
        "WIT-ENV-BOOTSTRAP-ONLY",
    }
)
_ROLLUP_RULES = frozenset({"WIT-ROLLUP-CHILD-ONLY", "WIT-PHASE-CLOSE-WITHOUT-SUB"})
_QUEUE_RULES = frozenset({"WIT-QUEUE-CONTRADICTION", "WIT-SNAG-DONE", "WIT-EXIT-PREDICATE"})


def _ensure_mcp_path() -> None:
    if str(MCP_PYTHON) not in sys.path:
        sys.path.insert(0, str(MCP_PYTHON))


def scan_witness_honesty(
    scan_dir: str | Path = "debug_runs",
    *,
    compression_level: int = 3,
    repo: Path | None = None,
) -> dict[str, Any]:
    _ensure_mcp_path()
    from rust_engine_mcp.validators.witness_honesty import validate_witness_honesty_scan

    root = repo or REPO_ROOT
    report = validate_witness_honesty_scan(
        scan_dir,
        compression_level=compression_level,
        include_queue_rules=False,
    )
    return report.to_dict()


def scan_queue_integrity(
    *,
    queue_filter: str | None = None,
    compression_level: int = 3,
    repo: Path | None = None,
) -> dict[str, Any]:
    _ensure_mcp_path()
    from rust_engine_mcp.validators.queue_integrity import collect_queue_integrity, validate_queue_registry

    root = repo or REPO_ROOT
    validate_queue_registry(repo=root)
    body = collect_queue_integrity(repo=root, queue_filter=queue_filter)
    return {
        "status": "failed" if (body.get("error_count") or 0) > 0 else "passed",
        "summary": (
            f"queue_integrity: contradictions={body.get('contradiction_count')} "
            f"errors={body.get('error_count')}"
        ),
        "error_count": body.get("error_count"),
        "warning_count": body.get("warning_count"),
        "body": body,
    }


def build_integrity_cache(*, repo: Path | None = None, compression_level: int = 1) -> dict[str, Any]:
    """Single scan pass — keyed issues for ops_witness_index honest_gate v2."""
    root = repo or REPO_ROOT
    _ensure_mcp_path()
    from rust_engine_mcp.validators.witness_honesty import (
        evaluate_witness_honesty_rules,
        load_witness_integrity_catalog,
    )

    catalog = load_witness_integrity_catalog(repo=root)
    by_file: dict[str, list[dict[str, Any]]] = {}
    inflated_green_count = 0
    rollup_inflated_count = 0
    scan_root = root / "debug_runs"
    if scan_root.is_dir():
        for path in sorted(scan_root.rglob("*_live.json")):
            try:
                rel = str(path.relative_to(root)).replace("\\", "/")
                data = json.loads(path.read_text(encoding="utf-8"))
            except (OSError, json.JSONDecodeError):
                continue
            file_issues = evaluate_witness_honesty_rules(
                data,
                witness_rel=rel,
                catalog=catalog,
                root=root,
                include_rollup_children=True,
            )
            if file_issues:
                by_file[rel] = [i.to_dict() for i in file_issues]
                for issue in file_issues:
                    kind = str(issue.signature or issue.kind or "")
                    if kind in _INFLATED_RULES:
                        inflated_green_count += 1
                    if kind in _ROLLUP_RULES:
                        rollup_inflated_count += 1

    witness = scan_witness_honesty("debug_runs", compression_level=compression_level, repo=root)
    queue = scan_queue_integrity(compression_level=3, repo=root)
    queue_body = queue.get("body") or {}

    for issue in witness.get("errors") or []:
        if not isinstance(issue, dict):
            continue
        rel = str(issue.get("file") or "").replace("\\", "/")
        if rel and rel not in by_file:
            by_file.setdefault(rel, []).append(issue)
        kind = str(issue.get("symbol") or issue.get("kind") or "")
        if kind in _INFLATED_RULES and rel not in by_file:
            inflated_green_count += 1
        if kind in _ROLLUP_RULES and rel not in by_file:
            rollup_inflated_count += 1

    stale_ids: set[str] = set()
    for row in queue_body.get("stale_ids") or []:
        if isinstance(row, dict) and row.get("id"):
            stale_ids.add(str(row["id"]))

    queue_contradiction_count = int(queue_body.get("contradiction_count") or 0)
    queue_error_count = int(queue_body.get("error_count") or 0)

    return {
        "witness_honesty": witness,
        "queue_integrity": queue,
        "by_file": by_file,
        "stale_ids": sorted(stale_ids),
        "inflated_green_count": inflated_green_count,
        "rollup_inflated_count": rollup_inflated_count,
        "queue_contradiction_count": queue_contradiction_count,
        "queue_stale_count": len(stale_ids),
        "queue_error_count": queue_error_count,
        "fail_count": int(witness.get("error_count") or 0) + queue_error_count,
    }


def honest_gate_v1(body: dict[str, Any], summary: dict[str, Any]) -> str:
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


def classify_honest_gate_v2(
    rel: str,
    body: dict[str, Any],
    summary: dict[str, Any],
    integrity_cache: dict[str, Any] | None = None,
) -> str:
    """honest_gate v2 — inflated_green, rollup_inflated, queue_stale + v1 fallback."""
    rel = rel.replace("\\", "/")
    cache = integrity_cache or {}
    issues = cache.get("by_file", {}).get(rel) or []
    kinds = {str(i.get("symbol") or i.get("kind") or "") for i in issues if isinstance(i, dict)}

    if kinds & _ROLLUP_RULES:
        return "rollup_inflated"
    if kinds & _INFLATED_RULES:
        return "inflated_green"

    task_id = str(
        summary.get("task_id")
        or body.get("slice_id")
        or body.get("gate_id")
        or body.get("gate")
        or ""
    )
    stale_ids = set(cache.get("stale_ids") or [])
    if task_id and task_id in stale_ids:
        return "queue_stale"

    if cache.get("queue_contradiction_count", 0) > 0 and rel.endswith("_live.json"):
        for issue in issues:
            if str(issue.get("symbol") or "") in _QUEUE_RULES:
                return "queue_stale"

    return honest_gate_v1(body, summary)


def run_post_build_hook(*, repo: Path | None = None, enforce: bool | None = None) -> dict[str, Any]:
    """MCP-WIT-023 — witness_honesty scan; warn unless enforce."""
    root = repo or REPO_ROOT
    if enforce is None:
        enforce = __import__("os").environ.get("RUST_ENGINE_WITNESS_INTEGRITY_ENFORCE") == "1"

    cache = build_integrity_cache(repo=root, compression_level=3)
    witness_status = (cache.get("witness_honesty") or {}).get("status")
    queue_status = (cache.get("queue_integrity") or {}).get("status")
    fail_count = int(cache.get("fail_count") or 0)
    exit_code = 1 if enforce and fail_count > 0 else 0

    body = refresh_mcp_witness_integrity_ops_witness(repo=root, cache=cache, enforce=enforce, exit_code=exit_code)
    body["hook"] = "post_build"
    body["enforce"] = enforce
    body["exit_code"] = exit_code
    body["witness_honesty_status"] = witness_status
    body["queue_integrity_status"] = queue_status
    return body


def refresh_mcp_witness_integrity_ops_witness(
    *,
    repo: Path | None = None,
    cache: dict[str, Any] | None = None,
    enforce: bool = False,
    exit_code: int = 0,
) -> dict[str, Any]:
    """MCP-WIT-024 — documents hook wiring + last scan rollup."""
    root = repo or REPO_ROOT
    cache = cache or build_integrity_cache(repo=root, compression_level=3)
    witness = cache.get("witness_honesty") or {}
    queue = cache.get("queue_integrity") or {}
    green = exit_code == 0 and int(cache.get("fail_count") or 0) == 0

    body: dict[str, Any] = {
        "gate": "MCP-WIT-024",
        "green": green,
        "verdict": "PASS" if green else "FAIL",
        "enforce_mode": enforce,
        "exit_code": exit_code,
        "integrity_cache": {
            "fail_count": cache.get("fail_count"),
            "inflated_green_count": cache.get("inflated_green_count"),
            "rollup_inflated_count": cache.get("rollup_inflated_count"),
            "queue_contradiction_count": cache.get("queue_contradiction_count"),
            "queue_stale_count": cache.get("queue_stale_count"),
            "queue_error_count": cache.get("queue_error_count"),
        },
        "witness_honesty": {
            "status": witness.get("status"),
            "summary": witness.get("summary"),
            "error_count": witness.get("error_count"),
            "warning_count": witness.get("warning_count"),
        },
        "queue_integrity": {
            "status": queue.get("status"),
            "summary": queue.get("summary"),
            "error_count": queue.get("error_count"),
        },
        "hooks": {
            "post_build": "tools/orchestrator/hooks/post_build.ps1",
            "ops_scan": "tools/orchestrator/scripts/ops_intelligence_scan.ps1",
            "env_enforce": "RUST_ENGINE_WITNESS_INTEGRITY_ENFORCE=1",
            "env_disable_hook": "RUST_ENGINE_WITNESS_HONESTY_HOOK=0",
        },
        "commands": [
            "python tools/orchestrator/scripts/witness_honesty_lib.py run-hook",
            "python -m rust_engine_mcp.cli validate-report witness_honesty --scan debug_runs --compress 3",
            "python -m rust_engine_mcp.cli validate-report queue_integrity --compress 3",
        ],
        "_agent_meta": {
            "schema": "mcp_witness_integrity_ops_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "MCP_WITNESS_INTEGRITY_OPS",
            "source_system": "witness_honesty_lib",
            "relative_path": OPS_WITNESS_REL,
            "ritual": "BLANG:WIT-HON MCP-WIT-024" if green else None,
        },
    }
    out = root / OPS_WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = OPS_WITNESS_REL
    return body


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="witness_honesty_lib — shared integrity engine")
    sub = parser.add_subparsers(dest="command", required=True)

    sub.add_parser("build-cache").set_defaults(func=lambda _: _cmd_build_cache())
    sub.add_parser("run-hook").set_defaults(func=lambda _: _cmd_run_hook())
    sub.add_parser("refresh-witness").set_defaults(func=lambda _: _cmd_refresh_witness())

    args = parser.parse_args(argv)
    return int(args.func(args))


def _cmd_build_cache() -> int:
    print(json.dumps(build_integrity_cache(), indent=2))
    return 0


def _cmd_run_hook() -> int:
    body = run_post_build_hook()
    print(json.dumps(body, indent=2))
    return int(body.get("exit_code") or 0)


def _cmd_refresh_witness() -> int:
    body = refresh_mcp_witness_integrity_ops_witness()
    print(json.dumps(body, indent=2))
    return 0 if body.get("green") else 1


if __name__ == "__main__":
    sys.exit(main())
