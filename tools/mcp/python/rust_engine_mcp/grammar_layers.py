"""GRAMMAR-002 — partial roof/facade layer regen on assembly snapshots."""

from __future__ import annotations

from typing import Any

from . import assembly, building_grammar

ROOF_TOKENS = frozenset({"R"})
FACADE_TOKENS = frozenset({"W", "D", "C"})


def _pinned_grammar_kwargs(snapshot: dict[str, Any], seed: int) -> dict[str, Any]:
    chain = snapshot.get("grammar_rule_chain") if isinstance(snapshot.get("grammar_rule_chain"), dict) else {}
    fp = snapshot.get("footprint") or {}
    kwargs: dict[str, Any] = {"seed": seed}
    if chain.get("massing"):
        kwargs["massing_strategy"] = str(chain["massing"])
    if fp.get("width") and fp.get("depth"):
        kwargs["footprint"] = {
            "width": int(fp["width"]),
            "depth": int(fp["depth"]),
            "floors": int(fp.get("floors") or 1),
        }
    if chain.get("footprint_mode"):
        kwargs["footprint_mode"] = str(chain["footprint_mode"])
    if chain.get("age"):
        kwargs["age_band_id"] = str(chain["age"])
    return kwargs


def apply_roof_layer(
    snapshot: dict[str, Any],
    overrides: dict[str, Any],
    *,
    seed: int | None = None,
) -> dict[str, Any]:
    """Re-resolve roof (R token) placements; preserve footprint and facade ring."""
    archetype_id = str(snapshot.get("archetype_id") or "")
    district_style = str(snapshot.get("district_style") or "")
    if not archetype_id or not district_style:
        raise ValueError("snapshot missing archetype_id or district_style")
    seed = int(seed if seed is not None else snapshot.get("seed") or 42)
    pins = _pinned_grammar_kwargs(snapshot, seed)
    pins["roof_slot"] = overrides.get("roof_slot")
    pins["roof_rule_id"] = overrides.get("roof_rule_id")
    grammar = building_grammar.generate_with_overrides(archetype_id, district_style, **pins)
    return assembly.refresh_placements_for_tokens(snapshot, grammar, ROOF_TOKENS)


def apply_facade_layer(
    snapshot: dict[str, Any],
    overrides: dict[str, Any],
    *,
    seed: int | None = None,
) -> dict[str, Any]:
    """Re-resolve perimeter W/D/C placements; preserve footprint and roof."""
    archetype_id = str(snapshot.get("archetype_id") or "")
    district_style = str(snapshot.get("district_style") or "")
    if not archetype_id or not district_style:
        raise ValueError("snapshot missing archetype_id or district_style")
    seed = int(seed if seed is not None else snapshot.get("seed") or 42)
    pins = _pinned_grammar_kwargs(snapshot, seed)
    pins["wall_slot"] = overrides.get("wall_slot")
    pins["door_slot"] = overrides.get("door_slot")
    pins["window_slot"] = overrides.get("window_slot")
    pins["facade_rule_id"] = overrides.get("facade_rule_id")
    grammar = building_grammar.generate_with_overrides(archetype_id, district_style, **pins)
    return assembly.refresh_placements_for_tokens(snapshot, grammar, FACADE_TOKENS)


def apply_roof_layer_copy(
    snapshot: dict[str, Any],
    overrides: dict[str, Any],
    *,
    seed: int | None = None,
) -> tuple[dict[str, Any], dict[str, Any]]:
    child = apply_roof_layer(snapshot, overrides, seed=seed)
    seed = int(seed if seed is not None else snapshot.get("seed") or 42)
    pins = _pinned_grammar_kwargs(snapshot, seed)
    pins["roof_slot"] = overrides.get("roof_slot")
    pins["roof_rule_id"] = overrides.get("roof_rule_id")
    grammar = building_grammar.generate_with_overrides(
        str(snapshot.get("archetype_id") or ""),
        str(snapshot.get("district_style") or ""),
        **pins,
    )
    return child, grammar
