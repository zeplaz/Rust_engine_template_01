"""BQ-C1 module geometric contract — Python constants mirrored from module_contract_v1.json."""

from __future__ import annotations

import math
from typing import Final

GRID_UNIT_M: Final[float] = 4.0
FLOOR_HEIGHT_M: Final[float] = 3.0
PIVOT_CONVENTION: Final[str] = "bottom_center"
SEAM_TOLERANCE_M: Final[float] = 0.01
CONTRACT_JSON_REL: Final[str] = "tools/mcp/schemas/module_contract_v1.json"
EDGE_SOCKET_NAMES: Final[tuple[str, ...]] = ("left", "right", "top", "bottom")


def grid_units_from_width_m(width_m: float) -> int:
    """Whole grid cells spanned by a module width in meters.

    Half-cells round away from zero. ``round()`` uses banker's rounding, so
    10 m (2.5 cells) and 18 m (4.5 cells) would under-span.
    """
    if width_m <= 0:
        return 1
    return max(1, int(math.floor(float(width_m) / GRID_UNIT_M + 0.5)))


def standard_wall_height_m() -> float:
    return FLOOR_HEIGHT_M
