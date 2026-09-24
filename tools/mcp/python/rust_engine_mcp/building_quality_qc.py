"""APSR-A4-Q1-001 / BQ-Q1 — read BQ-A2 witness for Assembly QC strip.

APS-QC-SMOKE-LABEL-001 — rubric v3 §4.1 smoke / LOD0 strip labels.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from rust_engine_mcp.kit_coverage_audit import CORE_SLOTS
from rust_engine_mcp.library import load_index_json
from rust_engine_mcp.paths import repo_root

WITNESS_REL = "debug_runs/building_quality_live.json"
PASS_THRESHOLD = 70.0
TASK_ID = "APSR-A4-Q1-001"
SMOKE_LABEL_TASK = "APS-QC-SMOKE-LABEL-001"

# Exact copy from src/dev/design_aps_operator_rubric_v3.md §4.1
LABEL_SMOKE = "Smoke kit — not ship"
LABEL_LOD0 = "LOD0 stand-in — production pending"


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


def _index_row_for_placement(
    placement: dict[str, Any],
    index: list[dict[str, Any]],
) -> dict[str, Any] | None:
    job_id = str(placement.get("job_id") or "")
    if job_id:
        for row in index:
            if str(row.get("job_id") or "") == job_id:
                return row
    module_id = str(placement.get("module_id") or "")
    if not module_id:
        return None
    for row in index:
        if str(row.get("module_id") or "") == module_id:
            return row
    return None


def _placement_is_smoke(placement: dict[str, Any], index_row: dict[str, Any] | None) -> bool:
    tier = str((index_row or {}).get("development_tier") or "")
    batch = str((index_row or {}).get("batch_id") or "")
    job_id = str(placement.get("job_id") or (index_row or {}).get("job_id") or "")
    if tier == "smoke":
        return True
    if batch.startswith("kit_greybox") or batch.startswith("kit_smoke"):
        return True
    if "kit_greybox" in job_id or "kit_smoke" in job_id or job_id.endswith("_smoke"):
        return True
    return False


def _placement_is_lod0_core(placement: dict[str, Any], index_row: dict[str, Any] | None) -> bool:
    slot = str(placement.get("slot_key") or "")
    if slot not in CORE_SLOTS:
        return False
    tier = str((index_row or {}).get("development_tier") or "")
    lod_policy = str(placement.get("lod_policy") or "")
    job_id = str(placement.get("job_id") or (index_row or {}).get("job_id") or "")
    if tier == "lod0" or lod_policy == "lod0":
        return True
    if "_lod0_" in job_id or job_id.endswith("_lod0"):
        return True
    return False


def resolve_assembly_snapshot(
    assembly_id: str | None = None,
    *,
    snapshot: dict[str, Any] | None = None,
    repo: Path | None = None,
) -> dict[str, Any] | None:
    """Prefer in-memory snapshot; else load staging JSON for assembly_id."""
    if isinstance(snapshot, dict) and snapshot:
        return snapshot
    if not assembly_id:
        return None
    root = repo or repo_root()
    from rust_engine_mcp.assembly import SNAPSHOT_STAGING, load_assembly_snapshot

    path = root / SNAPSHOT_STAGING / f"{assembly_id}.json"
    if not path.is_file():
        return None
    try:
        return load_assembly_snapshot(path, enrich=False)
    except (OSError, json.JSONDecodeError, ValueError):
        return None


def scan_snapshot_tier_labels(
    snapshot: dict[str, Any] | None,
    *,
    repo: Path | None = None,
) -> dict[str, Any]:
    """Detect smoke / CORE-lod0 hits for APS QC strip (rubric v3 §4.1)."""
    smoke = False
    lod0_core = False
    if not snapshot:
        return {
            "smoke_hit": False,
            "lod0_core_hit": False,
            "labels": [],
            "blocks_approve": False,
        }
    _ = repo  # index is repo-rooted via library.load_index_json
    try:
        index = load_index_json()
    except (OSError, json.JSONDecodeError):
        index = []
    for placement in snapshot.get("module_placements") or []:
        if not isinstance(placement, dict):
            continue
        row = _index_row_for_placement(placement, index)
        if _placement_is_smoke(placement, row):
            smoke = True
        if _placement_is_lod0_core(placement, row):
            lod0_core = True
        if smoke and lod0_core:
            break
    labels: list[str] = []
    if smoke:
        labels.append(LABEL_SMOKE)
    if lod0_core:
        labels.append(LABEL_LOD0)
    return {
        "smoke_hit": smoke,
        "lod0_core_hit": lod0_core,
        "labels": labels,
        "blocks_approve": smoke,
        "task_id": SMOKE_LABEL_TASK,
    }


def format_qc_strip_text(
    assembly_id: str | None = None,
    *,
    snapshot: dict[str, Any] | None = None,
    repo: Path | None = None,
) -> tuple[str, bool | None]:
    """Return (display_text, ok) for APS Assembly QC strip."""
    witness = load_building_quality_witness(repo=repo)
    snap = resolve_assembly_snapshot(assembly_id, snapshot=snapshot, repo=repo)
    tier = scan_snapshot_tier_labels(snap, repo=repo)
    tier_suffix = (" · " + " · ".join(tier["labels"])) if tier["labels"] else ""

    if not witness:
        base = "Building QC: no witness — run cargo test building_quality or refresh BQ-A2."
        if tier_suffix:
            # Smoke still fails the strip even without a BQ witness.
            return base + tier_suffix, False if tier["smoke_hit"] else None
        return base, None
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
                f"adj {adj} · missing {missing}{tier_suffix}"
            )
            if tier["smoke_hit"]:
                ok = False
            return text, ok
    green = bool(witness.get("green"))
    n = len(witness.get("assemblies") or [])
    text = (
        f"Building QC witness: {'pass' if green else 'fail'} · {n} assembly row(s) · "
        f"threshold {PASS_THRESHOLD:.0f}{tier_suffix}"
    )
    ok: bool | None = green if n else None
    if tier["smoke_hit"]:
        ok = False
    return text, ok


def assembly_qc_allows_approve(
    assembly_id: str | None,
    *,
    snapshot: dict[str, Any] | None = None,
    repo: Path | None = None,
) -> tuple[bool, str]:
    """Return (allowed, reason) — blocks Approve snapshot while QC red or smoke-tier."""
    if not assembly_id and not snapshot:
        return False, "No assembly loaded — generate or load a snapshot first."
    snap = resolve_assembly_snapshot(assembly_id, snapshot=snapshot, repo=repo)
    tier = scan_snapshot_tier_labels(snap, repo=repo)
    if tier["smoke_hit"]:
        return False, LABEL_SMOKE
    if not assembly_id:
        return False, "No assembly loaded — generate or load a snapshot first."
    row = lookup_assembly_score(assembly_id, repo=repo)
    if row is None:
        witness = load_building_quality_witness(repo=repo)
        if witness is None:
            return False, "Building QC witness missing — run BQ-A2 tests first."
        if tier["lod0_core_hit"]:
            return True, f"Assembly not in witness — {LABEL_LOD0}"
        return True, "Assembly not in witness — approve allowed with caution."
    score = float(row.get("overall_score", 0.0))
    ok = bool(row.get("passes_gate", score >= PASS_THRESHOLD))
    if ok:
        if tier["lod0_core_hit"]:
            return True, f"QC pass (score {score:.0f}) · {LABEL_LOD0}"
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
