"""APS-UX-NONBLOCK-001 + SCROLL-001 witness."""

from __future__ import annotations

import json
import re
from pathlib import Path

from .paths import repo_root

APS_UX_NONBLOCK_WITNESS = "debug_runs/aps_ux_nonblock_001_live.json"

# Modal allowlist per aps_ux_professional_polish_rules_v1.md §2
MODAL_ALLOWLIST = (
    "messagebox.askyesno",  # P0 proceed / assembly mismatch confirm
    "messagebox.showerror",  # sub-dialog add-profile only (parent=dlg)
)

PANELS_MIGRATED = [
    "assembly_panel.py",
    "atlas_panel.py",
    "variants_panel.py",
    "catalog.py",
    "material_library_widget.py",
    "grammar_iterate_panel.py",
    "assembly_preview_panel.py",
    "app.py",
]


def _count_messageboxes(suite: Path) -> dict[str, int | list[str]]:
    per_file: dict[str, int] = {}
    calls: list[str] = []
    pat = re.compile(r"messagebox\.(show\w+|ask\w+)")
    for name in PANELS_MIGRATED:
        path = suite / name
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8")
        matches = pat.findall(text)
        per_file[name] = len(matches)
        for m in matches:
            calls.append(f"{name}: messagebox.{m}")
    return {"per_file": per_file, "total": sum(per_file.values()), "calls": calls}


def refresh_aps_ux_nonblock_witness() -> bool:
    root = repo_root()
    suite = root / "tools/mcp/art_pipeline_suite"
    counts = _count_messageboxes(suite)
    catalog = (suite / "catalog.py").read_text(encoding="utf-8")
    scroll_ok = "attach_wheel_area" in catalog and "canvas_yscroll" in catalog
    async_witness = root / "debug_runs/aps_ux_async_001_live.json"
    async_green = False
    if async_witness.is_file():
        async_green = json.loads(async_witness.read_text(encoding="utf-8")).get("ok") is True

    # Allowlist: assembly P0 askyesno, variants bake mismatch askyesno, material add-profile dlg error
    allowlisted = sum(
        1
        for c in counts["calls"]
        if "askyesno" in c or ("material_library_widget" in c and "showerror" in c)
    )
    routine_remaining = int(counts["total"]) - allowlisted

    green = routine_remaining == 0 and scroll_ok and async_green

    payload = {
        "gate_id": "APS-UX-NONBLOCK-001",
        "ok": green,
        "green": green,
        "messagebox_total": counts["total"],
        "messagebox_allowlisted": allowlisted,
        "messagebox_routine_remaining": routine_remaining,
        "messagebox_per_file": counts["per_file"],
        "messagebox_calls": counts["calls"],
        "scroll_catalog_wheel": scroll_ok,
        "async_prerequisite_green": async_green,
        "panels_migrated": PANELS_MIGRATED,
    }
    out = root / APS_UX_NONBLOCK_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return green
