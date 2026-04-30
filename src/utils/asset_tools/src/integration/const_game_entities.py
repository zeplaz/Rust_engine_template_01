"""
Read human-editable `.const` text under `assets/game_entities/` (legacy toolchain parity).

Used for designer preview in the asset editor; the game may load these by other means.
"""

from __future__ import annotations

import re
from pathlib import Path
from typing import List, Optional

from ..repo_paths import game_entities_consts_file

# Legacy regex target: `/:Power:voltages:(0.5,0.65,...;unit: MV)`
_POWER_VOLTAGES_PATTERN = re.compile(r":Power:voltages:\(([^;]+)", re.MULTILINE)


def load_power_voltage_levels_raw(const_path: Optional[Path] = None) -> List[str]:
    """Return voltage tokens (numeric strings, MV) from the const file, or []."""
    path = const_path or game_entities_consts_file
    if not path.is_file():
        return []
    text = path.read_text(encoding="utf-8", errors="replace")
    m = _POWER_VOLTAGES_PATTERN.search(text)
    if not m:
        return []
    chunk = m.group(1).strip()
    return [part.strip() for part in chunk.split(",") if part.strip()]
