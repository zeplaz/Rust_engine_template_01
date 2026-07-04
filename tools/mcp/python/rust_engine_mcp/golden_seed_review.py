"""APSR-Q3 — golden-seed browse/approve flow (BQ-Q3 consumer)."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

BQ_Q3_WITNESS = "debug_runs/bq_q3_golden_001_live.json"
RUBRIC_ROWS_REL = "debug_runs/aps_golden_seed_rubric_rows.json"


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


def record_seed_verdict(
    entry: dict[str, Any],
    *,
    verdict: str,
    note: str = "",
    repo: Path | None = None,
) -> dict[str, Any]:
    """Approve/reject writes operator rubric row (design_aps_operator_rubric_v2)."""
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
        "verdict": verdict,
        "note": note,
        "recorded_at": datetime.now(timezone.utc).isoformat(),
        "rubric_ref": "src/dev/design_aps_operator_rubric_v2.md#BQ-Q3",
    }
    rows = [r for r in rows if r.get("seed_key") != key]
    rows.append(row)
    out = {"version": 1, "rows": rows}
    path = root / RUBRIC_ROWS_REL
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(out, indent=2), encoding="utf-8")
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
