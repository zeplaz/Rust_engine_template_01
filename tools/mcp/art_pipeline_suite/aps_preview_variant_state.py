"""OVR-P55-PREVIEW-002 — four visual variant states for tile/module previews."""

from __future__ import annotations

from typing import Literal, TypedDict

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


def merge_variant_patch(base: dict, state: VariantVisualState) -> dict:
    out = dict(base)
    patch = variant_axis_patch(state)
    variants = dict(out.get("variants") or {})
    variants.update(patch)
    out["variants"] = variants
    out["preview_variant_state"] = state
    return out
