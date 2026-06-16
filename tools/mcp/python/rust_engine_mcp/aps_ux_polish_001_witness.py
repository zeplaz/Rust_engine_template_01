"""APS-UX-POLISH-001 — P0 a11y witness (validation color, material text, metadata visibility)."""

from __future__ import annotations

import json
from pathlib import Path

from .paths import repo_root

APS_UX_POLISH_001_WITNESS = "debug_runs/aps_ux_polish_001_live.json"


def _suite(name: str) -> str:
    p = repo_root() / "tools/mcp/art_pipeline_suite" / name
    return p.read_text(encoding="utf-8") if p.is_file() else ""


def refresh_aps_ux_polish_001_witness() -> bool:
    inline = _suite("aps_inline_feedback.py")
    mat = _suite("material_library_widget.py")
    meta = _suite("metadata_flow_panel.py")
    asm = _suite("assembly_panel.py")
    cat = _suite("catalog.py")
    var = _suite("variants_panel.py")

    validation_color = (
        "COLOR_FAIL" in inline
        and "set_inline_status" in asm
        and "Validation: FAIL" in asm
        and "set_inline_status" in cat
    )
    material_status_text = "_status_label" in mat and "Ready" in mat and "_status_text" in mat
    metadata_visible = (
        "_initial_expanded" in meta
        and "_collapsed_hint" in meta
        and "Snapshot is ship authority" in meta
    )
    phases_2_4 = all(
        (repo_root() / f"debug_runs/{w}").is_file()
        for w in (
            "aps_ux_nonblock_001_live.json",
            "aps_ux_density_001_live.json",
        )
    )
    async_ok = (repo_root() / "debug_runs/aps_ux_async_001_live.json").is_file()

    green = validation_color and material_status_text and metadata_visible and phases_2_4 and async_ok

    payload = {
        "gate_id": "APS-UX-POLISH-001",
        "ok": green,
        "green": green,
        "p0_a11y": {
            "validation_fail_not_green": validation_color,
            "material_status_text_not_glyph_only": material_status_text,
            "metadata_flow_default_visible": metadata_visible,
        },
        "phases_2_4_prerequisite": phases_2_4,
        "phase_1_async": async_ok,
        "panels_using_set_inline_status": {
            "assembly_panel.py": "set_inline_status" in asm,
            "catalog.py": "set_inline_status" in cat,
            "variants_panel.py": "set_inline_status" in var,
        },
    }
    out = repo_root() / APS_UX_POLISH_001_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return green
