"""DES-STYLE-LANDSCAPE-RIparian-001 — riparian/agri style bible witness."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.schemas import load_json_file

WITNESS_REL = "debug_runs/art_pipeline/dmcp_style_landscape_riparian_live.json"
GATE_ID = "DES-STYLE-LANDSCAPE-RIparian-001"
DOC_REL = "src/dev/design_style_landscape_riparian_v1.md"
PRESET_REL = "assets/configs/landscape/presets/agri_riparian_v0.json"
DNA_REL = "tools/mcp/schemas/examples/landscape_dna_agri_riparian_v0.json"
BURN_LANG_REL = "design_veg_burn_visual_language_v1.md"

REQUIRED_TOPOLOGIES = (
    "CORRIDOR_RIPARIAN",
    "RING_SHELTERBELT",
    "PATCH_IRREGULAR",
    "FRINGE_EDGE",
)


def run_riparian_style_audit(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    doc_path = root / DOC_REL
    text = doc_path.read_text(encoding="utf-8") if doc_path.is_file() else ""
    preset = load_json_file(root / PRESET_REL) if (root / PRESET_REL).is_file() else {}
    required = list((preset.get("landscape_program") or {}).get("required_topologies") or [])
    checks = {
        "doc_on_disk": doc_path.is_file(),
        "gate_id": GATE_ID in text,
        "preset_ref": "agri_riparian_v0" in text,
        "chart_ref": "AGRI-LANDSCAPE-Δ9" in text or "AGRI-LANDSCAPE" in text,
        "canopy_mass_section": "## 1. Canopy mass" in text,
        "edge_softness_section": "## 2. Edge softness" in text,
        "burn_read_section": "## 3. Burn" in text,
        "palette_section": "## 4. Palette" in text,
        "burn_cross_ref": BURN_LANG_REL.replace(".md", "") in text,
        "three_concept_refs": "**R1**" in text and "**R2**" in text and "**R3**" in text,
        "preset_on_disk": (root / PRESET_REL).is_file(),
        "dna_example_on_disk": (root / DNA_REL).is_file(),
        "preset_id_match": preset.get("preset_id") == "agri_riparian_v0",
        "topologies_in_preset": all(t in required for t in REQUIRED_TOPOLOGIES),
    }
    green = all(checks.values())
    return {
        "gate": GATE_ID,
        "deliverable": DOC_REL,
        "deliverable_exists": checks["doc_on_disk"],
        "status": "done" if green else "open",
        "verdict": "PASS" if green else "FAIL",
        "checks": checks,
        "preset_id": preset.get("preset_id"),
        "required_topologies": required,
        "audit_complete": True,
        "green": green,
        "handoff": {
            "coder_mcp": "VEG-DISTRICT-HYDRO-001 · LG-5 corridor atlas rows",
            "designer_mcp": "G4 topology_corridor_regrowth_grass manual still",
        },
    }


def refresh_dmcp_style_landscape_riparian_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    body = run_riparian_style_audit(repo=root)
    body["_agent_meta"] = {
        "schema": "dmcp_style_landscape_riparian_live_v1",
        "written_at_epoch_secs": int(time.time()),
        "profile": "DES_STYLE_LANDSCAPE_RIPARIAN",
        "source_system": "dmcp_style_landscape_riparian",
        "relative_path": WITNESS_REL,
        "ritual": f"BLANG:WIT-HON→Q✓ {GATE_ID}" if body.get("green") else "BLANG:WIT-HON FAIL",
        "agent": "designer-mcp",
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = WITNESS_REL
    return body
