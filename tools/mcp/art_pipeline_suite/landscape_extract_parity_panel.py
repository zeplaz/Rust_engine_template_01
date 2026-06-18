"""APS-E5 — read-only engine extract / map-stamp parity callout (States tab)."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk
from typing import Any

from rust_engine_mcp.aps_veg_extract_parity import check_veg_extract_parity

from .aps_inline_feedback import apply_status_atom, set_inline_status
from .aps_theme import FONT_SMALL, FONT_UI
from .aps_tooltips import bind_aps_tooltip

_VEG_ENGINE_READ_COPY = (
    "The game reads vegetation state (growth + fire) to pick the right tile from the landscape atlas. "
    "Authored states live in the catalog file, not in Blender."
)


class LandscapeExtractParityPanel(ttk.LabelFrame):
    """Read-only parity QC — APS must not write extract or ActiveBurn."""

    def __init__(self, master: tk.Misc, *, on_log) -> None:
        super().__init__(master, text="Engine read check (vegetation)", padding=6)
        self._on_log = on_log
        self._summary_var = tk.StringVar(value="")
        self._detail_var = tk.StringVar(value="")
        self._validation = tk.StringVar(value="")
        self._last: dict[str, Any] | None = None
        self._build()

    def _build(self) -> None:
        path_lbl = ttk.Label(
            self,
            text=_VEG_ENGINE_READ_COPY,
            wraplength=720,
            justify=tk.LEFT,
            font=FONT_SMALL,
        )
        path_lbl.pack(anchor=tk.W, pady=(0, 4))
        bind_aps_tooltip(path_lbl, "veg_extract_authority")

        row = ttk.Frame(self)
        row.pack(fill=tk.X, pady=(0, 4))
        ttk.Button(row, text="Check parity", command=self.refresh_parity).pack(side=tk.LEFT)
        self._status_lbl = ttk.Label(row, textvariable=self._summary_var, font=FONT_UI)
        self._status_lbl.pack(side=tk.LEFT, padx=(8, 0))

        ttk.Label(self, textvariable=self._detail_var, font=FONT_SMALL, wraplength=720).pack(
            anchor=tk.W
        )
        self._validator_lbl = ttk.Label(self, textvariable=self._validation, font=FONT_SMALL)
        self._validator_lbl.pack(anchor=tk.W, pady=(4, 0))

    def refresh_parity(self) -> dict[str, Any]:
        body = check_veg_extract_parity()
        self._last = body
        green = bool(body.get("parity_green"))
        if green:
            detail = f"{body.get('authored_count')} authored veg keys resolve"
            apply_status_atom(self._status_lbl, self._summary_var, "pass", word="valid", detail=detail)
        else:
            missing = body.get("missing_from_resolver") or []
            detail = "authored keys not consumable by engine resolver"
            if missing:
                detail += f" ({', '.join(missing[:3])}{'…' if len(missing) > 3 else ''})"
            apply_status_atom(self._status_lbl, self._summary_var, "fail", word="blocked", detail=detail)
        self._detail_var.set(
            f"Resolver keys: {body.get('resolver_count')} · "
            f"Extract sample keys: {len(body.get('extract_sample_keys') or [])} · "
            f"Authority: {body.get('engine_authority')}"
        )
        set_inline_status(
            self._validator_lbl,
            self._validation,
            "Parity green — burn tiles will resolve in game extract path"
            if green
            else "Some states won't load in-game yet — flag this to engineering before publishing.",
            ok=green,
        )
        self._on_log(f"veg extract parity · {'valid' if green else 'blocked'}")
        return body

    def last_report(self) -> dict[str, Any] | None:
        return self._last
