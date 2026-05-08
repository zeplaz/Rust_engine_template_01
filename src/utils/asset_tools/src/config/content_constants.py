"""
Designer-facing role filters (legacy toolbox parity).

These labels describe *what kind* of asset is being authored. The canonical editor does not yet
mutually exclude nav pages based on them (legacy PySide UI did); roles are shown on Home for
workflow clarity and can drive save metadata later.
"""

from __future__ import annotations

from .asset_config import ASSET_TYPES

# Single source: `ASSET_TYPES` in `asset_config.py` (S8 alignment).
MASTER_ENTITY_FILTER_ROLES: tuple[str, ...] = tuple(ASSET_TYPES)
