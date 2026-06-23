"""APS P0-C — reaction-territory event catalog + deterministic variant sessions."""

from __future__ import annotations

import hashlib
import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.variant_matrix_expand import VARIANT_BAKE, variant_set_rows

CATALOG_JSON_REL = "tools/mcp/schemas/examples/reaction_territory_events_v1.json"
CATALOG_YAML_REL = "tools/mcp/schemas/examples/reaction_territory_events_v1.yaml"
WITNESS_REL = "debug_runs/art_pipeline/variants_reaction_territory_live.json"

P0_EVENT_IDS: tuple[str, ...] = (
    "heritage_site_destruction",
    "language_ban",
    "transparent_bilingual_service_continuation",
)

CATALOG_EVENT_IDS: tuple[str, ...] = (
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

# Catalog preview_states → APS four-state strip ids (CMCP-REACTION-TERRITORY-PREVIEW-001).
CATALOG_PREVIEW_STATE_MAP: dict[str, str] = {
    "clean": "clean",
    "damaged": "damaged",
    "burning": "burning",
    "night": "night",
    "scar": "damaged",
}

# Mandate-family tag picks for Variants tab (light / fire / heritage).
TAG_FAMILIES: dict[str, tuple[str, ...]] = {
    "light": (
        "day_lit",
        "night_on",
        "night_off",
        "night_lights",
        "censorship_overlay",
        "service_continuity",
        "censorship",
    ),
    "fire": (
        "sim_fire",
        "fire_frame_0",
        "fire_frame_axis",
        "emissive_overlay",
        "burning_session",
        "burn_origin",
    ),
    "heritage": (
        "cultural_survival",
        "heritage_integrity",
        "language_vitality",
        "cultural_continuity",
        "institutional_memory",
        "record_preservation",
        "legitimacy",
        "essential_service_continuity",
        "cultural_erasure",
        "service_continuity",
        "bilingual_transparency",
        "legitimate_civil_change",
        "heritage_marker",
        "archive_slot",
        "language_script",
        "signage_locale",
    ),
}

_ABSTRACT_VARIANT_FALLBACK: dict[str, str] = {
    "damaged_heavy": "damaged_day",
    "burning": "burning_00",
    "censorship_dim": "clean_night_off",
    "service_lit": "clean_night_on",
    "scar_recovery_0": "clean_day",
    "clean_restored": "clean_day",
    "fire": "burning_00",
    "topology_patch_scar": "damaged_day",
    "topology_patch_burn_04": "burning_00",
    "topology_patch_regrowth_grass": "clean_day",
    "topology_patch": "clean_day",
}

_DOMAIN_TO_TAG_FAMILY: dict[str, str] = {
    "fire": "fire",
    "heritage": "heritage",
    "policy": "light",
    "mandate": "heritage",
}


def load_reaction_catalog(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    json_path = root / CATALOG_JSON_REL
    if not json_path.is_file():
        raise FileNotFoundError(f"missing reaction catalog: {CATALOG_JSON_REL}")
    return json.loads(json_path.read_text(encoding="utf-8"))


def reaction_key_hash(assembly_id: str, event_id: str, seed: int) -> str:
    """Deterministic reaction key: hash(assembly_id + event_id + seed)."""
    payload = f"{assembly_id}|{event_id}|{seed}"
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()[:8]


def reaction_session_variant_key(event_id: str, base_key: str, reaction_key: str) -> str:
    return f"{event_id}__{base_key}__{reaction_key}"


def resolution_domain_for_assembly(assembly_id: str) -> str:
    aid = assembly_id.lower()
    if "landscape" in aid or "topology" in aid:
        return "landscape_topology"
    if "heritage" in aid or "civic" in aid:
        return "heritage_civic"
    if "rowhouse" in aid or "victorian" in aid or "shopfront" in aid or "colonial" in aid:
        return "building_rowhouse"
    return "building_warehouse"


def resolve_concrete_variant_key(catalog: dict[str, Any], abstract_key: str, domain: str) -> str:
    resolution = (catalog.get("variant_layer_resolution") or {}).get(domain) or {}
    concrete = str(resolution.get(abstract_key) or abstract_key)
    if concrete not in VARIANT_BAKE:
        concrete = _ABSTRACT_VARIANT_FALLBACK.get(concrete, concrete)
    if concrete not in VARIANT_BAKE:
        concrete = _ABSTRACT_VARIANT_FALLBACK.get(abstract_key, "clean_day")
    return concrete


def resolve_event_variant_keys(
    catalog: dict[str, Any],
    event: dict[str, Any],
    *,
    assembly_id: str,
) -> list[str]:
    domain = str(event.get("default_resolution_domain") or resolution_domain_for_assembly(assembly_id))
    keys: list[str] = []
    for abstract in event.get("variant_keys") or []:
        concrete = resolve_concrete_variant_key(catalog, str(abstract), domain)
        if concrete not in keys:
            keys.append(concrete)
    return keys


def resolve_tag_anchors(catalog: dict[str, Any], event: dict[str, Any]) -> dict[str, list[str]]:
    anchor_catalog = catalog.get("tag_anchor_catalog") or {}
    grouped: dict[str, list[str]] = {"light": [], "fire": [], "heritage": []}
    for anchor_id in event.get("tag_anchors") or []:
        entry = anchor_catalog.get(str(anchor_id)) or {}
        domain = str(entry.get("domain") or "heritage")
        family = _DOMAIN_TO_TAG_FAMILY.get(domain, "heritage")
        for bind in entry.get("binds") or []:
            tag = str(bind)
            if tag not in grouped[family]:
                grouped[family].append(tag)
        if str(anchor_id) not in grouped[family]:
            grouped[family].append(str(anchor_id))
    return grouped


def flatten_resolved_tag_anchors(grouped: dict[str, list[str]]) -> list[str]:
    out: list[str] = []
    for family in ("light", "fire", "heritage"):
        for tag in grouped.get(family) or []:
            if tag not in out:
                out.append(tag)
    return out


def resolve_reaction_territory_variant(
    event_id: str,
    domain: str,
    *,
    repo: Path | None = None,
) -> dict[str, Any]:
    """CMCP-REACTION-TERRITORY-RESOLVE-001 — event + domain → concrete variant keys."""
    catalog = load_reaction_catalog(repo=repo)
    events = catalog.get("events") or {}
    event = events.get(event_id)
    if not event:
        raise KeyError(f"unknown reaction event: {event_id}")
    abstract_keys = list(event.get("variant_keys") or [])
    concrete_keys = [
        resolve_concrete_variant_key(catalog, str(a), domain) for a in abstract_keys
    ]
    deduped: list[str] = []
    for key in concrete_keys:
        if key not in deduped:
            deduped.append(key)
    resolution = (catalog.get("variant_layer_resolution") or {}).get(domain) or {}
    return {
        "event_id": event_id,
        "domain": domain,
        "abstract_variant_keys": abstract_keys,
        "concrete_variant_keys": deduped,
        "resolution_map": {str(k): resolution.get(str(k), str(k)) for k in abstract_keys},
        "preview_states": list(event.get("preview_states") or []),
        "preview_visual_states": preview_visual_states_for_catalog_states(
            list(event.get("preview_states") or [])
        ),
        "tag_anchors": resolve_tag_anchors(catalog, event),
    }


def preview_visual_states_for_catalog_states(states: list[str]) -> list[str]:
    """Map catalog preview_states labels to APS strip state ids."""
    out: list[str] = []
    for state in states:
        mapped = CATALOG_PREVIEW_STATE_MAP.get(str(state).lower())
        if mapped and mapped not in out:
            out.append(mapped)
    return out or ["clean"]


def preview_visual_states_for_entry(entry: dict[str, Any]) -> list[str]:
    catalog_states = list(entry.get("preview_states") or [])
    if catalog_states:
        return preview_visual_states_for_catalog_states(catalog_states)
    return ["clean", "night", "damaged", "burning"]


def audit_resolver_all_domains(*, repo: Path | None = None) -> dict[str, bool]:
    """Resolver smoke — every catalog event resolves on every domain."""
    catalog = load_reaction_catalog(repo=repo)
    domains = tuple((catalog.get("variant_layer_resolution") or {}).keys()) or (
        "building_warehouse",
        "building_rowhouse",
        "landscape_topology",
        "heritage_civic",
    )
    checks: dict[str, bool] = {}
    for event_id in CATALOG_EVENT_IDS:
        for domain in domains:
            try:
                body = resolve_reaction_territory_variant(event_id, str(domain), repo=repo)
                checks[f"{event_id}@{domain}"] = bool(body.get("concrete_variant_keys"))
            except KeyError:
                checks[f"{event_id}@{domain}"] = False
    return checks


def reaction_preview_cell(snapshot: dict[str, Any], entry: dict[str, Any] | None = None) -> tuple[int, int]:
    """v1 anchor: tag_anchor cell on entry, else footprint center, else first placement."""
    if entry:
        anchor = entry.get("tag_anchor") or {}
        if "cell_x" in anchor and "cell_y" in anchor:
            return int(anchor["cell_x"]), int(anchor["cell_y"])
    footprint = snapshot.get("footprint") or {}
    w = int(footprint.get("width") or 4)
    d = int(footprint.get("depth") or 3)
    cx, cy = w // 2, d // 2
    event_id = str((entry or {}).get("reaction_event_id") or "")
    placements = snapshot.get("module_placements") or []
    if event_id == "heritage_site_destruction":
        for row in placements:
            tags = list(row.get("placement_tags") or []) + list(row.get("variant_tags") or [])
            joined = " ".join(str(t).lower() for t in tags)
            if any(k in joined for k in ("heritage", "facade", "corner", "exterior")):
                return int(row.get("grid_x") or cx), int(row.get("grid_y") or cy)
    if placements:
        first = placements[0]
        return int(first.get("grid_x") or cx), int(first.get("grid_y") or cy)
    return cx, cy


def build_reaction_session_rows(
    *,
    assembly_id: str,
    seed: int,
    style_pack_id: str,
    event_ids: tuple[str, ...] | None = None,
    repo: Path | None = None,
) -> list[dict[str, Any]]:
    catalog = load_reaction_catalog(repo=repo)
    events = catalog.get("events") or {}
    pack_tag = str(style_pack_id or "style_victorian").removeprefix("style_")
    rows: list[dict[str, Any]] = []
    selected = event_ids or P0_EVENT_IDS

    for event_id in selected:
        event = events.get(event_id)
        if not event:
            continue
        rkey = reaction_key_hash(assembly_id, event_id, seed)
        base_keys = resolve_event_variant_keys(catalog, event, assembly_id=assembly_id)
        base_rows = {r["variant_key"]: r for r in variant_set_rows(base_keys)}
        tag_grouped = resolve_tag_anchors(catalog, event)
        anchor_tags = flatten_resolved_tag_anchors(tag_grouped)
        mandate = str(event.get("mandate_family") or "heritage")
        meta_tags = [f"reaction_event_{event_id}", mandate]

        for base_key in base_keys:
            base = dict(base_rows.get(base_key) or {})
            if not base:
                continue
            vkey = reaction_session_variant_key(event_id, base_key, rkey)
            tags = list(base.get("tags") or [])
            tags.extend(anchor_tags)
            tags.extend(
                [
                    "reaction_territory",
                    f"reaction_key_{rkey}",
                    f"stylepack_{pack_tag}",
                    f"seed_{seed}",
                ]
            )
            rows.append(
                {
                    **base,
                    "variant_key": vkey,
                    "base_variant_key": base_key,
                    "abstract_variant_key": next(
                        (
                            str(a)
                            for a in (event.get("variant_keys") or [])
                            if resolve_concrete_variant_key(
                                catalog,
                                str(a),
                                str(event.get("default_resolution_domain") or resolution_domain_for_assembly(assembly_id)),
                            )
                            == base_key
                        ),
                        base_key,
                    ),
                    "reaction_event_id": event_id,
                    "reaction_key": rkey,
                    "reaction_label": event.get("label"),
                    "mandate_family": mandate,
                    "cultural_liquidation_trigger": event.get("cultural_liquidation_trigger"),
                    "preview_states": list(event.get("preview_states") or []),
                    "metric_deltas": dict(event.get("metric_deltas") or {}),
                    "tag_anchors": tag_grouped,
                    "tag_anchor": {
                        "cell_x": 0,
                        "cell_y": 0,
                        "anchor_kind": "cell_center_v1",
                    },
                    "tags": list(dict.fromkeys(tags + meta_tags)),
                    "sim_tags": list(dict.fromkeys(list(base.get("sim_tags") or []) + tags + meta_tags)),
                    "agent_preferred_response": list(event.get("agent_preferred_response") or []),
                }
            )
    return rows


def apply_tag_anchor_from_snapshot(rows: list[dict[str, Any]], snapshot: dict[str, Any]) -> None:
    for row in rows:
        cx, cy = reaction_preview_cell(snapshot, row)
        anchor = dict(row.get("tag_anchor") or {})
        anchor["cell_x"] = cx
        anchor["cell_y"] = cy
        row["tag_anchor"] = anchor


def refresh_reaction_territory_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    catalog = load_reaction_catalog(repo=root)
    events = catalog.get("events") or {}
    p0_rows = build_reaction_session_rows(
        assembly_id="warehouse_industrial_west_production_v1",
        seed=42,
        style_pack_id="style_industrial_west",
        repo=root,
    )
    full_catalog_rows = build_reaction_session_rows(
        assembly_id="warehouse_industrial_west_production_v1",
        seed=42,
        style_pack_id="style_industrial_west",
        event_ids=CATALOG_EVENT_IDS,
        repo=root,
    )
    keys = [str(r.get("variant_key")) for r in full_catalog_rows]
    rkey = reaction_key_hash("warehouse_industrial_west_production_v1", "language_ban", 42)
    resolver_checks = audit_resolver_all_domains(repo=root)
    heritage_resolve = resolve_reaction_territory_variant(
        "heritage_site_destruction",
        "heritage_civic",
        repo=root,
    )
    preview_map_ok = heritage_resolve.get("preview_visual_states") == ["damaged", "burning", "clean"]
    catalog_ok = all(eid in events for eid in CATALOG_EVENT_IDS)
    yaml_ok = (root / CATALOG_YAML_REL).is_file()
    json_ok = (root / CATALOG_JSON_REL).is_file()
    resolver_green = all(resolver_checks.values())
    green = (
        catalog_ok
        and len(p0_rows) >= 4
        and len(full_catalog_rows) >= len(p0_rows)
        and json_ok
        and yaml_ok
        and resolver_green
        and preview_map_ok
    )
    body: dict[str, Any] = {
        "gate": "APS-P0-REACTION-TERRITORY-001",
        "green": green,
        "event_ids": list(P0_EVENT_IDS),
        "catalog_event_ids": list(CATALOG_EVENT_IDS),
        "reaction_session_count": len(p0_rows),
        "reaction_session_count_p0": len(p0_rows),
        "reaction_session_count_full_catalog": len(full_catalog_rows),
        "sample_variant_keys": keys,
        "sample_reaction_key_language_ban": rkey,
        "cmcp_resolve_001_green": resolver_green,
        "cmcp_preview_001_green": preview_map_ok,
        "resolver_checks": resolver_checks,
        "heritage_civic_resolve": heritage_resolve,
        "catalog_json": CATALOG_JSON_REL,
        "catalog_yaml": CATALOG_YAML_REL,
        "tag_families": {k: list(v) for k, v in TAG_FAMILIES.items()},
        "_agent_meta": {
            "schema": "variants_reaction_territory_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "APS_REACTION_TERRITORY",
            "source_system": "reaction_territory",
            "relative_path": WITNESS_REL,
            "ritual": "BLANG:WIT-HON CMCP-REACTION-TERRITORY-RESOLVE+PREVIEW" if green else None,
        },
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = WITNESS_REL
    return body
