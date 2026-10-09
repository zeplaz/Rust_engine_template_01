"""DES-POWER-TIER-001 — designer units → power tier labels (catalog authority)."""

from __future__ import annotations


HEAVY_BAND_MAX = 200.0


def _finite_power(units: float, *, field: str) -> float:
    """Reject bool and non-finite values before any band comparison.

    ``True`` is an ``int`` in Python, so ``0 <= True <= 30`` would label it light.
    NaN fails every comparison and used to fall through to heavy.
    """
    if isinstance(units, bool) or not isinstance(units, (int, float)):
        raise ValueError(f"{field} must be a finite number")
    value = float(units)
    if value != value or value == float("inf") or value == float("-inf"):
        raise ValueError(f"{field} must be a finite number")
    return value


def power_tier_from_units(units: float, *, utility_role: str | None = None, power_generation: float = 0) -> str:
    """Map catalog power fields to APS tier word — never override from grammar alone.

    ``utility_role`` set, or a power plant whose generation exceeds consumption,
    is grid. Consumption above the heavy band clamps to heavy (see warning).
    """
    units = _finite_power(units, field="power_consumption")
    power_generation = _finite_power(power_generation, field="power_generation")
    generation_grid = utility_role == "power_plant" and power_generation > units
    if utility_role or generation_grid:
        return "grid"
    if units <= 30:
        return "light"
    if units <= 80:
        return "medium"
    if units <= HEAVY_BAND_MAX:
        return "heavy"
    return "heavy"


def power_tier_band_warning(units: float, *, utility_role: str | None = None) -> str | None:
    """DES-POWER-TIER-001 — flag consumption outside the published bands."""
    units = _finite_power(units, field="power_consumption")
    if utility_role:
        return None
    if units < 0:
        return "power_consumption is negative; treated as light"
    if units > HEAVY_BAND_MAX:
        return (
            f"power_consumption {units:g} exceeds heavy band {HEAVY_BAND_MAX:g}; "
            "clamped to heavy"
        )
    return None
