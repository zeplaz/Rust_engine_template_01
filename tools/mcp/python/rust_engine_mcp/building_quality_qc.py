"""APSR-A4-Q1-001 / BQ-Q1 — read BQ-A2 witness for Assembly QC strip."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

WITNESS_REL = "debug_runs/building_quality_live.json"
PASS_THRESHOLD = 70.0
TASK_ID = "APSR-A4-Q1-001"


def load_building_quality_witness(repo: Path | None = None) -> dict[str, Any] | None:
    root = repo or repo_root()
    path = root / WITNESS_REL
    if not path.is_file():
        return None
    try:
        body = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    if isinstance(body.get("payload"), dict):
        return body["payload"]
    return body


def lookup_assembly_score(
    assembly_id: str,
    *,
    repo: Path | None = None,
) -> dict[str, Any] | None:
    witness = load_building_quality_witness(repo=repo)
    if not witness:
        return None
    for row in witness.get("assemblies") or []:
        if isinstance(row, dict) and row.get("assembly_id") == assembly_id:
            return row
    return None


def format_qc_strip_text(
    assembly_id: str | None = None,
    *,
    repo: Path | None = None,
) -> tuple[str, bool | None]:
    """Return (display_text, ok) for APS Assembly QC strip."""
    witness = load_building_quality_witness(repo=repo)
    if not witness:
        return (
            "Building QC: no witness — run cargo test building_quality or refresh BQ-A2.",
            None,
        )
    if assembly_id:
        row = lookup_assembly_score(assembly_id, repo=repo)
        if row:
            score = float(row.get("overall_score", 0.0))
            ok = bool(row.get("passes_gate", score >= PASS_THRESHOLD))
            purity = float(row.get("style_purity_pct", 0.0))
            adj = int(row.get("adjacency_violation_count", 0))
            missing = int(row.get("missing_slot_count", 0))
            text = (
                f"QC {assembly_id}: score {score:.0f} · purity {purity:.0f}% · "
                f"adj {adj} · missing {missing}"
            )
            return text, ok
    green = bool(witness.get("green"))
    n = len(witness.get("assemblies") or [])
    text = f"Building QC witness: {'pass' if green else 'fail'} · {n} assembly row(s) · threshold {PASS_THRESHOLD:.0f}"
    return text, green if n else None


def assembly_qc_allows_approve(assembly_id: str | None, *, repo: Path | None = None) -> tuple[bool, str]:
    """Return (allowed, reason) — blocks Approve snapshot while QC red."""
    if not assembly_id:
        return False, "No assembly loaded — generate or load a snapshot first."
    row = lookup_assembly_score(assembly_id, repo=repo)
    if row is None:
        witness = load_building_quality_witness(repo=repo)
        if witness is None:
            return False, "Building QC witness missing — run BQ-A2 tests first."
        return True, "Assembly not in witness — approve allowed with caution."
    score = float(row.get("overall_score", 0.0))
    ok = bool(row.get("passes_gate", score >= PASS_THRESHOLD))
    if ok:
        return True, f"QC pass (score {score:.0f})."
    adj = int(row.get("adjacency_violation_count", 0))
    missing = int(row.get("missing_slot_count", 0))
    return False, f"QC fail — score {score:.0f}, adj {adj}, missing {missing}."


def write_apsr_q1_witness(*, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    root = repo or repo_root()
    text, ok = format_qc_strip_text(repo=root)
    witness = load_building_quality_witness(repo=root)
    strip_path = root / "tools/mcp/art_pipeline_suite/assembly_qc_strip.py"
    trace_path = root / "tools/mcp/art_pipeline_suite/generation_trace_strip.py"
    blocks_approve = "assembly_qc_allows_approve" in trace_path.read_text(encoding="utf-8")
    green = (
        witness is not None
        and bool(witness.get("green"))
        and strip_path.is_file()
        and blocks_approve
    )
    body: dict[str, Any] = {
        "task_id": TASK_ID,
        "gate": TASK_ID,
        "green": green,
        "qc_strip_sample": text,
        "building_quality_witness": WITNESS_REL,
        "bq_a1_wired": bool((witness or {}).get("bq_a1_wired")),
        "approve_blocks_on_red_qc": blocks_approve,
        "plan_ref": "src/dev/plan_aps_refactor_v1.md#APSR-Q1",
    }
    return write_aps_live_witness(
        body,
        "debug_runs/apsr_a4_q1_001_live.json",
        schema="apsr_a4_q1_live_v1",
        profile="APSR_A4_Q1",
        source_system="apsr_a4_q1",
        ritual=f"BLANG:WIT-HON {TASK_ID}" if green else None,
        repo=root,
    )
