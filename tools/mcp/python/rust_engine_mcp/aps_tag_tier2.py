"""APS-TAG-TIER2-IMPL — archetype tag presets + reaction suggested tags."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from rust_engine_mcp.aps_tag_vocabulary import mandate_tag_label
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.reaction_territory import load_reaction_catalog

PRESETS_REL = "tools/mcp/schemas/examples/aps_tag_tier2_presets_v1.json"
WITNESS_REL = "debug_runs/aps_tag_tier2_live.json"


def load_archetype_tag_presets(*, repo: Path | None = None) -> dict[str, Any]:
    path = (repo or repo_root()) / PRESETS_REL
    return json.loads(path.read_text(encoding="utf-8"))


def preset_for_archetype(archetype_id: str, *, repo: Path | None = None) -> dict[str, Any] | None:
    body = load_archetype_tag_presets(repo=repo)
    row = (body.get("archetypes") or {}).get(str(archetype_id))
    return dict(row) if isinstance(row, dict) else None


def preset_confirm_lines(archetype_id: str, *, repo: Path | None = None) -> list[str]:
    row = preset_for_archetype(archetype_id, repo=repo)
    if not row:
        return [f"No tier-2 preset for archetype {archetype_id!r}."]
    lines = [f"Preset: {row.get('label') or row.get('preset_name')}"]
    mandate = row.get("mandate_tags") or []
    if mandate:
        labels = [mandate_tag_label(str(t)) for t in mandate]
        lines.append(f"Mandate tags: {', '.join(labels)}")
    semantic = row.get("semantic_tags") or {}
    if semantic:
        flat = [f"{cat}:{tid}" for cat, ids in semantic.items() for tid in ids or []]
        lines.append(f"Semantic tags: {', '.join(flat)}")
    return lines


def suggested_mandate_tags_for_event(event_id: str, *, repo: Path | None = None) -> list[str]:
    catalog = load_reaction_catalog(repo=repo)
    event = (catalog.get("events") or {}).get(event_id) or {}
    anchors = event.get("tag_anchors") or []
    return [str(a) for a in anchors if a]


def audit_tag_tier2(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    presets = load_archetype_tag_presets(repo=root)
    archetypes = presets.get("archetypes") or {}
    required = ("IndustrialWarehouse", "FactoryCluster", "CivicBlock", "RailEdge")
    missing = [aid for aid in required if aid not in archetypes]
    taxonomy = json.loads(
        (root / "tools/mcp/schemas/examples/aps_tag_taxonomy_v1.json").read_text(encoding="utf-8")
    )
    detail_ids = {str(t["id"]) for t in (taxonomy.get("categories") or {}).get("detail", {}).get("tags") or []}
    condition_ids = {
        str(t["id"]) for t in (taxonomy.get("categories") or {}).get("condition", {}).get("tags") or []
    }
    sim_coupled = ("district_power_feed", "bilingual_signage", "occupation_banner", "decommissioned")
    sim_ok = all(tag in detail_ids or tag in condition_ids for tag in sim_coupled)
    variants_src = (root / "tools/mcp/art_pipeline_suite/variants_panel.py").read_text(encoding="utf-8")
    assembly_src = (root / "tools/mcp/art_pipeline_suite/assembly_panel.py").read_text(encoding="utf-8")
    ui_wired = (
        "Apply tag preset" in variants_src
        and "Apply tag preset" in assembly_src
        and "_suggested_tags_row" in variants_src
    )
    return {
        "task_id": "APS-TAG-TIER2-IMPL",
        "green": not missing and sim_ok and ui_wired,
        "preset_count": len(archetypes),
        "missing_archetypes": missing,
        "sim_coupled_tags_present": sim_ok,
        "ui_preset_buttons_wired": "Apply tag preset" in variants_src and "Apply tag preset" in assembly_src,
        "ui_suggested_tags_wired": "_suggested_tags_row" in variants_src,
        "presets_path": PRESETS_REL,
    }


def write_aps_tag_tier2_witness(*, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    body = audit_tag_tier2(repo=repo)
    return write_aps_live_witness(
        body,
        WITNESS_REL,
        schema="aps_tag_tier2_live_v1",
        profile="APS_TAG_TIER2",
        source_system="aps_tag_tier2",
        ritual="BLANG:WIT-HON APS-TAG-TIER2-IMPL" if body.get("green") else None,
    )
