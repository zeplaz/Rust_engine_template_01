"""APS-MAT-AUTH-UI-001 — plain-language material authority hints for APS."""

from __future__ import annotations

from typing import Any

from .aps_validator_plain import format_p0_display, plain_validation_lines
from .validators.report import ValidationReport

ENGINE_READ_PATH = (
    "Runtime: placement.material_profile → material registry (assets/materials/textures/<id>/) "
    "→ worker bake / Bevy preview bind → render extract. "
    "Assembly snapshot is authority — not Catalog sidecar or Blender viewport."
)

__all__ = [
    "ENGINE_READ_PATH",
    "format_p0_display",
    "plain_validation_lines",
    "count_missing_material_profiles",
    "save_hint",
]


def count_missing_material_profiles(snapshot: dict[str, Any] | None) -> tuple[int, int]:
    if not snapshot:
        return 0, 0
    placements = list(snapshot.get("module_placements") or [])
    total = len(placements)
    missing = sum(
        1
        for row in placements
        if isinstance(row, dict) and not str(row.get("material_profile") or "").strip()
    )
    return missing, total


def save_hint(snapshot: dict[str, Any] | None) -> str:
    missing, total = count_missing_material_profiles(snapshot)
    if total == 0:
        return "No placements on snapshot."
    if missing:
        return f"{missing} of {total} placements missing material_profile — assign before ship."
    return f"All {total} placements have material_profile."
