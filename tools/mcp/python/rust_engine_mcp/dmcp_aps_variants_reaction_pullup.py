"""DMCP-APS-VARIANTS-REACTION-PULLUP-001 — Variants tab confront critique witness."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.dmcp_reaction_territory_events import run_reaction_territory_events_audit
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.reaction_territory import refresh_reaction_territory_witness

WITNESS_REL = "debug_runs/art_pipeline/dmcp_aps_variants_reaction_pullup_live.json"
GATE_ID = "DMCP-APS-VARIANTS-REACTION-PULLUP-001"
CRITIQUE_DOC = "src/dev/dmcp_aps_variants_reaction_pullup_critique_v1.md"
VARIANTS_PANEL = "tools/mcp/art_pipeline_suite/variants_panel.py"
VARIANTS_PREVIEW = "tools/mcp/art_pipeline_suite/variants_preview_panel.py"
CATALOG_JSON = "tools/mcp/schemas/examples/reaction_territory_events_v1.json"


def run_pullup_critique_audit(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    panel_src = (root / VARIANTS_PANEL).read_text(encoding="utf-8") if (root / VARIANTS_PANEL).is_file() else ""
    preview_src = (root / VARIANTS_PREVIEW).read_text(encoding="utf-8") if (root / VARIANTS_PREVIEW).is_file() else ""
    catalog = json.loads((root / CATALOG_JSON).read_text(encoding="utf-8")) if (root / CATALOG_JSON).is_file() else {}
    events = catalog.get("events") or {}

    schema_audit = run_reaction_territory_events_audit(repo=root)
    reaction_witness = refresh_reaction_territory_witness(repo=root)

    checks = {
        "critique_doc": (root / CRITIQUE_DOC).is_file(),
        "variants_panel": (root / VARIANTS_PANEL).is_file(),
        "variants_preview_panel": (root / VARIANTS_PREVIEW).is_file(),
        "reaction_filter_combobox": "_reaction_filter_var" in panel_src and "Combobox" in panel_src,
        "variants_preview_panel_wired": "VariantsPreviewPanel" in panel_src,
        "preview_four_state_strip": "VARIANT_STATES" in preview_src or "variant_state_label" in preview_src,
        "empty_state_label": "empty_state_label" in panel_src,
        "agent_patch_collapsed": "CollapsibleSection" in panel_src and "Agent patch" in panel_src,
        "catalog_event_count_11": len(events) == 11,
        "schema_witness_green": schema_audit.get("green") is True,
        "liquidation_triggers_complete": schema_audit.get("checks", {}).get("liquidation_triggers_complete") is True,
        "reaction_territory_witness_green": reaction_witness.get("green") is True,
        "cmcp_resolve_green": reaction_witness.get("cmcp_resolve_001_green") is True,
        "cmcp_preview_green": reaction_witness.get("cmcp_preview_001_green") is True,
        "catalog_spec_only": catalog.get("spec_only") is True,
    }
    green = all(checks.values())
    return {
        "gate": GATE_ID,
        "verdict": "PASS" if green else "FAIL",
        "critique_doc": CRITIQUE_DOC,
        "event_count": len(events),
        "checks": checks,
        "schema_audit": {
            "green": schema_audit.get("green"),
            "event_ids": schema_audit.get("event_ids"),
        },
        "reaction_territory": {
            "green": reaction_witness.get("green"),
            "reaction_session_count_full_catalog": reaction_witness.get("reaction_session_count_full_catalog"),
        },
        "confront_summary": {
            "ui_shell": "aps_tk_variants_tab",
            "horror_show": False,
            "proper_workflow_ui": True,
            "ship_score_band": "8/10 internal tool",
        },
        "green": green,
        "handoff_tails": [
            "CMCP default filter Base sessions",
            "CMCP list label humanization",
            "CMCP mandate tag collapse",
            "CMCP-REACTION-TERRITORY-PREVIEW-001",
        ],
    }


def refresh_dmcp_aps_variants_reaction_pullup_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    body = run_pullup_critique_audit(repo=root)
    body["_agent_meta"] = {
        "schema": "dmcp_aps_variants_reaction_pullup_live_v1",
        "written_at_epoch_secs": int(time.time()),
        "profile": "DMCP_APS_VARIANTS_REACTION_PULLUP",
        "source_system": "dmcp_aps_variants_reaction_pullup",
        "relative_path": WITNESS_REL,
        "ritual": f"BLANG:WIT-HON→Q✓ {GATE_ID}" if body.get("green") else "BLANG:WIT-HON FAIL",
        "agent": "designer-mcp",
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = WITNESS_REL
    return body
