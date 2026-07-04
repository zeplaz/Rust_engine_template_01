"""BQ-C1 module geometric contract — Python constants mirrored from module_contract_v1.json."""

from __future__ import annotations

from typing import Final

GRID_UNIT_M: Final[float] = 4.0
FLOOR_HEIGHT_M: Final[float] = 3.0
PIVOT_CONVENTION: Final[str] = "bottom_center"
SEAM_TOLERANCE_M: Final[float] = 0.01
CONTRACT_JSON_REL: Final[str] = "tools/mcp/schemas/module_contract_v1.json"
EDGE_SOCKET_NAMES: Final[tuple[str, ...]] = ("left", "right", "top", "bottom")


def grid_units_from_width_m(width_m: float) -> int:
    """Whole grid cells spanned by a module width in meters."""
    return max(1, int(round(width_m / GRID_UNIT_M)))


def standard_wall_height_m() -> float:
    return FLOOR_HEIGHT_M
