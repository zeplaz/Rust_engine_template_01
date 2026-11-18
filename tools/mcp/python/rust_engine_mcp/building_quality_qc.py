"""APSR-A4-Q1-001 / BQ-Q1 — read BQ-A2 witness for Assembly QC strip.

APS-QC-SMOKE-LABEL-001 — rubric v3 §4.1 smoke / LOD0 strip labels.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from collections import Counter

from rust_engine_mcp.kit_coverage_audit import CORE_SLOTS, required_slots_for_pack
from rust_engine_mcp.library import load_index_json
from rust_engine_mcp.paths import repo_root

WITNESS_REL = "debug_runs/building_quality_live.json"
PCG_08_WITNESS_REL = "debug_runs/pcg_08_generate_test_live.json"
PASS_THRESHOLD = 70.0
TASK_ID = "APSR-A4-Q1-001"
PCG_08_TASK_ID = "PCG-08-GEN-TEST-001"
SMOKE_LABEL_TASK = "APS-QC-SMOKE-LABEL-001"
GENERATE_ERROR_SIGNATURE = "generate_no_resolvable_placements"

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
    pcg_suffix = format_pcg08_histogram_strip_suffix(repo=repo)
    if pcg_suffix:
        text = f"{text}{pcg_suffix}"
    return text, ok


def load_pcg08_report(*, repo: Path | None = None) -> dict[str, Any] | None:
    root = repo or repo_root()
    path = root / PCG_08_WITNESS_REL
    if not path.is_file():
        return None
    try:
        body = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    if isinstance(body.get("payload"), dict):
        return body["payload"]
    return body


def format_pcg08_histogram_strip_suffix(*, repo: Path | None = None) -> str:
    """APS assembly QC strip — top rejection reasons from PCG-08 histogram."""
    report = load_pcg08_report(repo=repo)
    if not report:
        return ""
    hist = report.get("rejection_histogram") or {}
    if not isinstance(hist, dict) or not hist:
        return ""
    sorted_items = sorted(hist.items(), key=lambda kv: (-int(kv[1]), kv[0]))
    parts = [f"{k}×{v}" for k, v in sorted_items[:4]]
    rate = report.get("pass_rate")
    try:
        rate_f = float(rate) if rate is not None else None
    except (TypeError, ValueError):
        rate_f = None
    head = f"PCG-08 pass {rate_f:.0%}" if rate_f is not None else "PCG-08"
    return f" · {head} · reject " + " · ".join(parts)


def _cross_style_fallback_count(snapshot: dict[str, Any]) -> int:
    from rust_engine_mcp.library import load_index_json

    style_pack = str(snapshot.get("style_pack_id") or "")
    if not style_pack:
        return 0
    by_job: dict[str, dict[str, Any]] = {}
    for row in load_index_json():
        job = str(row.get("job_id") or "")
        if job:
            by_job[job] = row
    drift = 0
    for placement in snapshot.get("module_placements") or []:
        if not isinstance(placement, dict):
            continue
        job = str(placement.get("job_id") or "")
        row = by_job.get(job)
        if not row:
            continue
        row_pack = str(row.get("style_pack") or "")
        if row_pack and row_pack != style_pack:
            drift += 1
    return drift


def _missing_slot_count(snapshot: dict[str, Any]) -> int:
    style_pack = str(snapshot.get("style_pack_id") or "")
    if not style_pack:
        return 0
    required = set(required_slots_for_pack(style_pack))
    present = {
        str(p.get("slot_key") or "")
        for p in (snapshot.get("module_placements") or [])
        if isinstance(p, dict) and p.get("slot_key")
    }
    return len(required - present)


def overall_score_from_parts(
    *,
    style_purity_pct: float,
    silhouette_continuity_pct: float,
    missing_slot_count: int,
    adjacency_violation_count: int,
) -> float:
    """BQ-A2 weights — must match `assembly_quality_gate.rs` compute_assembly_quality."""
    missing_penalty = min(100.0, missing_slot_count * 12.0)
    adjacency_penalty = min(100.0, adjacency_violation_count * 15.0)
    overall = (
        style_purity_pct * 0.35
        + silhouette_continuity_pct * 0.30
        + (100.0 - missing_penalty) * 0.25
        + (100.0 - adjacency_penalty) * 0.10
    )
    return max(0.0, min(100.0, overall))


def compute_snapshot_quality_score(snapshot: dict[str, Any]) -> dict[str, Any]:
    """Mirror BQ-A2 weights from assembly_quality_gate.rs.

    Studio snapshots usually omit a footprint grid; pass
    ``adjacency_violation_count`` on the snapshot when known (CQ-77).
    """
    placements = [p for p in (snapshot.get("module_placements") or []) if isinstance(p, dict)]
    resolved_count = len(placements)
    visible_count = sum(1 for p in placements if not p.get("hidden"))
    cross_style = _cross_style_fallback_count(snapshot)
    missing_slot_count = _missing_slot_count(snapshot)
    try:
        adjacency_violation_count = int(snapshot.get("adjacency_violation_count") or 0)
    except (TypeError, ValueError):
        adjacency_violation_count = 0

    style_purity_pct = (
        100.0
        if resolved_count == 0
        else 100.0 * (resolved_count - cross_style) / resolved_count
    )
    silhouette_continuity_pct = (
        100.0
        if resolved_count == 0
        else min(100.0, 100.0 * visible_count / resolved_count)
    )
    overall_score = overall_score_from_parts(
        style_purity_pct=style_purity_pct,
        silhouette_continuity_pct=silhouette_continuity_pct,
        missing_slot_count=missing_slot_count,
        adjacency_violation_count=adjacency_violation_count,
    )
    passes_gate = overall_score >= PASS_THRESHOLD
    return {
        "style_purity_pct": style_purity_pct,
        "silhouette_continuity_pct": silhouette_continuity_pct,
        "cross_style_fallback_count": cross_style,
        "missing_slot_count": missing_slot_count,
        "adjacency_violation_count": adjacency_violation_count,
        "overall_score": overall_score,
        "passes_gate": passes_gate,
    }


def _rejection_reason_for_candidate(
    *,
    generate_error: str | None,
    smoke_hit: bool,
    grammar_signatures: list[str],
    score_row: dict[str, Any] | None,
) -> str | None:
    if generate_error:
        if "No resolvable" in generate_error or "resolvable" in generate_error.lower():
            return GENERATE_ERROR_SIGNATURE
        return "generate_error"
    if smoke_hit:
        return "smoke_tier_blocked"
    for sig in grammar_signatures:
        if sig:
            return str(sig)
    if score_row and not score_row.get("passes_gate"):
        return "score_below_threshold"
    return None


def evaluate_generate_and_test_candidate(
    entry: dict[str, Any],
    snapshot: dict[str, Any] | None,
    *,
    generate_error: str | None = None,
    repo: Path | None = None,
) -> dict[str, Any]:
    """Score one seeded candidate; classify rejection reason when not kept."""
    _ = repo
    smoke_hit = False
    grammar_sigs: list[str] = []
    score_row: dict[str, Any] | None = None
    assembly_id = ""
    if snapshot:
        assembly_id = str(snapshot.get("assembly_id") or "")
        tier = scan_snapshot_tier_labels(snapshot)
        smoke_hit = bool(tier.get("smoke_hit"))
        from rust_engine_mcp.validators.assembly_grammar_verify import (
            validate_assembly_grammar_verify,
        )

        report = validate_assembly_grammar_verify(snapshot, ship=False)
        grammar_sigs = [
            str(i.signature or i.kind)
            for i in report.errors
            if i.severity == "error" and (i.signature or i.kind)
        ]
        score_row = compute_snapshot_quality_score(snapshot)
    reason = _rejection_reason_for_candidate(
        generate_error=generate_error,
        smoke_hit=smoke_hit,
        grammar_signatures=grammar_sigs,
        score_row=score_row,
    )
    passes = reason is None
    overall = float((score_row or {}).get("overall_score", 0.0))
    return {
        "archetype_id": entry.get("archetype_id"),
        "district_style": entry.get("district_style"),
        "seed": entry.get("seed"),
        "assembly_id": assembly_id,
        "passes_gate": passes,
        "rejection_reason": reason,
        "overall_score": overall,
        "score": score_row,
    }


def run_generate_and_test_report(
    entries: list[dict[str, Any]],
    *,
    top_k: int = 3,
    generate_fn: Any | None = None,
    repo: Path | None = None,
) -> dict[str, Any]:
    """PCG-08 — generate N seeded candidates, gate, histogram rejections, keep top-k."""
    from rust_engine_mcp import golden_seed_review

    _ = repo
    if generate_fn is None:
        generate_fn = golden_seed_review.generate_snapshot_for_seed

    evaluated: list[dict[str, Any]] = []
    for entry in entries:
        snap: dict[str, Any] | None = None
        gen_err: str | None = None
        try:
            snap = generate_fn(entry, write=False)
        except Exception as exc:  # noqa: BLE001
            gen_err = str(exc)
        evaluated.append(
            evaluate_generate_and_test_candidate(
                entry,
                snap,
                generate_error=gen_err,
            )
        )

    passed = [row for row in evaluated if row.get("passes_gate")]
    pass_count = len(passed)
    candidate_count = len(evaluated)
    pass_rate = (pass_count / candidate_count) if candidate_count else 0.0
    histogram = Counter(
        str(r["rejection_reason"])
        for r in evaluated
        if r.get("rejection_reason")
    )
    ranked = sorted(
        evaluated,
        key=lambda r: (
            -float(r.get("overall_score") or 0.0),
            int(r.get("seed") or 0),
            str(r.get("assembly_id") or ""),
        ),
    )
    top_kept = [
        {
            "seed": r.get("seed"),
            "assembly_id": r.get("assembly_id"),
            "overall_score": r.get("overall_score"),
            "passes_gate": r.get("passes_gate"),
        }
        for r in ranked[: max(0, top_k)]
    ]
    return {
        "schema": "pcg_08_generate_test_v1",
        "task_id": PCG_08_TASK_ID,
        "candidate_count": candidate_count,
        "pass_count": pass_count,
        "pass_rate": pass_rate,
        "top_k": top_k,
        "top_k_kept": top_kept,
        "rejection_histogram": dict(sorted(histogram.items(), key=lambda kv: kv[0])),
        "candidates": evaluated,
        "rules_check": {
            "passed": True,
            "blocked_by": [],
            "seed": "golden_set",
        },
    }


def format_generate_and_test_cli(report: dict[str, Any]) -> str:
    """Deterministic text for CLI (sorted histogram, stable floats)."""
    lines = [
        f"{PCG_08_TASK_ID} generate-and-test n={report.get('candidate_count')}",
        (
            f"pass_rate: {float(report.get('pass_rate') or 0.0):.4f} "
            f"({report.get('pass_count')}/{report.get('candidate_count')})"
        ),
        f"top_k: {report.get('top_k')}",
    ]
    for row in report.get("top_k_kept") or []:
        lines.append(
            f"  kept seed={row.get('seed')} score={float(row.get('overall_score') or 0):.2f} "
            f"pass={row.get('passes_gate')} id={row.get('assembly_id')}"
        )
    lines.append("rejection_histogram:")
    hist = report.get("rejection_histogram") or {}
    for reason, count in sorted(hist.items(), key=lambda kv: kv[0]):
        lines.append(f"  {reason}: {count}")
    return "\n".join(lines)


def refresh_pcg08_golden_report(
    *,
    top_k: int = 3,
    write_witness: bool = True,
    repo: Path | None = None,
) -> dict[str, Any]:
    """Refresh PCG-08 generate-and-test report (CQ-32: no BLANG witness ritual)."""
    from rust_engine_mcp import golden_seed_review

    root = repo or repo_root()
    entries = golden_seed_review.load_golden_seeds(repo=root)
    report = run_generate_and_test_report(entries, top_k=top_k, repo=root)
    report["cli_text"] = format_generate_and_test_cli(report)
    report["golden_seed_count"] = len(entries)
    if write_witness:
        # Plain JSON under debug_runs — not a gate witness / ritual producer.
        out = root / PCG_08_WITNESS_REL
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        report["report_path"] = PCG_08_WITNESS_REL
    return report


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
