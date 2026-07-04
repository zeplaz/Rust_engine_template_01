"""APSR-P1 — shared assembly panel helpers."""
from __future__ import annotations

from rust_engine_mcp.aps_grammar_labels import human_label

MATERIAL_AUTHORITY_COPY = (
    "The material you assign here is saved on each piece. The game and the preview both read it "
    "from this Assembly — not from Catalog tags or the Blender viewport. So: assign here, save, "
    "and it shows up everywhere."
)


def is_dark_color(hex_color: str) -> bool:
    try:
        h = hex_color.lstrip("#")
        if len(h) != 6:
            return False
        r, g, b = int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16)
        return (0.299 * r + 0.587 * g + 0.114 * b) < 140
    except (ValueError, TypeError):
        return False


def grammar_combo_maps(ids: list[str]) -> tuple[list[str], dict[str, str]]:
    labels = [human_label(i) for i in ids if i]
    label_to_id = {human_label(i): i for i in ids if i}
    return labels, label_to_id
