"""DMCP open designer-mcp lane — veg atlas ship criteria + building atlas QC copy v2."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.schemas import load_json_file

WITNESS_REL = "debug_runs/art_pipeline/dmcp_designer_mcp_open_lane_live.json"
GATE_ID = "DMCP-DESIGNER-MCP-OPEN-LANE-001"

VEG_DOC = "src/dev/dmcp_veg_atlas_ship_v1.md"
VEG_SPEC = "assets/staging/specs/veg_atlas_ship_001.json"
VEG_G0 = "debug_runs/art_pipeline/landscape_expanded_g0_rules.yaml"
VEG_G4 = "debug_runs/art_pipeline/landscape_expanded_g4_signoff.yaml"
VEG_BATCH = "assets/staging/specs/tile_batch_landscape_expanded_v1.json"
VEG_KEYFRAMES = "assets/staging/tiles/keyframe_stills/tile_landscape_expanded_v1"
LG5_QC_DOC = "src/dev/design_landscape_lg5_keyframe_qc_v1.md"

PLAIN_DOC = "src/dev/design_aps_atlas_qc_copy_buildings_v2.md"
PLAIN_BATCHES = (
    "tile_warehouse_industrial_west_production_v1",
    "tile_shopfront_colonial_production_v1",
    "tile_bunker_military_production_v1",
)

LANE_SLICES: tuple[dict[str, str], ...] = (
    {"id": "DMCP-VEG-ATLAS-SHIP-001", "deliverable": VEG_DOC, "spec": VEG_SPEC},
    {"id": "DMCP-ATLAS-QC-PLAIN-002", "deliverable": PLAIN_DOC},
)

G4_MINIMUM = (
    "topology_patch_burn_04",
    "topology_patch_scar",
    "topology_corridor_regrowth_grass",
)


def _parse_g4_proceed_ship(text: str) -> str | None:
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith("proceed_ship:"):
            return stripped.split(":", 1)[1].strip()
    return None


def _check_veg_atlas_ship(root: Path) -> dict[str, Any]:
    spec = load_json_file(root / VEG_SPEC) if (root / VEG_SPEC).is_file() else {}
    batch = load_json_file(root / VEG_BATCH) if (root / VEG_BATCH).is_file() else {}
    g0_text = (root / VEG_G0).read_text(encoding="utf-8") if (root / VEG_G0).is_file() else ""
    g4_text = (root / VEG_G4).read_text(encoding="utf-8") if (root / VEG_G4).is_file() else ""
    keyframe_dir = root / VEG_KEYFRAMES
    variant_keys = [
        str(v.get("variant_key")) for v in batch.get("variants") or [] if v.get("variant_key")
    ]
    png_count = sum(1 for k in variant_keys if (keyframe_dir / f"{k}.png").is_file())
    g4_minimum_in_spec = list(spec.get("g4_minimum_review") or [])
    proceed_ship = _parse_g4_proceed_ship(g4_text)

    checks = {
        "dmcp_doc": (root / VEG_DOC).is_file(),
        "spec_on_disk": (root / VEG_SPEC).is_file(),
        "g0_rules": (root / VEG_G0).is_file(),
        "g4_signoff_template": (root / VEG_G4).is_file(),
        "lg5_qc_prerequisite": (root / LG5_QC_DOC).is_file(),
        "gate_id_match": spec.get("gate") == "DMCP-VEG-ATLAS-SHIP-001",
        "ship_false_honest": spec.get("ship") is False and batch.get("ship") is False,
        "spec_only": spec.get("spec_only") is True,
        "variant_count_16": int(spec.get("variant_count") or 0) == 16,
        "g4_minimum_set": g4_minimum_in_spec == list(G4_MINIMUM),
        "keyframes_16_on_disk": png_count == 16,
        "proceed_ship_honest_no": proceed_ship == "no",
        "g0_proceed_tile_ship_no": "proceed_tile_ship: no" in g0_text,
    }
    green = all(checks.values())
    return {
        "green": green,
        "verdict": "PASS_WITH_NOTES" if green else "FAIL",
        "checks": checks,
        "png_count": png_count,
        "proceed_ship": proceed_ship,
        "g4_gap": "topology_corridor_regrowth_grass",
    }


def _check_atlas_qc_plain_002(root: Path) -> dict[str, Any]:
    doc_path = root / PLAIN_DOC
    text = doc_path.read_text(encoding="utf-8") if doc_path.is_file() else ""
    checks = {
        "doc_on_disk": doc_path.is_file(),
        "gate_id": "DMCP-ATLAS-QC-PLAIN-002" in text,
        "warehouse_batch_ref": "tile_warehouse_industrial_west_production_v1" in text,
        "shopfront_batch_ref": "tile_shopfront_colonial_production_v1" in text,
        "bunker_batch_ref": "tile_bunker_military_production_v1" in text,
        "footprint_messages": all(
            token in text for token in ("warehouse_footprint", "shopfront_footprint", "bunker_footprint")
        ),
        "burn_damage_gaps": "burn_frame_gap" in text and "damage_frame_gap" in text,
        "pilot_ship_false_note": "ship: false" in text or "ship:false" in text,
        "supersedes_v1_note": "DMCP-ATLAS-QC-PLAIN-001" in text,
    }
    batch_examples = {
        bid: (
            root / "tools/mcp/schemas/examples" / f"tile_batch_{bid.replace('tile_', '')}.json"
        ).is_file()
        for bid in PLAIN_BATCHES
    }
    checks["batch_examples_on_disk"] = all(batch_examples.values())
    green = all(checks.values())
    return {
        "green": green,
        "verdict": "PASS" if green else "FAIL",
        "checks": checks,
        "batch_examples": batch_examples,
    }


def run_open_lane_audit(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    auditors = {
        "DMCP-VEG-ATLAS-SHIP-001": _check_veg_atlas_ship,
        "DMCP-ATLAS-QC-PLAIN-002": _check_atlas_qc_plain_002,
    }
    rows: list[dict[str, Any]] = []
    for slice_row in LANE_SLICES:
        gate_id = slice_row["id"]
        audit = auditors[gate_id](root)
        rows.append(
            {
                "id": gate_id,
                "deliverable": slice_row["deliverable"],
                "deliverable_exists": (root / slice_row["deliverable"]).is_file(),
                "status": "done" if audit.get("green") else "open",
                "verdict": audit.get("verdict"),
                "audit": audit,
            }
        )
    done_count = sum(1 for r in rows if r["status"] == "done")
    all_green = done_count == len(rows)
    overall_verdict = "PASS_WITH_NOTES" if all_green else "FAIL"
    return {
        "gate": GATE_ID,
        "lanes": [r["id"] for r in rows],
        "slice_count": len(rows),
        "done_count": done_count,
        "rows": rows,
        "audit_complete": True,
        "green": all_green,
        "verdict": overall_verdict,
        "handoff": {
            "veg_ship_flip": "operator G4 manual → landscape_expanded_g4_signoff.yaml proceed_ship:yes",
            "veg_engine": "VEG-F01-ATLAS-SHIP-001 @coder_a",
            "buildings_qc": "wire copy into aps_atlas_qc.py _PLAIN map @coder-mcp",
        },
    }


def refresh_dmcp_designer_mcp_open_lane_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    body = run_open_lane_audit(repo=root)
    body["_agent_meta"] = {
        "schema": "dmcp_designer_mcp_open_lane_live_v1",
        "written_at_epoch_secs": int(time.time()),
        "profile": "DMCP_DESIGNER_MCP_OPEN",
        "source_system": "dmcp_designer_mcp_open_lane",
        "relative_path": WITNESS_REL,
        "ritual": "BLANG:WIT-HON→Q✓ DMCP-OPEN-LANE" if body.get("green") else "BLANG:WIT-HON FAIL",
        "agent": "designer-mcp",
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = WITNESS_REL
    return body
