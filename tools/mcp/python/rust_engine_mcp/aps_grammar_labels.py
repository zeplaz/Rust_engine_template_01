"""APS-UX-GRAMMAR-WHY — human labels for grammar rule_ids (designer glossary)."""

from __future__ import annotations

# Human label (short) + detail (why) from docs/archive/2026-06-src-dev/plans/aps_ux_grammar_why_glossary_v1.md
GRAMMAR_LABELS: dict[str, str] = {
    "IndustrialWarehouse": "Industrial warehouse",
    "industrial_west": "Industrial West district",
    "long_hall": "Long hall",
    "double_hall": "Double hall",
    "l_shape": "L-shaped yard",
    "yard_complex": "Yard complex",
    "rect": "Rectangle fill",
    "yard_interior": "Interior yard",
    "roof_default": "Default roof",
    "roof_industrial": "Industrial roof",
    "roof_flat": "Flat roof",
    "wall_1u": "Standard wall (1u)",
    "window_industrial": "Industrial window",
    "door_wide": "Wide bay door",
    "prop_clutter": "Roof / yard clutter",
    "new": "New build",
    "weathered": "Weathered",
    "abandoned": "Abandoned",
    "RailEdge": "Rail edge",
    "FactoryCluster": "Factory cluster",
    "SawtoothHall": "Sawtooth hall",
    "logistics": "Logistics",
    "industrial_british": "Industrial British",
    "temperate": "Temperate",
    "sparse": "Sparse",
    "industrial": "Industrial wealth",
    "rail": "Rail infrastructure",
    "controlled": "Controlled security",
    "utilitarian": "Utilitarian philosophy",
    "steel": "Steel material",
}

GRAMMAR_WHY: dict[str, str] = {
    "IndustrialWarehouse": "Long-span storage / logistics shell; drives footprint bounds and module kit.",
    "industrial_west": "Sawtooth + steel palette; default material profile map for wall/roof/door slots.",
    "long_hall": "Wide shallow rectangle — main storage hall along street frontage.",
    "double_hall": "Two-bay depth; moderate width:depth ratio for split interior zones.",
    "l_shape": "L footprint — corner yard or loading wing.",
    "yard_complex": "Interior yard massing; flat roof bias, yard-facing modules.",
    "rect": "Standard grid rectangle placement.",
    "yard_interior": "Open yard cell pattern inside shell.",
    "roof_default": "Fallback roof module slot when massing has no override.",
    "roof_industrial": "Sawtooth / metal industrial roof — paired with long_hall.",
    "roof_flat": "Low-pitch flat cap — paired with yard_complex.",
    "wall_1u": "Primary exterior wall module slot.",
    "window_industrial": "Factory-style glazing slot on street-facing runs.",
    "door_wide": "Loading / vehicle door slot.",
    "prop_clutter": "Vents, pipes, platforms — density prop scatter.",
    "new": "Clean variant tags; high weight in fresh districts.",
    "weathered": "Mix of clean + weathered variant states.",
    "abandoned": "Damaged / abandoned variant bias for edge lots.",
    "RailEdge": "Primary massing hugs rail spur — loading wing + yard bias (βyard).",
    "FactoryCluster": "Multi-bay industrial cluster — higher βmod and service pressure.",
    "SawtoothHall": "Sawtooth roof hall — pairs with long_hall / industrial roof slots.",
    "logistics": "Storage + movement program — warehouse / rail yard bias.",
    "industrial_british": "Victorian–Edwardian industrial lineage palette.",
    "temperate": "Moderate climate — default weathering band.",
    "sparse": "Low site density — large yards and setbacks.",
    "industrial": "Working industrial wealth tier — utilitarian finishes.",
    "rail": "Rail spur / siding infrastructure anchor.",
    "controlled": "Fenced / gated security posture.",
    "utilitarian": "Function-over-form design philosophy.",
    "steel": "Primary structural / cladding material.",
}


def human_label(rule_id: str) -> str:
    rid = str(rule_id or "").strip()
    if not rid:
        return "—"
    return GRAMMAR_LABELS.get(rid, rid.replace("_", " ").title())


def grammar_why_detail(rule_id: str) -> str:
    rid = str(rule_id or "").strip()
    why = GRAMMAR_WHY.get(rid, "")
    label = human_label(rid)
    if why:
        return f"{label} — {why}"
    return label
