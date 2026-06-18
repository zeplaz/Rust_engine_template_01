"""DMCP veg wave tail — reconcile queue rows vs on-disk deliverables."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

WITNESS_REL = "debug_runs/art_pipeline/dmcp_veg_wave_tail_reconcile_live.json"
WAVE0_WITNESS_REL = "debug_runs/art_pipeline/designer_mcp_parallel_wave0_live.json"

TAIL_SLICES: tuple[dict[str, str], ...] = (
    {
        "id": "DMCP-TILE-BATCH-EXPAND-SPEC-001",
        "deliverable": "assets/staging/specs/tile_batch_landscape_expanded_v1.json",
        "witness": "debug_runs/art_pipeline/landscape_expanded_g0_rules.yaml",
    },
    {
        "id": "DMCP-CONTENT-G0-RULES-001",
        "deliverable": "debug_runs/art_pipeline/landscape_expanded_g0_rules.yaml",
        "witness": "debug_runs/art_pipeline/landscape_expanded_g0_rules.yaml",
    },
    {
        "id": "DMCP-SUCCESSION-STATE-CONTENT-001",
        "deliverable": "src/dev/design_veg_succession_state_content_v1.md",
    },
    {
        "id": "DMCP-BURN-VISUAL-LANG-001",
        "deliverable": "src/dev/design_veg_burn_visual_language_v1.md",
    },
    {
        "id": "DMCP-PRESET-DISPLAY-STRINGS-001",
        "deliverable": "assets/configs/landscape/presets/_display_strings_v1.json",
    },
    {
        "id": "DMCP-ATLAS-QC-PLAIN-001",
        "deliverable": "src/dev/design_aps_atlas_qc_copy_v1.md",
    },
    {
        "id": "DMCP-G4-STAGING-SIGN-001",
        "deliverable": "src/dev/design_landscape_g4_staging_sign_v1.md",
    },
    {
        "id": "DMCP-PILOT-TEACH-ANNOT-001",
        "deliverable": "assets/staging/tiles/tile_landscape_lg5_pilot_v1/batch_status.json",
        "pilot_spec": "tools/mcp/schemas/examples/tile_batch_landscape_lg5_pilot_v1.json",
    },
)


def _load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def verify_pilot_teach_annotation(*, repo: Path) -> dict[str, Any]:
    status_path = repo / "assets/staging/tiles/tile_landscape_lg5_pilot_v1/batch_status.json"
    spec_path = repo / "tools/mcp/schemas/examples/tile_batch_landscape_lg5_pilot_v1.json"
    meta_path = repo / "assets/staging/tiles/tile_landscape_lg5_pilot_v1/atlas_meta.json"
    checks: dict[str, bool] = {
        "batch_status_on_disk": status_path.is_file(),
        "pilot_spec_on_disk": spec_path.is_file(),
        "atlas_meta_on_disk": meta_path.is_file(),
    }
    if status_path.is_file():
        body = _load_json(status_path)
        checks["batch_status_not_a_ship_target"] = body.get("not_a_ship_target") is True
        checks["batch_status_teach_gate"] = body.get("gate") == "DMCP-PILOT-TEACH-ANNOT-001"
    if spec_path.is_file():
        meta = _load_json(spec_path).get("_meta") or {}
        checks["spec_not_a_ship_target"] = meta.get("not_a_ship_target") is True
    if meta_path.is_file():
        meta_body = _load_json(meta_path)
        checks["atlas_meta_not_a_ship_target"] = meta_body.get("not_a_ship_target") is True
    green = all(checks.values())
    return {"green": green, "checks": checks}


def run_veg_wave_tail_reconcile(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    rows: list[dict[str, Any]] = []
    all_green = True
    for spec in TAIL_SLICES:
        gate = spec["id"]
        deliverable = root / spec["deliverable"]
        row: dict[str, Any] = {
            "id": gate,
            "deliverable": spec["deliverable"],
            "deliverable_exists": deliverable.is_file(),
            "status": "done" if deliverable.is_file() else "fail",
        }
        if "witness" in spec:
            row["witness"] = spec["witness"]
            row["witness_exists"] = (root / spec["witness"]).is_file()
            if not row["witness_exists"]:
                row["status"] = "fail"
        if "pilot_spec" in spec:
            row["pilot_spec"] = spec["pilot_spec"]
            row["pilot_spec_exists"] = (root / spec["pilot_spec"]).is_file()
            pilot = verify_pilot_teach_annotation(repo=root)
            row["pilot_teach"] = pilot
            if not pilot.get("green"):
                row["status"] = "fail"
        if row["status"] != "done":
            all_green = False
        rows.append(row)

    return {
        "gate": "DMCP-VEG-WAVE-TAIL-RECONCILE-001",
        "slice_count": len(rows),
        "done_count": sum(1 for r in rows if r["status"] == "done"),
        "rows": rows,
        "audit_complete": True,
        "green": all_green,
        "verdict": "PASS" if all_green else "FAIL",
        "queue": "tools/orchestrator/queues/parallel_wave_aps_veg_dispatch_v1.json",
        "wave0_witness": WAVE0_WITNESS_REL,
    }


def refresh_dmcp_veg_wave_tail_reconcile_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    body = run_veg_wave_tail_reconcile(repo=root)
    body["_agent_meta"] = {
        "schema": "dmcp_veg_wave_tail_reconcile_live_v1",
        "written_at_epoch_secs": int(time.time()),
        "profile": "DMCP_VEG_WAVE_TAIL",
        "source_system": "dmcp_veg_wave_tail_reconcile",
        "relative_path": WITNESS_REL,
        "ritual": "BLANG:WIT-HON→Q✓ DMCP-VEG-WAVE-TAIL-RECONCILE" if body.get("green") else "BLANG:WIT-HON FAIL",
        "agent": "designer-mcp",
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = WITNESS_REL
    return body
