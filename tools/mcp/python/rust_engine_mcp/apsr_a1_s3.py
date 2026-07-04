"""APSR-A1-S3-001 — LaneChanged shell cleanup witness."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

TASK_ID = "APSR-A1-S3-001"
WITNESS_REL = "debug_runs/apsr_a1_s3_001_live.json"
APP_REL = "tools/mcp/art_pipeline_suite/app.py"
WIRING_REL = "tools/mcp/art_pipeline_suite/aps_shell_wiring.py"
MAX_APP_LOC = 700


def _app_line_count(path: Path) -> int:
    return len(path.read_text(encoding="utf-8").splitlines())


def _shell_sync_from_state_calls(path: Path) -> list[int]:
    hits: list[int] = []
    for line_no, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        stripped = line.strip()
        if "sync_from_state" in stripped and not stripped.startswith("#"):
            hits.append(line_no)
    return hits


def write_apsr_a1_s3_witness(*, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    root = repo or repo_root()
    app_path = root / APP_REL
    wiring_path = root / WIRING_REL
    app_text = app_path.read_text(encoding="utf-8")
    loc = _app_line_count(app_path)
    sync_hits = _shell_sync_from_state_calls(app_path)
    lane_changed_publish = "publish(\"LaneChanged\"" in app_text or "publish('LaneChanged'" in app_text
    send_to_assembly_publish = (
        "publish(\"SendToAssembly\"" in app_text or "publish('SendToAssembly'" in app_text
    )
    green = (
        loc < MAX_APP_LOC
        and sync_hits == []
        and lane_changed_publish
        and send_to_assembly_publish
        and wiring_path.is_file()
    )
    body: dict[str, Any] = {
        "task_id": TASK_ID,
        "gate": TASK_ID,
        "green": green,
        "app_loc": loc,
        "app_loc_max": MAX_APP_LOC,
        "shell_sync_from_state_lines": sync_hits,
        "lane_changed_event": lane_changed_publish,
        "send_to_assembly_event": send_to_assembly_publish,
        "shell_wiring_module": WIRING_REL,
        "plan_ref": "src/dev/plan_aps_refactor_v1.md#APSR-S3",
    }
    return write_aps_live_witness(
        body,
        WITNESS_REL,
        schema="apsr_a1_s3_live_v1",
        profile="APSR_A1_S3",
        source_system="apsr_a1_s3",
        ritual=f"BLANG:WIT-HON {TASK_ID}" if green else None,
        repo=root,
    )
