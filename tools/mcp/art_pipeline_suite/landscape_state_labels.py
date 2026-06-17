"""DES-APS-STATE-AXIS-LABELS-001 — v2 label tables (schema enum ↔ UI display)."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from .aps_inline_feedback import validation_foreground

# Schema enums — vegetation_variant_catalog_v1.schema.json axes.succession_stages
SUCCESSION_STAGE_ENUMS: tuple[str, ...] = (
    "Grass",
    "Shrub",
    "Sapling",
    "Canopy",
    "OldGrowth",
    "BurnScar",
)

REGROWTH_MACRO_ENUMS: tuple[str, ...] = (
    "None",
    "Scar",
    "Nuclei",
    "Front",
    "Closing",
    "Mature",
)


@dataclass(frozen=True, slots=True)
class AxisLabelRow:
    enum: str
    ui_label: str
    short: str
    tooltip: str


SUCCESSION_STAGE_ROWS: tuple[AxisLabelRow, ...] = (
    AxisLabelRow("Grass", "Pioneer grass", "grass", "Bare / pioneer cover after gap"),
    AxisLabelRow("Shrub", "Shrub thicket", "shrub", "Low woody regrowth"),
    AxisLabelRow("Sapling", "Young stems", "sapling", "Establishing canopy"),
    AxisLabelRow("Canopy", "Closed canopy", "canopy", "Mature closed cover"),
    AxisLabelRow("OldGrowth", "Old growth", "old growth", "Long-horizon climax stage"),
    AxisLabelRow("BurnScar", "Burn scar", "burn scar", "Persistent post-fire scar on succession graph"),
)

REGROWTH_MACRO_ROWS: tuple[AxisLabelRow, ...] = (
    AxisLabelRow("None", "No regrowth", "none", "Undisturbed macro phase"),
    AxisLabelRow("Scar", "Scar hold", "scar", "Ash scar before nuclei"),
    AxisLabelRow("Nuclei", "Regrowth nuclei", "nuclei", "Spot regrowth seeds"),
    AxisLabelRow("Front", "Regrowth front", "front", "Advancing edge"),
    AxisLabelRow("Closing", "Canopy closing", "closing", "Gaps filling in"),
    AxisLabelRow("Mature", "Regrowth mature", "mature", "Hands off to succession stage"),
)

_UI_BY_ENUM: dict[str, str] = {
    **{r.enum: r.ui_label for r in SUCCESSION_STAGE_ROWS},
    **{r.enum: r.ui_label for r in REGROWTH_MACRO_ROWS},
}

_SHORT_BY_ENUM: dict[str, str] = {
    **{r.enum: r.short for r in SUCCESSION_STAGE_ROWS},
    **{r.enum: r.short for r in REGROWTH_MACRO_ROWS},
}


def ui_label_for_enum(enum: str) -> str:
    return _UI_BY_ENUM.get(enum, enum)


def enum_from_ui_label(label: str, *, rows: tuple[AxisLabelRow, ...]) -> str | None:
    for row in rows:
        if label == row.ui_label or label == row.enum:
            return row.enum
    return None


def combobox_display_values(rows: tuple[AxisLabelRow, ...]) -> list[str]:
    """Combobox dropdown text — UI label column."""
    return [r.ui_label for r in rows]


def combobox_enum_values(rows: tuple[AxisLabelRow, ...]) -> list[str]:
    """Authoritative schema enum values (parallel to display list)."""
    return [r.enum for r in rows]


def burn_frame_enum(frame_index: int) -> str:
    return f"veg_burn_{frame_index:02}"


def burn_preview_label(frame_index: int) -> str:
    key = burn_frame_enum(frame_index)
    if frame_index == 0:
        return f"Fire start ({key})"
    if frame_index == 7:
        return f"Fire end ({key})"
    if frame_index == 3:
        return f"Fire mid ({key})"
    return f"Fire frame {frame_index} ({key})"


def burn_preview_rows(frame_count: int = 8) -> list[tuple[str, str]]:
    """Return (enum, display) pairs for burn preview combobox."""
    count = max(1, min(int(frame_count), 16))
    return [(burn_frame_enum(i), burn_preview_label(i)) for i in range(count)]


def resolver_plain_label(entry: dict[str, Any]) -> str:
    resolver = entry.get("resolver") if isinstance(entry.get("resolver"), dict) else {}
    kind = str(resolver.get("kind") or "")
    if kind == "topology_kind":
        topo = str(resolver.get("topology_kind") or "Patch")
        return f"{topo} topology sprite"
    if kind == "active_burn_frame":
        idx = int(resolver.get("frame_index") or 0)
        return f"Active fire · frame {idx}"
    if kind == "regrowth_macro":
        phase = str(resolver.get("regrowth_macro_phase") or "None")
        return f"Regrowth · {ui_label_for_enum(phase)}"
    if kind == "succession_stage":
        stage = str(resolver.get("succession_stage") or "")
        return f"Succession · {ui_label_for_enum(stage)}"
    if kind == "default":
        return "Default fallback"
    return kind or "—"


def atlas_slot_label(entry: dict[str, Any]) -> str:
    atlas = entry.get("atlas") if isinstance(entry.get("atlas"), dict) else {}
    atlas_id = str(atlas.get("atlas_id") or "").strip()
    return atlas_id or "—"


def status_display(
    internal: str,
    *,
    catalog_ok: bool | None = None,
) -> tuple[str, bool | None]:
    """Map internal row status → (glyph+words, ok for validation_foreground)."""
    if internal == "blocked":
        return "○ blocked — no preset", None
    if internal == "await_grammar":
        return "◐ await grammar", None
    if internal == "scaffold":
        return "◐ scaffold", None
    if internal == "catalog_fail":
        return "✗ FAIL", False
    if internal == "catalog_ok" or internal == "valid":
        return "✓ valid", True
    if internal == "validate":
        return "○ pending", None
    return "○ pending", catalog_ok


def status_foreground(internal: str, *, catalog_ok: bool | None = None) -> str:
    _text, ok = status_display(internal, catalog_ok=catalog_ok)
    return validation_foreground(ok)


def inline_hint(
    *,
    has_preset: bool,
    grammar_ok: bool,
    catalog_ok: bool | None,
) -> str:
    if catalog_ok is False:
        return "✗ Catalog FAIL — fix rows before bake"
    if not has_preset:
        return "○ States pending — select a landscape preset on Presets tab"
    if not grammar_ok:
        return "◐ States blocked — generate grammar on Grammar tab"
    return "Bake states prepares LG-5 tile batch — then Pack LG-5 atlas on Flow bar"
