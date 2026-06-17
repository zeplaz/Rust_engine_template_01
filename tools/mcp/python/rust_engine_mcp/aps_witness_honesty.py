"""MCP-APS-WIT-HON-HOOK-001 — BLANG:WIT-HON gate before APS witness green:true."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root


def gate_green_with_witness_honesty(
    body: dict[str, Any],
    witness_rel: str,
    *,
    repo: Path | None = None,
) -> dict[str, Any]:
    """Refuse green:true when witness_honesty fails on the in-memory body."""
    out = dict(body)
    if out.get("green") is not True:
        out["witness_honesty"] = {"status": "skipped", "reason": "green not requested"}
        return out
    from rust_engine_mcp.validators.witness_honesty import validate_witness_honesty

    root = repo or repo_root()
    rel = witness_rel.replace("\\", "/")
    report = validate_witness_honesty(
        out,
        witness_rel=rel,
        root=root,
        compression_level=3,
    )
    out["witness_honesty"] = {
        "status": report.status,
        "error_count": report.error_count,
        "warning_count": report.warning_count,
        "summary": report.summary,
    }
    if report.status == "failed":
        out["green"] = False
        out["not_green_reason"] = f"WIT-HON blocked green: {report.summary}"
    return out


def write_aps_live_witness(
    body: dict[str, Any],
    rel_path: str,
    *,
    schema: str,
    profile: str,
    source_system: str = "aps_witness",
    ritual: str | None = None,
    exit_predicate_must: list[dict[str, Any]] | None = None,
    repo: Path | None = None,
) -> dict[str, Any]:
    """Envelope v1 + WIT-HON hook, then write debug_runs/*_live.json."""
    root = repo or repo_root()
    rel = rel_path.replace("\\", "/")
    payload = dict(body)
    if exit_predicate_must:
        payload["exit_predicate"] = {"witness": rel, "must": exit_predicate_must}
    meta: dict[str, Any] = {
        "schema": schema,
        "profile": profile,
        "relative_path": rel,
        "source_system": source_system,
        "written_at_epoch_secs": int(time.time()),
    }
    if ritual and payload.get("green"):
        meta["ritual"] = ritual
    payload["_agent_meta"] = meta
    payload = gate_green_with_witness_honesty(payload, rel, repo=root)
    if ritual and payload.get("green"):
        payload.setdefault("_agent_meta", meta)["ritual"] = ritual
    out = root / rel
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    payload["written"] = rel
    return payload
