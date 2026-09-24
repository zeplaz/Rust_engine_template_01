"""APSR-Q3 — golden-seed browse/approve flow (BQ-Q3 consumer).

APS-GOLDEN-RUBRIC-OPS-001 / BQ-Q3-OPS-APPROVE-001: machine may *scaffold* a
pending rubric sheet; only a human operator may write approve/reject that counts
as operator_pass. Pytest theater (`note=pytest`) never counts.
"""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

BQ_Q3_WITNESS = "debug_runs/bq_q3_golden_001_live.json"
RUBRIC_ROWS_REL = "debug_runs/aps_golden_seed_rubric_rows.json"
SCAFFOLD_WITNESS_REL = "debug_runs/aps_golden_rubric_sheet_scaffold_live.json"
RUBRIC_REF_V2 = "src/dev/design_aps_operator_rubric_v2.md#BQ-Q3"
RUBRIC_REF_V3 = "src/dev/design_aps_operator_rubric_v3.md#BQ-Q3"
CRITERION = "reads as a real building"
MIN_SHEET_ROWS = 12


def load_golden_seeds(*, repo: Path | None = None) -> list[dict[str, Any]]:
    root = repo or repo_root()
    path = root / BQ_Q3_WITNESS
    if not path.is_file():
        return []
    body = json.loads(path.read_text(encoding="utf-8"))
    seeds = body.get("seeds") or []
    return [s for s in seeds if isinstance(s, dict)]


def load_rubric_rows(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    path = root / RUBRIC_ROWS_REL
    if not path.is_file():
        return {"rows": [], "version": 1}
    return json.loads(path.read_text(encoding="utf-8"))


def seed_key(entry: dict[str, Any]) -> str:
    return f"{entry.get('archetype_id')}:{entry.get('district_style')}:s{entry.get('seed')}"


def _rubric_ref_for_repo(root: Path) -> str:
    if (root / "src/dev/design_aps_operator_rubric_v3.md").is_file():
        return RUBRIC_REF_V3
    return RUBRIC_REF_V2


def is_pytest_theater(row: dict[str, Any]) -> bool:
    """True when the row is machine/pytest approve theater — not operator eyes."""
    note = str(row.get("note") or "").strip().lower()
    by = str(row.get("recorded_by") or "").strip().lower()
    if note == "pytest":
        return True
    if "pytest" in note and row.get("verdict") == "approve":
        return True
    if by in ("pytest", "coder-mcp-machine", "machine"):
        return True
    return False


def is_operator_verdict(row: dict[str, Any]) -> bool:
    """Human approve/reject that may count toward BQ-Q3 / APS-GOLDEN operator pass."""
    if is_pytest_theater(row):
        return False
    return row.get("verdict") in ("approve", "reject")


def operator_pass_status(rows: list[dict[str, Any]]) -> dict[str, Any]:
    """Never true from scaffold alone — requires ≥12 non-pytest operator verdicts."""
    op_rows = [r for r in rows if is_operator_verdict(r)]
    pending = [r for r in rows if r.get("verdict") == "pending_operator"]
    theater = [r for r in rows if is_pytest_theater(r)]
    passed = len(op_rows) >= MIN_SHEET_ROWS and len(theater) == 0 and len(pending) == 0
    return {
        "operator_pass": passed,
        "operator_verdict_count": len(op_rows),
        "pending_operator_count": len(pending),
        "pytest_theater_count": len(theater),
        "min_required": MIN_SHEET_ROWS,
    }


def _pending_row_for_seed(entry: dict[str, Any], *, rubric_ref: str) -> dict[str, Any]:
    return {
        "seed_key": seed_key(entry),
        "archetype_id": entry.get("archetype_id"),
        "district_style": entry.get("district_style"),
        "seed": entry.get("seed"),
        "expected_hash": entry.get("expected_hash"),
        "criterion": CRITERION,
        "verdict": "pending_operator",
        "screen_pass": False,
        "note": (
            "Scaffolded sheet — operator/designer must judge "
            f"'{CRITERION}' (APS-GOLDEN-RUBRIC-OPS-001 / BQ-Q3-OPS-APPROVE-001)."
        ),
        "recorded_at": datetime.now(timezone.utc).isoformat(),
        "rubric_ref": rubric_ref,
        "recorded_by": "coder-mcp-scaffold",
    }


def scaffold_aps_golden_rubric_sheet(
    *,
    demote_pytest: bool = True,
    repo: Path | None = None,
) -> dict[str, Any]:
    """Expand golden-seed rubric sheet to pending_operator rows; never auto-approve.

    Preserves real operator approve/reject. Optionally demotes pytest theater to
    pending_operator so BQ-Q3 eyes are honest.
    """
    root = repo or repo_root()
    seeds = load_golden_seeds(repo=root)
    rubric_ref = _rubric_ref_for_repo(root)
    existing = {str(r.get("seed_key")): r for r in (load_rubric_rows(repo=root).get("rows") or [])}
    rows: list[dict[str, Any]] = []
    demoted = 0
    added = 0
    preserved = 0
    for entry in seeds:
        key = seed_key(entry)
        prev = existing.get(key)
        if prev is not None and is_operator_verdict(prev):
            rows.append(prev)
            preserved += 1
            continue
        if prev is not None and demote_pytest and is_pytest_theater(prev):
            demoted += 1
        elif prev is not None and prev.get("verdict") == "pending_operator":
            # Refresh scaffold metadata but keep pending
            pass
        else:
            added += 1
        rows.append(_pending_row_for_seed(entry, rubric_ref=rubric_ref))
    # Keep orphan non-seed rows that are real operator verdicts (rare)
    seed_keys = {seed_key(s) for s in seeds}
    for key, prev in existing.items():
        if key in seed_keys:
            continue
        if is_operator_verdict(prev):
            rows.append(prev)
            preserved += 1
    out = {
        "version": 2,
        "task_id": "APS-GOLDEN-RUBRIC-OPS-001",
        "criterion": CRITERION,
        "operator_pass": False,
        "scaffold_only": True,
        "rows": rows,
    }
    path = root / RUBRIC_ROWS_REL
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(out, indent=2) + "\n", encoding="utf-8")
    status = operator_pass_status(rows)
    return {
        "rubric_rows_path": RUBRIC_ROWS_REL,
        "row_count": len(rows),
        "added_pending": added,
        "demoted_pytest_theater": demoted,
        "preserved_operator_verdicts": preserved,
        "golden_seed_count": len(seeds),
        "rubric_ref": rubric_ref,
        **status,
    }


def write_aps_golden_rubric_scaffold_witness(*, repo: Path | None = None) -> dict[str, Any]:
    """Machine-green when sheet is scaffolded for eyes; never claims operator pass."""
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    root = repo or repo_root()
    sheet = scaffold_aps_golden_rubric_sheet(demote_pytest=True, repo=root)
    v3_exists = (root / "src/dev/design_aps_operator_rubric_v3.md").is_file()
    # Sheet ready = enough pending/operator rows from golden seeds; ops still open
    sheet_ready = (
        int(sheet.get("row_count") or 0) >= MIN_SHEET_ROWS
        and int(sheet.get("golden_seed_count") or 0) >= MIN_SHEET_ROWS
        and sheet.get("operator_pass") is False
    )
    body: dict[str, Any] = {
        "task_id": "APS-GOLDEN-RUBRIC-SHEET-SCAFFOLD-001",
        "gate": "APS-GOLDEN-RUBRIC-OPS-001",
        "green": sheet_ready,
        "proceed_ship": False,
        "art_quality": "rubric_sheet_pending_operator",
        "operator_pass": False,
        "aps_golden_ops_closed": False,
        "bq_q3_ops_closed": False,
        "designer_rubric_v3_exists": v3_exists,
        "next_owners": ["designer", "operator"],
        "unblock": [
            "APS-GOLDEN-RUBRIC-OPS-001 — @designer charter design_aps_operator_rubric_v3.md",
            "BQ-Q3-OPS-APPROVE-001 — @operator display session; verdicts ≠ pytest",
        ],
        "plan_ref": "tools/orchestrator/queues/mcp_aps_tooling_finish_queue.json#APS-GOLDEN-RUBRIC-OPS-001",
        **sheet,
    }
    return write_aps_live_witness(
        body,
        SCAFFOLD_WITNESS_REL,
        schema="aps_golden_rubric_sheet_scaffold_v1",
        profile="APS_GOLDEN_RUBRIC_SHEET",
        source_system="aps_golden_rubric_scaffold",
        ritual="BLANG:WIT-HON APS-GOLDEN-RUBRIC-SHEET-SCAFFOLD-001" if sheet_ready else None,
        exit_predicate_must=[
            {"field": "row_count", "gte": MIN_SHEET_ROWS},
            {"field": "operator_pass", "equals": False},
            {"field": "aps_golden_ops_closed", "equals": False},
        ],
        repo=root,
    )


def record_seed_verdict(
    entry: dict[str, Any],
    *,
    verdict: str,
    note: str = "",
    repo: Path | None = None,
) -> dict[str, Any]:
    """Approve/reject writes operator rubric row (design_aps_operator_rubric_v2/v3)."""
    root = repo or repo_root()
    data = load_rubric_rows(repo=root)
    rows: list[dict[str, Any]] = list(data.get("rows") or [])
    key = seed_key(entry)
    row = {
        "seed_key": key,
        "archetype_id": entry.get("archetype_id"),
        "district_style": entry.get("district_style"),
        "seed": entry.get("seed"),
        "expected_hash": entry.get("expected_hash"),
        "criterion": CRITERION,
        "verdict": verdict,
        "note": note,
        "recorded_at": datetime.now(timezone.utc).isoformat(),
        "rubric_ref": _rubric_ref_for_repo(root),
        "recorded_by": "operator" if note.strip().lower() != "pytest" else "pytest",
    }
    rows = [r for r in rows if r.get("seed_key") != key]
    rows.append(row)
    status = operator_pass_status(rows)
    out = {
        "version": 2,
        "task_id": "APS-GOLDEN-RUBRIC-OPS-001",
        "criterion": CRITERION,
        "operator_pass": status["operator_pass"],
        "scaffold_only": not status["operator_pass"],
        "rows": rows,
    }
    path = root / RUBRIC_ROWS_REL
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(out, indent=2) + "\n", encoding="utf-8")
    return row


def generate_snapshot_for_seed(entry: dict[str, Any], *, write: bool = False) -> dict[str, Any]:
    from rust_engine_mcp import assembly

    return assembly.generate_assembly_snapshot(
        archetype_id=str(entry["archetype_id"]),
        district_style=str(entry["district_style"]),
        seed=int(entry["seed"]),
        source_tier="lod0",
        write=write,
    )


def write_apsr_q3_witness(*, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    root = repo or repo_root()
    seeds = load_golden_seeds(repo=root)
    bq_path = root / BQ_Q3_WITNESS
    panel_path = root / "tools/mcp/art_pipeline_suite/golden_seed_review_panel.py"
    rubric_path = root / RUBRIC_ROWS_REL
    bq_green = False
    if bq_path.is_file():
        bq_green = bool(json.loads(bq_path.read_text(encoding="utf-8")).get("green"))
    green = bq_green and panel_path.is_file() and len(seeds) >= 12
    body: dict[str, Any] = {
        "task_id": "APSR-A4-Q3-001",
        "gate": "APSR-A4-Q3-001",
        "green": green,
        "golden_seed_count": len(seeds),
        "bq_q3_witness": BQ_Q3_WITNESS,
        "rubric_rows_path": RUBRIC_ROWS_REL,
        "rubric_row_count": len(load_rubric_rows(repo=root).get("rows") or []),
        "plan_ref": "src/dev/plan_aps_refactor_v1.md#APSR-Q3",
    }
    if not rubric_path.is_file():
        rubric_path.parent.mkdir(parents=True, exist_ok=True)
        rubric_path.write_text(json.dumps({"version": 1, "rows": []}, indent=2), encoding="utf-8")
    return write_aps_live_witness(
        body,
        "debug_runs/apsr_a4_q3_001_live.json",
        schema="apsr_a4_q3_live_v1",
        profile="APSR_A4_Q3",
        source_system="apsr_a4_q3",
        ritual="BLANG:WIT-HON APSR-A4-Q3-001" if green else None,
        repo=root,
    )
