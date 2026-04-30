"""
Designer-facing role filters (legacy toolbox parity).

These labels describe *what kind* of asset is being authored. The canonical editor does not yet
mutually exclude nav pages based on them (legacy PySide UI did); roles are shown on Home for
workflow clarity and can drive save metadata later.
"""

from __future__ import annotations

# Same intent as legacy `utils/asset_tools/src/asset_config.py` MASTER_ENTITY_FILTER_ROLES
MASTER_ENTITY_FILTER_ROLES: tuple[str, ...] = (
    "Building",
    "Vehicle",
    "Transportable",
    "Power",
    "Productive",
    "Scenery",
)
