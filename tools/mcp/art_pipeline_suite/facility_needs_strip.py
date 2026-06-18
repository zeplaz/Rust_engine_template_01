"""CMCP-FACILITY-NEEDS-PANEL-001 — read-only process summary on Assembly tab."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk
from typing import Any

from rust_engine_mcp import building_grammar, grammar_facility_brief

from .aps_inline_feedback import power_tier_atom
from .aps_theme import COLOR_MUTED, COLOR_TEXT_SUBTLE, FONT_SMALL, FONT_UI, track_wraplength
from .state import ArtDomain

_STEP_LABELS: dict[str, str] = {
    "aggregate_mine": "Aggregate quarry",
    "cement_kiln": "Cement kiln",
    "concrete_mixer": "Concrete batching plant",
    "integrated_plant": "Integrated cement plant (legacy)",
    "bauxite_mine": "Bauxite mine",
    "alumina_refinery": "Alumina refinery",
    "aluminum_smelter": "Aluminum smelter",
    "aluminum_fabrication": "Aluminum fabrication",
}

_UTILITY_LABELS: dict[str, str] = {
    "substation": "Substation yard",
    "transformer": "Transformer pad",
    "power_plant": "Coal power plant",
}


def _step_label(brief: dict[str, Any]) -> str:
    catalog = brief.get("catalog") or {}
    utility = catalog.get("utility_role")
    if utility:
        return _UTILITY_LABELS.get(str(utility), str(utility).replace("_", " ").title())
    role = str(catalog.get("supply_chain_role") or "")
    return _STEP_LABELS.get(role, role.replace("_", " ").title() or "Process step")


def _line1(brief: dict[str, Any]) -> str:
    catalog = brief.get("catalog") or {}
    chain = brief.get("chain") or {}
    tier = str((brief.get("derived") or {}).get("power_tier_from_catalog") or catalog.get("power_tier") or "light")
    glyph, word, _fg, _bg = power_tier_atom(tier, detail=_step_label(brief))
    chain_name = chain.get("display_name")
    if catalog.get("utility_role"):
        return f"{glyph} {word}"
    if chain_name:
        return f"{glyph} {word} · {chain_name}"
    return f"{glyph} {word}"


def _line2(brief: dict[str, Any]) -> str:
    io = brief.get("io_summary") or {}
    parts: list[str] = []
    consumes = io.get("consumes_top3") or []
    produces = io.get("produces_top3") or []
    if consumes:
        parts.append(f"In: {', '.join(consumes)}")
    if produces:
        parts.append(f"Out: {', '.join(produces)}")
    return "  ·  ".join(parts)


class FacilityNeedsStrip(ttk.Frame):
    """DES-APS-FACILITY-NEEDS-001 — catalog authority only; no invented numbers."""

    def __init__(self, master: tk.Misc) -> None:
        super().__init__(master)
        self._tier = "G0"
        self._line1_var = tk.StringVar(value="")
        self._line2_var = tk.StringVar(value="")
        self._line3_var = tk.StringVar(value="")
        self._line1 = ttk.Label(self, textvariable=self._line1_var, font=FONT_UI, wraplength=880)
        self._line2 = ttk.Label(
            self, textvariable=self._line2_var, font=FONT_SMALL, foreground=COLOR_TEXT_SUBTLE, wraplength=880
        )
        self._line3 = ttk.Label(
            self, textvariable=self._line3_var, font=FONT_SMALL, foreground=COLOR_MUTED, wraplength=880
        )
        self._empty_var = tk.StringVar(value="○ Visual-only grammar — no process binding")
        self._empty = ttk.Label(
            self, textvariable=self._empty_var, font=FONT_SMALL, foreground=COLOR_MUTED, wraplength=880
        )
        track_wraplength(self, self._line1, minimum=480)
        track_wraplength(self, self._line2, minimum=480)
        track_wraplength(self, self._line3, minimum=480)
        track_wraplength(self, self._empty, minimum=480)

    def set_grammar_tier(self, tier: str) -> None:
        self._tier = str(tier or "G0").upper()

    def refresh(self, *, archetype_id: str | None, lane: str) -> None:
        for child in (self._line1, self._line2, self._line3, self._empty):
            child.pack_forget()
        if lane == ArtDomain.LANDSCAPE.value:
            return
        brief: dict[str, Any] | None = None
        if archetype_id:
            try:
                grammar = building_grammar.load_building_grammar_by_archetype(archetype_id)
                gid = str(grammar.get("grammar_id") or "")
                if gid and grammar.get("facility_binding"):
                    body = grammar_facility_brief.grammar_facility_brief(grammar_id=gid)
                    brief = body.get("brief")
            except (KeyError, FileNotFoundError, NotImplementedError):
                brief = None
        if not brief or not brief.get("facility_binding"):
            self._empty.pack(anchor=tk.W)
            return
        self._line1_var.set(_line1(brief))
        tier = self._tier
        if tier in ("G1", "G2", "G3", "G4"):
            self._line1.pack(anchor=tk.W)
            self._line2_var.set(_line2(brief))
            self._line2.pack(anchor=tk.W)
        elif tier == "G0":
            self._line1.pack(anchor=tk.W)
        if tier in ("G2", "G3", "G4"):
            catalog = (brief.get("catalog") or {}).get("catalog_id") or ""
            self._line3_var.set(f"catalog: {catalog}.json")
            self._line3.pack(anchor=tk.W)
