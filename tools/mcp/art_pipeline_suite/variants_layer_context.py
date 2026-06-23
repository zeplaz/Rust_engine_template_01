"""DES-APS-VARIANTS-LIVE-PREVIEW-001 — layer control context lines + draft layer dict."""

from __future__ import annotations

from typing import Any

_LIGHTING_HINTS: dict[str, str] = {
    "day": "Daylight still — default tile read.",
    "night_off": "Night scene with window lights off.",
    "night_on": "Night still with emissive windows — maps to Night preview chip.",
}

_POWER_HINTS: dict[str, str] = {
    "off": "Grid off — no powered read on tiles.",
    "partial": "Partial grid — dims non-critical lights in preview.",
    "on": "Full power — pairs with night_on for lit facade tiles.",
}

_DAMAGE_HINTS: dict[str, str] = {
    "clean": "Pristine facade — Clean chip.",
    "dirty": "Surface grime — subtle wear, still reads as occupied.",
    "damaged": "Visible damage — Damaged chip when wear ≥ 25%.",
    "ruined": "Ruined shell — strongest damage read for tiles.",
}

_FILL_HINTS: dict[str, str] = {
    "empty": "Vacant read — occupancy overlay only, not geometry swap.",
    "quarter": "Light occupancy — sim/state tile hint.",
    "half": "Half occupancy — common mid-state for sim tiles.",
    "full": "Full occupancy — busiest still for batch.",
}


def build_layers_from_controls(
    *,
    lighting: str,
    power: str,
    night_lights: bool,
    damage_state: str,
    damage: float,
    fill: str,
    wall_material: str,
) -> dict[str, Any]:
    layers: dict[str, Any] = {
        "lighting": {
            "lighting": lighting,
            "power": power,
            "night_lights": night_lights,
        },
        "damage": {
            "state": damage_state,
            "damage": round(float(damage), 3),
        },
        "fill": {"fill": fill},
    }
    mat = wall_material.strip()
    if mat:
        layers["material"] = {"wall_material": mat}
    return layers


def tags_from_vars(tag_vars: dict[str, Any]) -> list[str]:
    return sorted(tag for tag, var in tag_vars.items() if var.get())


def layers_match_saved(entry: dict[str, Any] | None, draft_layers: dict[str, Any]) -> bool:
    if not entry:
        return True
    saved = entry.get("layers") or {}
    return _normalize_layers(saved) == _normalize_layers(draft_layers)


def tags_match_saved(entry: dict[str, Any] | None, draft_tags: list[str]) -> bool:
    if not entry:
        return True
    saved = sorted(entry.get("tags") or [])
    return saved == sorted(draft_tags)


def draft_is_dirty(entry: dict[str, Any] | None, draft_layers: dict[str, Any], draft_tags: list[str]) -> bool:
    if entry is None:
        return False
    return not layers_match_saved(entry, draft_layers) or not tags_match_saved(entry, draft_tags)


def merge_draft_into_entry(entry: dict[str, Any], draft_layers: dict[str, Any], draft_tags: list[str]) -> dict[str, Any]:
    out = dict(entry)
    out["layers"] = draft_layers
    if draft_tags:
        out["tags"] = list(draft_tags)
    elif "tags" in out:
        out["tags"] = []
    return out


def compose_context_line(
    *,
    lighting: str,
    power: str,
    night_lights: bool,
    damage_state: str,
    damage: float,
    fill: str,
    wall_material: str,
    focus: str | None = None,
) -> str:
    """One-line artist context for preview strip; focus highlights the control that changed."""
    if focus == "lighting":
        return _LIGHTING_HINTS.get(lighting, f"Lighting: {lighting}")
    if focus == "power":
        return _POWER_HINTS.get(power, f"Power: {power}")
    if focus == "night_lights":
        return (
            "Window emissive on — night still uses lit glass."
            if night_lights
            else "Window emissive off — night still stays dark."
        )
    if focus == "damage_state":
        return _DAMAGE_HINTS.get(damage_state, f"Damage: {damage_state}")
    if focus == "damage":
        pct = int(round(float(damage) * 100))
        return f"Wear slider {pct}% — preview blends toward Damaged chip above ~25%."
    if focus == "fill":
        return _FILL_HINTS.get(fill, f"Fill: {fill}")
    if focus == "material":
        if wall_material.strip():
            return f"Wall material override `{wall_material.strip()}` on preview merge only until Apply."
        return "No material override — snapshot placement materials stay."

    parts = [
        _LIGHTING_HINTS.get(lighting, lighting).split("—")[0].strip(),
        _POWER_HINTS.get(power, power).split("—")[0].strip(),
        _DAMAGE_HINTS.get(damage_state, damage_state).split("—")[0].strip(),
        _FILL_HINTS.get(fill, fill).split("—")[0].strip(),
    ]
    if wall_material.strip():
        parts.append(f"mat `{wall_material.strip()}`")
    return " · ".join(parts)


def _normalize_layers(layers: dict[str, Any]) -> dict[str, Any]:
    lighting = layers.get("lighting") or {}
    damage = layers.get("damage") or {}
    fill = layers.get("fill") or {}
    material = layers.get("material") or {}
    out: dict[str, Any] = {
        "lighting": {
            "lighting": str(lighting.get("lighting") or "day"),
            "power": str(lighting.get("power") or "off"),
            "night_lights": bool(lighting.get("night_lights")),
        },
        "damage": {
            "state": str(damage.get("state") or "clean"),
            "damage": round(float(damage.get("damage") or 0.0), 3),
        },
        "fill": {"fill": str(fill.get("fill") or "empty")},
    }
    mat = material.get("wall_material")
    if mat:
        out["material"] = {"wall_material": str(mat)}
    return out
