#!/usr/bin/env python3
"""Mark queue rows done when on-disk witness JSON proves green."""
from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
QUEUES = REPO / "tools/orchestrator/queues"
DEBUG = REPO / "debug_runs"

# task_id -> (witness_path, json_path_or_true_key)
WITNESS_CHECKS: dict[str, tuple[str, str]] = {
    "COD-POWER-ISLAND-HIGHLIGHT-001": ("power_grid_track_bd_live.json", "COD-POWER-ISLAND-HIGHLIGHT-001"),
    "COD-POWER-TOOL-RAIL-001": ("power_grid_track_bd_live.json", "COD-POWER-TOOL-RAIL-001"),
    "COD-UTILITY-ACTIVATION-LINK-001": ("power_grid_track_bd_live.json", "COD-UTILITY-ACTIVATION-LINK-001"),
    "COD-POWER-OVERLAY-RENDER-001": ("power_map_overlay_live.json", "green"),
    "COD-POWER-DAMAGE-SEGMENT-001": ("power_grid_track_c_live.json", "COD-POWER-DAMAGE-SEGMENT-001"),
    "CMCP-GRAM-SWEEP-PROCESS-001": ("grammar_sweep_process_live.json", "green"),
    "APS-TAG-TIER2-IMPL": ("aps_tag_tier2_live.json", "green"),
    "APS-UX-POLISH-TAIL-001": ("aps_ux_polish_tail_live.json", "green"),
    "APS-G4-COVERAGE-001": ("aps_g4_coverage_live.json", "green"),
    "CMCP-FACILITY-NEEDS-PANEL-001": ("art_pipeline/facility_needs_panel_live.json", "green"),
    "CMCP-SITE-PREVIEW-PANEL-001": ("art_pipeline/site_preview_panel_live.json", "green"),
    "CDR-B-VEG-MINIMAP-LEGEND-UI-001": ("minimap_topology_legend_live.json", "green"),
    "DES-POWER-NODE-HOVER-001": ("design_power_node_hover_v1.md", "PASS"),  # spec file marker
    "VEG-F01-ATLAS-SHIP-001": ("veg_ship_close_live.json", "vegetation_program_close"),
    "SIM-STEWARD-FIRE-REGRESS-001": ("fire_ecology_live.json", "green"),
    "VA2-HARNESS-CLOSE-001": ("visual_aidv2_live.json", "green"),
}

QUEUE_FILES = [
    QUEUES / "power_grid_construction_ux_queue.json",
    QUEUES / "multi_parallel_home_queues_v1.json",
    QUEUES / "multi_parallel_tracks_dispatch_v1.json",
    QUEUES / "coder_active_queue.json",
    QUEUES / "grammar_continuation_queue.json",
    QUEUES / "designer_active_queue.json",
    QUEUES / "coder_vegetation_drain_queue.json",
]


def witness_green(task_id: str) -> bool:
    spec = WITNESS_CHECKS.get(task_id)
    if not spec:
        return False
    path_key, field = spec
    if path_key.endswith(".md"):
        text = (REPO / "src" / "dev" / path_key).read_text(encoding="utf-8")
        return field in text
    data = json.loads((DEBUG / path_key).read_text(encoding="utf-8"))
    if task_id == "VA2-HARNESS-CLOSE-001":
        return (
            data.get("green") is True
            and data.get("done") == 6
            and data.get("lib_fixture") is not True
        )
    val = data.get(field)
    if isinstance(val, bool):
        return val
    if task_id in (
        "CMCP-GRAM-SWEEP-PROCESS-001",
        "CMCP-FACILITY-NEEDS-PANEL-001",
        "CMCP-SITE-PREVIEW-PANEL-001",
        "SIM-STEWARD-FIRE-REGRESS-001",
        "APS-TAG-TIER2-IMPL",
        "APS-UX-POLISH-TAIL-001",
        "APS-G4-COVERAGE-001",
    ):
        return data.get("green") is True
    return val is True or data.get(task_id) is True


def iter_rows(obj: dict | list) -> list[dict]:
    rows: list[dict] = []
    if isinstance(obj, list):
        for r in obj:
            if isinstance(r, dict) and "id" in r:
                rows.append(r)
        return rows
    for key in ("drain", "tasks", "p2_tasks", "parallel", "items", "queue", "active"):
        if isinstance(obj.get(key), list):
            for r in obj[key]:
                if isinstance(r, dict) and "id" in r:
                    rows.append(r)
    return rows


def main() -> None:
    now = datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")
    marked = 0
    for qpath in QUEUE_FILES:
        if not qpath.exists():
            continue
        obj = json.loads(qpath.read_text(encoding="utf-8"))
        changed = False
        for row in iter_rows(obj):
            tid = row.get("id", "")
            if row.get("status") in {"done", "closed", "signed"}:
                continue
            if not witness_green(tid):
                continue
            row["status"] = "done"
            row["completed"] = now[:10]
            row["witness_refresh"] = now
            marked += 1
            changed = True
        if changed:
            qpath.write_text(json.dumps(obj, indent=2) + "\n", encoding="utf-8")
            print(f"Updated {qpath.name}")
    print(f"Marked done: {marked}")


if __name__ == "__main__":
    main()
