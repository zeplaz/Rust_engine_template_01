"""DES-REACTION-TERRITORY-EVENTS-001 — reaction territory event catalog witness."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.schemas import load_json_file

WITNESS_REL = "debug_runs/art_pipeline/dmcp_reaction_territory_events_live.json"
GATE_ID = "DES-REACTION-TERRITORY-EVENTS-001"
CATALOG_JSON = "tools/mcp/schemas/examples/reaction_territory_events_v1.json"
CATALOG_YAML = "tools/mcp/schemas/examples/reaction_territory_events_v1.yaml"
SCHEMA_JSON = "tools/mcp/schemas/reaction_territory_events_v1.schema.json"
DESIGN_DOC = "src/dev/design_reaction_territory_events_schema_v1.md"
SOURCE_DOC = "docs/extr_cell_and_liquidation/scarydayzx.txt"

HERITAGE_EXAMPLE = {
    "variant_keys": ["damaged_heavy", "burning", "scar_recovery_0"],
    "metric_deltas": {
        "heritage_integrity_index": -0.25,
        "cultural_continuity_index": -0.18,
    },
    "tag_anchors": ["burn_origin", "heritage_marker", "archive_slot"],
    "preview_states": ["damaged", "burning", "clean"],
}

REQUIRED_EVENTS = (
    "heritage_site_destruction",
    "language_ban",
    "transparent_bilingual_service_continuation",
    "forced_assimilation_in_schools",
    "archive_seizure_or_censorship",
    "forced_renaming",
    "banning_cultural_or_religious_practices",
    "removal_of_children_from_institutions",
    "forced_displacement",
    "erasure_of_local_history",
    "imperial_institution_replacement",
)

DOC_LIQUIDATION_TRIGGERS = (
    "language_ban",
    "forced_renaming",
    "destruction_of_heritage_sites",
    "seizure_or_censorship_of_archives",
    "forced_assimilation_in_schools",
    "banning_of_cultural_or_religious_practices",
    "removal_of_children_from_community_institutions",
    "forced_displacement",
    "erasure_of_local_history",
    "replacement_of_local_institutions_with_imperial_administration",
)

REQUIRED_RESOLUTION_DOMAINS = (
    "building_warehouse",
    "building_rowhouse",
    "landscape_topology",
    "heritage_civic",
)


def _heritage_pattern_ok(event: dict[str, Any]) -> dict[str, bool]:
    return {
        "variant_keys": event.get("variant_keys") == HERITAGE_EXAMPLE["variant_keys"],
        "metric_deltas_core": all(
            event.get("metric_deltas", {}).get(k) == v
            for k, v in HERITAGE_EXAMPLE["metric_deltas"].items()
        ),
        "tag_anchors": event.get("tag_anchors") == HERITAGE_EXAMPLE["tag_anchors"],
        "preview_states": event.get("preview_states") == HERITAGE_EXAMPLE["preview_states"],
    }


def _layers_resolve(catalog: dict[str, Any]) -> dict[str, bool]:
    layer_catalog = catalog.get("variant_layer_catalog") or {}
    resolution = catalog.get("variant_layer_resolution") or {}
    checks: dict[str, bool] = {}
    for domain in REQUIRED_RESOLUTION_DOMAINS:
        domain_map = resolution.get(domain) or {}
        checks[f"domain_{domain}"] = bool(domain_map)
    for layer in HERITAGE_EXAMPLE["variant_keys"]:
        checks[f"layer_{layer}_defined"] = layer in layer_catalog
    heritage = (catalog.get("events") or {}).get("heritage_site_destruction") or {}
    domain = str(heritage.get("default_resolution_domain") or "")
    domain_map = resolution.get(domain) or {}
    for layer in HERITAGE_EXAMPLE["variant_keys"]:
        checks[f"heritage_resolves_{layer}"] = layer in domain_map
    return checks


def _metric_keys_registered(catalog: dict[str, Any]) -> bool:
    metrics = catalog.get("occupied_region_metrics") or {}
    events = catalog.get("events") or {}
    for event in events.values():
        for key in (event.get("metric_deltas") or {}):
            if key not in metrics:
                return False
    return True


def _liquidation_triggers_covered(catalog: dict[str, Any]) -> bool:
    events = catalog.get("events") or {}
    mapped = {
        str(event.get("cultural_liquidation_trigger"))
        for event in events.values()
        if event.get("cultural_liquidation_trigger")
    }
    return all(trigger in mapped for trigger in DOC_LIQUIDATION_TRIGGERS)


def run_reaction_territory_events_audit(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    catalog_path = root / CATALOG_JSON
    if not catalog_path.is_file():
        return {
            "gate": GATE_ID,
            "catalog_id": None,
            "deliverable": DESIGN_DOC,
            "checks": {"catalog_json": False},
            "audit_complete": False,
            "green": False,
            "verdict": "FAIL",
            "handoff": {},
        }
    catalog = load_json_file(catalog_path)
    events = catalog.get("events") or {}
    heritage = events.get("heritage_site_destruction") or {}
    heritage_checks = _heritage_pattern_ok(heritage)
    resolution_checks = _layers_resolve(catalog)

    checks = {
        "design_doc": (root / DESIGN_DOC).is_file(),
        "catalog_json": (root / CATALOG_JSON).is_file(),
        "catalog_yaml": (root / CATALOG_YAML).is_file(),
        "schema_json": (root / SCHEMA_JSON).is_file(),
        "source_doc": (root / SOURCE_DOC).is_file(),
        "gate_id": catalog.get("gate") == GATE_ID,
        "spec_only": catalog.get("spec_only") is True,
        "event_count_11": len(events) == 11,
        "all_required_events": all(eid in events for eid in REQUIRED_EVENTS),
        "liquidation_triggers_complete": _liquidation_triggers_covered(catalog),
        "metrics_registered": _metric_keys_registered(catalog),
        **heritage_checks,
        **resolution_checks,
    }
    green = all(checks.values())
    from rust_engine_mcp.reaction_territory import (
        WITNESS_REL as REACTION_WITNESS_REL,
        refresh_reaction_territory_witness,
    )

    cmcp = refresh_reaction_territory_witness(repo=root)
    return {
        "gate": GATE_ID,
        "catalog_id": catalog.get("catalog_id"),
        "deliverable": DESIGN_DOC,
        "catalog_json": CATALOG_JSON,
        "event_ids": sorted(events.keys()),
        "heritage_site_destruction": {
            "variant_keys": heritage.get("variant_keys"),
            "preview_states": heritage.get("preview_states"),
            "tag_anchors": heritage.get("tag_anchors"),
        },
        "checks": checks,
        "audit_complete": True,
        "green": green,
        "verdict": "PASS" if green else "FAIL",
        "handoff": {
            "coder_mcp": [
                "CMCP-REACTION-TERRITORY-RESOLVE-001",
                "CMCP-REACTION-TERRITORY-PREVIEW-001",
            ],
            "resolver_input": "event_id + default_resolution_domain → variant_layer_resolution",
            "cmcp_resolve_001_green": cmcp.get("cmcp_resolve_001_green"),
            "cmcp_preview_001_green": cmcp.get("cmcp_preview_001_green"),
            "variants_reaction_territory_witness": REACTION_WITNESS_REL,
        },
    }


def refresh_dmcp_reaction_territory_events_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    body = run_reaction_territory_events_audit(repo=root)
    body["_agent_meta"] = {
        "schema": "dmcp_reaction_territory_events_live_v1",
        "written_at_epoch_secs": int(time.time()),
        "profile": "DES_REACTION_TERRITORY_EVENTS",
        "source_system": "dmcp_reaction_territory_events",
        "relative_path": WITNESS_REL,
        "ritual": f"BLANG:WIT-HON→Q✓ {GATE_ID}" if body.get("green") else "BLANG:WIT-HON FAIL",
        "agent": "designer-mcp",
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = WITNESS_REL
    return body
