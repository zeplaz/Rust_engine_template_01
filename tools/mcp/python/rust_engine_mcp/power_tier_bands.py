"""DES-POWER-TIER-001 — designer units → power tier labels (catalog authority)."""

from __future__ import annotations


def power_tier_from_units(units: float, *, utility_role: str | None = None, power_generation: float = 0) -> str:
    """Map catalog power fields to APS tier word — never override from grammar alone."""
    if utility_role:
        return "grid"
    if power_generation > units and utility_role == "power_plant":
        return "grid"
    if units <= 30:
        return "light"
    if units <= 80:
        return "medium"
    if units <= 200:
        return "heavy"
    return "heavy"
