"""OVR-P55-PREVIEW-002 — four visual variant states for tile/module previews."""

from __future__ import annotations

from typing import Any, Literal, TypedDict

VariantVisualState = Literal["clean", "night", "damaged", "burning"]

VARIANT_STATES: tuple[VariantVisualState, ...] = ("clean", "night", "damaged", "burning")

_STATE_LABELS: dict[VariantVisualState, str] = {
    "clean": "Clean",
    "night": "Night",
    "damaged": "Damaged",
    "burning": "Burning",
}


class VariantAxisPatch(TypedDict, total=False):
    lighting: str
    night_lights: bool
    damage_state: str
    damage: float
    emissive_overlay: bool


def variant_state_label(state: VariantVisualState) -> str:
    return _STATE_LABELS[state]


def variant_axis_patch(state: VariantVisualState) -> VariantAxisPatch:
    """Variant axes for quick renderer / catalog thumb jobs."""
    if state == "clean":
        return {"lighting": "day", "damage_state": "clean"}
    if state == "night":
        return {"lighting": "night_on", "night_lights": True, "damage_state": "clean"}
    if state == "damaged":
        return {"lighting": "day", "damage_state": "damaged", "damage": 0.45}
    return {
        "lighting": "day",
        "damage_state": "damaged",
        "damage": 0.45,
        "emissive_overlay": True,
    }


def variant_entry_to_visual_state(entry: dict) -> VariantVisualState:
    """Map variant_set row → four-state preview strip (deterministic)."""
    key = str(entry.get("variant_key") or "").lower()
    if key.startswith("burning") or "burning" in key:
        return "burning"
    layers = entry.get("layers") or {}
    damage = layers.get("damage") or {}
    lighting = layers.get("lighting") or {}
    dmg_state = str(damage.get("state") or "clean").lower()
    dmg_val = float(damage.get("damage") or 0.0)
    if dmg_state in ("damaged", "ruined") or dmg_val >= 0.25:
        return "damaged"
    lit = str(lighting.get("lighting") or "day").lower()
    if lit in ("night_on", "night_off") or bool(lighting.get("night_lights")):
        return "night"
    if "night" in key:
        return "night"
    if "damaged" in key or "damage" in key:
        return "damaged"
    return "clean"


def merge_variant_patch(base: dict, state: VariantVisualState) -> dict:
    out = dict(base)
    patch = variant_axis_patch(state)
    variants = dict(out.get("variants") or {})
    variants.update(patch)
    out["variants"] = variants
    out["preview_variant_state"] = state
    return out


def merge_variant_entry_layers(
    base: dict[str, Any],
    entry: dict[str, Any],
    state: VariantVisualState,
) -> dict[str, Any]:
    """Merge four-state strip + variant_set row layers for MCP preview."""
    out = merge_variant_patch(base, state)
    layers = entry.get("layers") or {}
    lighting = layers.get("lighting") or {}
    damage = layers.get("damage") or {}
    fill = layers.get("fill") or {}
    material = layers.get("material") or {}
    variants = dict(out.get("variants") or {})
    if lighting.get("lighting"):
        variants["lighting"] = lighting["lighting"]
    if "night_lights" in lighting:
        variants["night_lights"] = bool(lighting["night_lights"])
    if lighting.get("power"):
        variants["power"] = lighting["power"]
    if damage.get("state"):
        variants["damage_state"] = damage["state"]
    if damage.get("damage") is not None:
        variants["damage"] = float(damage["damage"])
    if fill.get("fill"):
        variants["fill"] = fill["fill"]
    if material.get("wall_material"):
        variants["wall_material"] = material["wall_material"]
    if entry.get("reaction_event_id"):
        out["reaction_event_id"] = entry["reaction_event_id"]
    if entry.get("reaction_key"):
        out["reaction_key"] = entry["reaction_key"]
    tags = entry.get("tags") or []
    if tags:
        out["reaction_tags"] = list(tags)
    anchor = entry.get("tag_anchor")
    if anchor:
        out["tag_anchor"] = dict(anchor)
    out["variants"] = variants
    return out
