"""APS tag vocabulary — human labels + artist context for mandate and assembly variant tags."""

from __future__ import annotations

from typing import Any

from rust_engine_mcp.reaction_territory import TAG_FAMILIES

# tag_id → (artist label, one-line context)
MANDATE_TAG_VOCAB: dict[str, tuple[str, str]] = {
    # light / policy reads
    "day_lit": ("Day lit", "Neutral daylight tile — default civic read."),
    "night_on": ("Night · lights on", "Emissive windows and street spill for night tiles."),
    "night_off": ("Night · lights out", "Dark facade — curfew or power cut story."),
    "night_lights": ("Window glow", "Localized emissive bands on facade modules."),
    "censorship_overlay": ("Censorship dim", "Signage blanked or obscured — policy pressure read."),
    "censorship": ("Censorship", "Media or language suppression visual cue."),
    "service_continuity": ("Service continues", "Essential services stay lit under occupation."),
    # fire / disturbance
    "sim_fire": ("Sim fire hook", "Links variant to engine fire ecology row."),
    "fire_frame_0": ("Fire frame 0", "First burn frame in atlas sequence."),
    "fire_frame_axis": ("Fire frame axis", "Burn animation strip anchor for tile batch."),
    "emissive_overlay": ("Emissive overlay", "Heat glow pass on damaged modules."),
    "burning_session": ("Burning session", "Active fire reaction session — not recovery."),
    "burn_origin": ("Burn origin", "Cell where fire started — heritage destruction anchor."),
    # heritage / mandate
    "cultural_survival": ("Cultural survival", "Practices continue under pressure — positive mandate."),
    "heritage_integrity": ("Heritage integrity", "Built heritage legibility on tile still."),
    "language_vitality": ("Language vitality", "Bilingual or local script still visible."),
    "cultural_continuity": ("Cultural continuity", "Community ritual or gathering spaces read."),
    "institutional_memory": ("Institutional memory", "Archive, school, or civic record presence."),
    "record_preservation": ("Record preservation", "Documents or collections protected."),
    "legitimacy": ("Legitimacy", "Occupation reads as lawful transition — gray mandate."),
    "essential_service_continuity": ("Essential services", "Hospital, water, transit stay open."),
    "cultural_erasure": ("Cultural erasure", "Symbols removed — negative mandate delta."),
    "bilingual_transparency": ("Bilingual transparency", "Both languages on signage — resistance read."),
    "legitimate_civil_change": ("Civil transition", "Lawful regime change without destruction."),
    "heritage_marker": ("Heritage marker", "Monument or protected facade slot."),
    "archive_slot": ("Archive slot", "Library / record room placement anchor."),
    "language_script": ("Language script", "Local script on signage modules."),
    "signage_locale": ("Signage locale", "Locale-specific wayfinding still present."),
}

ASSEMBLY_VARIANT_TAG_VOCAB: dict[str, tuple[str, str]] = {
    "clean": ("Clean", "Default maintained piece — pairs with clean_day variant."),
    "damaged": ("Damaged", "Wear or strike damage on this module."),
    "night": ("Night read", "Piece participates in night lighting pass."),
    "construction": ("Under construction", "Scaffold or incomplete facade on this slot."),
    "fire": ("Fire damage", "Char or burn mark on this piece — not active flame."),
}

SEMANTIC_TAG_EXTRA_HINTS: dict[str, str] = {
    "street_facing": "Primary facade toward the street — grammar facade.primary.",
    "corner": "Corner token — wraps two frontages.",
    "rear": "Secondary facade away from main frontage.",
    "interior": "Interior court or yard-facing module.",
    "yard_facing": "Opens toward private yard massing.",
    "alley_facing": "Service alley or laneway frontage.",
    "industrial": "Industrial family — pipes, loading, heavy massing.",
    "commercial": "Shopfront or market read.",
    "residential": "Dwelling scale windows and entries.",
    "military": "Hardened or restricted access read.",
    "civic": "Public institution — school, hall, clinic.",
    "agricultural": "Barn, silo, or farm utility read.",
    "pipework": "Visible process pipes on facade.",
    "stack": "Chimney or exhaust stack silhouette.",
    "ventilation": "Rooftop or wall ventilation units.",
    "loading_dock": "Truck bay and roll-up door zone.",
    "signage": "Billboard or fascia sign anchor.",
    "light_fixture": "Wall pack or street fixture mount.",
    "ac_unit": "Rooftop HVAC read.",
    "crane": "Industrial crane or hoist hint.",
    "platform": "Catwalk or mezzanine edge.",
    "window_band": "Repeated window rhythm band.",
    "door_rollup": "Industrial roll-up door bay.",
    "rail_adjacent": "Facade along rail corridor — logistics read.",
    "waterfront": "Harbor or river edge frontage.",
    "utility": "Process plant or substation massing.",
    "cooling_tower": "Cooling tower silhouette on roofline.",
    "transformer_yard": "Pad-mounted transformers and yard equipment.",
    "district_power_feed": "Grid tie-in read near substation placement.",
    "bilingual_signage": "Both languages on signage — locale visibility.",
    "occupation_banner": "Transitional governance banner on facade.",
    "decommissioned": "Powered down but structurally intact.",
    "clean": "Well maintained surface.",
    "weathered": "Age patina without structural loss.",
    "damaged": "Visible damage — syncs with variant damage layer.",
    "abandoned": "Vacant — boarded or dark windows.",
    "construction": "Temporary works on this piece.",
    "fire": "Fire scorch on module — static, not animated burn.",
}


def mandate_tag_label(tag_id: str) -> str:
    row = MANDATE_TAG_VOCAB.get(tag_id)
    if row:
        return row[0]
    return tag_id.replace("_", " ").title()


def mandate_tag_hint(tag_id: str) -> str:
    row = MANDATE_TAG_VOCAB.get(tag_id)
    if row:
        return row[1]
    return f"Mandate tag `{tag_id}` — binds reaction session metadata."


def assembly_variant_tag_label(tag_id: str) -> str:
    row = ASSEMBLY_VARIANT_TAG_VOCAB.get(tag_id)
    if row:
        return row[0]
    return tag_id.replace("_", " ").title()


def assembly_variant_tag_hint(tag_id: str) -> str:
    row = ASSEMBLY_VARIANT_TAG_VOCAB.get(tag_id)
    if row:
        return row[1]
    return f"Variant tag `{tag_id}` on this placement."


def semantic_tag_hint(tag_id: str, *, grammar_use: str = "") -> str:
    extra = SEMANTIC_TAG_EXTRA_HINTS.get(tag_id, "")
    if grammar_use and extra:
        return f"{extra} Grammar: {grammar_use}."
    if extra:
        return extra
    if grammar_use:
        return f"Grammar match: {grammar_use}."
    return f"Semantic tag `{tag_id}` — saved on assembly snapshot for this piece."


def reaction_event_context(event: dict[str, Any]) -> str:
    label = str(event.get("label") or "Reaction event")
    anchors = event.get("tag_anchors") or []
    previews = event.get("preview_states") or []
    anchor_labels = [mandate_tag_label(str(a)) for a in anchors[:3]]
    preview = ", ".join(str(p) for p in previews)
    anchor_text = ", ".join(anchor_labels) if anchor_labels else "—"
    return f"{label} — suggested anchors: {anchor_text} · preview: {preview}"


def compose_mandate_tag_context(active_tags: list[str], *, focus: str | None = None) -> str:
    if focus:
        return mandate_tag_hint(focus)
    if not active_tags:
        return "No mandate tags — add tags that match your reaction session story."
    labels = [mandate_tag_label(t) for t in active_tags[:4]]
    tail = f" +{len(active_tags) - 4} more" if len(active_tags) > 4 else ""
    return f"Active: {', '.join(labels)}{tail} — Apply layers to save on variant row."


def tag_vocabulary_audit() -> dict[str, Any]:
    """Witness payload — every mandate family tag must have artist label coverage."""
    missing: list[str] = []
    for _family, tags in TAG_FAMILIES.items():
        seen: set[str] = set()
        for tag in tags:
            if tag in seen:
                continue
            seen.add(tag)
            if tag not in MANDATE_TAG_VOCAB:
                missing.append(tag)
    dupes: list[str] = []
    for family, tags in TAG_FAMILIES.items():
        if len(tags) != len(set(tags)):
            dupes.append(family)
    return {
        "mandate_tags_total": sum(len(set(v)) for v in TAG_FAMILIES.values()),
        "mandate_labels_covered": sum(len(set(v)) for v in TAG_FAMILIES.values()) - len(missing),
        "missing_labels": missing,
        "families_with_duplicates": dupes,
        "green": not missing and not dupes,
    }
