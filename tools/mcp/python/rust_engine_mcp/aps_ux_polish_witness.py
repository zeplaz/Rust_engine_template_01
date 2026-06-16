"""APS-UX-DENSITY-001 + APS-UX-TOKENS-001 witness."""

from __future__ import annotations

import json
import re
from pathlib import Path

from .paths import repo_root

APS_UX_DENSITY_WITNESS = "debug_runs/aps_ux_density_001_live.json"
APS_UX_TOKENS_WITNESS = "debug_runs/aps_ux_tokens_001_live.json"

DENSITY_MARKERS = (
    ("CollapsibleSection", "assembly_panel.py"),
    ("Semantic & variant tags", "assembly_panel.py"),
    ("Grammar inspector", "assembly_panel.py"),
    ("_refresh_collapsible_titles", "assembly_panel.py"),
    ("set_title", "aps_collapsible.py"),
    ("Iterate grammar (advanced)", "assembly_panel.py"),
    ("Agent patch strip (advanced)", "variants_panel.py"),
)

TOKEN_MARKERS = (
    ("aps_theme.py", "AUTHORITY_STRIP"),
    ("FONT_UI", "aps_theme.py"),
    ("_build_authority_strip", "app.py"),
)


def _suite_text(name: str) -> str:
    path = repo_root() / "tools/mcp/art_pipeline_suite" / name
    return path.read_text(encoding="utf-8") if path.is_file() else ""


def refresh_aps_ux_density_witness() -> bool:
    wiring = {}
    for marker, fname in DENSITY_MARKERS:
        text = _suite_text(fname)
        wiring[f"{fname}:{marker}"] = marker in text
    footprint_min = "set_initial_pane_widths" in _suite_text("assembly_panel.py")
    green = all(wiring.values()) and footprint_min
    payload = {
        "gate_id": "APS-UX-DENSITY-001",
        "ok": green,
        "green": green,
        "wiring": wiring,
        "assembly_pane_stretch": footprint_min,
    }
    out = repo_root() / APS_UX_DENSITY_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return green


def refresh_aps_ux_tokens_witness() -> bool:
    theme = _suite_text("aps_theme.py")
    app = _suite_text("app.py")
    mat = _suite_text("material_library_widget.py")
    consolas8 = len(re.findall(r'Consolas", 8\)|Consolas", 8,', app + mat + _suite_text("atlas_preview_panel.py")))
    segoe8 = len(re.findall(r'Segoe UI", 8\)', app + mat + _suite_text("slot_preview_panel.py")))
    status_word_first = "def _status_label" in mat and "_status_text" in mat
    green = (
        "AUTHORITY_STRIP" in theme
        and "_build_authority_strip" in app
        and status_word_first
        and consolas8 == 0
    )
    payload = {
        "gate_id": "APS-UX-TOKENS-001",
        "ok": green,
        "green": green,
        "authority_strip": "_build_authority_strip" in app,
        "theme_module": (repo_root() / "tools/mcp/art_pipeline_suite/aps_theme.py").is_file(),
        "material_status_word_first": status_word_first,
        "consolas_8_remaining": consolas8,
        "segoe_8_remaining": segoe8,
    }
    out = repo_root() / APS_UX_TOKENS_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return green


def refresh_aps_ux_polish_witnesses() -> dict[str, bool]:
    return {
        "density": refresh_aps_ux_density_witness(),
        "tokens": refresh_aps_ux_tokens_witness(),
    }
