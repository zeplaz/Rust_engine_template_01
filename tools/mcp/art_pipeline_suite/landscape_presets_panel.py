"""APS-E2 — Landscape Presets tab (Option D mockup; not Catalog reuse)."""

from __future__ import annotations

import json
import tkinter as tk
from tkinter import ttk

from rust_engine_mcp.landscape_preset_browse import (
    list_landscape_presets,
    preset_path,
    preset_summary,
    validate_landscape_preset,
)
from rust_engine_mcp.paths import repo_root

from .aps_inline_feedback import set_inline_status
from .aps_scroll import attach_wheel_area, bind_debounced_scrollregion, canvas_yscroll
from .aps_theme import FONT_HINT, FONT_SMALL
from .metadata_flow_panel import MetadataFlowPanel
from .state import SuiteState

_DISPLAY_STRINGS_REL = "assets/configs/landscape/presets/_display_strings_v1.json"


def _load_display_strings() -> dict[str, str]:
    path = repo_root() / _DISPLAY_STRINGS_REL
    if not path.is_file():
        return {}
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {}
    if not isinstance(data, dict):
        return {}
    return {str(k): str(v) for k, v in data.items()}


def _topology_plain_summary(preset_id: str) -> str:
    path = preset_path(preset_id)
    if not path.is_file():
        return "Topologies: (missing preset file)"
    doc = json.loads(path.read_text(encoding="utf-8"))
    graph = doc.get("topology_graph") if isinstance(doc.get("topology_graph"), dict) else {}
    nodes = graph.get("nodes") or []
    kinds: list[str] = []
    for node in nodes:
        if not isinstance(node, dict):
            continue
        kind = str(node.get("kind") or node.get("type") or "").strip()
        if kind:
            kinds.append(kind.replace("_", " ").lower())
    kinds_plain = ", ".join(sorted(set(kinds))[:6]) or "(none)"
    required = doc.get("required_topologies") or []
    req_join = ", ".join(str(x) for x in required[:6]) if required else "—"
    max_depth = graph.get("max_nested_depth") or doc.get("max_nested_depth") or "—"
    return (
        f"Topologies: {kinds_plain} ({len(set(kinds))} kinds)\n"
        f"Nested depth: {max_depth} · Required: {req_join}"
    )


def _ship_badge(preset_id: str, validate_ok: bool | None) -> str:
    path = preset_path(preset_id)
    if not path.is_file():
        return "Draft"
    doc = json.loads(path.read_text(encoding="utf-8"))
    meta = doc.get("_meta") if isinstance(doc.get("_meta"), dict) else {}
    if meta.get("not_a_ship_target"):
        return "Teach"
    if validate_ok is True:
        return "Ship"
    return "Draft"


class LandscapePresetsPanel(ttk.Frame):
    """Browse landscape grammar presets — wireframe Presets tab (design_aps_uiux_style_quality §2.3)."""

    def __init__(self, master: tk.Misc, state: SuiteState, *, on_select, on_log) -> None:
        super().__init__(master, padding=4)
        self.state = state
        self._on_select = on_select
        self._on_log = on_log
        self._preset_ids: list[str] = []
        self._display = _load_display_strings()
        self._build()
        self.refresh_list()

    def _build(self) -> None:
        self.metadata_flow = MetadataFlowPanel(self, context="landscape_presets")
        self.metadata_flow.pack(fill=tk.X, pady=(0, 6))
        bar = ttk.Frame(self)
        bar.pack(fill=tk.X, pady=(0, 4))
        ttk.Label(bar, text="Landscape presets", font=("Segoe UI", 9, "bold")).pack(side=tk.LEFT)
        refresh_btn = ttk.Button(bar, text="Refresh", command=self.refresh_list)
        refresh_btn.pack(side=tk.RIGHT)
        validate_btn = ttk.Button(bar, text="Validate preset", command=self._validate_selected)
        validate_btn.pack(side=tk.RIGHT, padx=(0, 8))

        list_wrap = ttk.Frame(self)
        list_wrap.pack(fill=tk.BOTH, expand=True)
        scroll = ttk.Scrollbar(list_wrap, orient=tk.VERTICAL)
        scroll.pack(side=tk.RIGHT, fill=tk.Y)
        self._canvas = tk.Canvas(list_wrap, highlightthickness=0, yscrollcommand=scroll.set)
        self._canvas.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        scroll.configure(command=self._canvas.yview)
        self._inner = ttk.Frame(self._canvas)
        self._win = self._canvas.create_window((0, 0), window=self._inner, anchor=tk.NW)
        self._canvas.bind("<Configure>", lambda e: self._canvas.itemconfigure(self._win, width=e.width))
        bind_debounced_scrollregion(self._canvas, self._inner)
        attach_wheel_area(
            self._canvas,
            self._inner,
            on_scroll_y=canvas_yscroll(self._canvas),
            area_id=f"aps-landscape-presets-{id(self)}",
        )

        detail = ttk.LabelFrame(self, text="Must-read (DMCP-E2 preset QC)", padding=6)
        detail.pack(fill=tk.X, pady=(8, 0))
        self._q1 = tk.StringVar(value="Preset name: —")
        self._q2 = tk.StringVar(value="District read: —")
        self._q3 = tk.StringVar(value="Topology summary: —")
        self._q4 = tk.StringVar(value="Pressure headline: —")
        self._q5 = tk.StringVar(value="Ship status: —")
        for var in (self._q1, self._q2, self._q3, self._q4, self._q5):
            ttk.Label(detail, textvariable=var, wraplength=720, justify=tk.LEFT, font=FONT_SMALL).pack(
                anchor=tk.W, pady=1
            )
        self._validate_var = tk.StringVar(value="Validator: select a preset")
        self._validate_lbl = ttk.Label(detail, textvariable=self._validate_var, font=FONT_HINT)
        self._validate_lbl.pack(anchor=tk.W, pady=(6, 0))

    def refresh_list(self) -> None:
        body = list_landscape_presets()
        ship = [str(x) for x in (body.get("ship_presets") or []) if x]
        topology = [str(x) for x in (body.get("topology_presets") or []) if x]
        seen: set[str] = set()
        self._preset_ids = []
        for pid in ship + topology:
            if pid not in seen:
                seen.add(pid)
                self._preset_ids.append(pid)
        for child in self._inner.winfo_children():
            child.destroy()
        for pid in self._preset_ids:
            title = self._display.get(pid, pid.replace("_", " ").title())
            btn = ttk.Button(
                self._inner,
                text=title,
                command=lambda p=pid: self._select_preset(p),
            )
            btn.pack(fill=tk.X, pady=1, padx=2)
        self._on_log(f"landscape presets · listed {len(self._preset_ids)}")

    def _select_preset(self, preset_id: str) -> None:
        self.state.selected_landscape_preset_id = preset_id
        self.state.landscape_preset_validate_ok = None
        title = self._display.get(preset_id, preset_id)
        summary = preset_summary(preset_id)
        path = preset_path(preset_id)
        district = "—"
        pressure = "—"
        if path.is_file():
            doc = json.loads(path.read_text(encoding="utf-8"))
            prog = doc.get("landscape_program") if isinstance(doc.get("landscape_program"), dict) else {}
            district = str(prog.get("district_ref") or prog.get("district_class") or doc.get("district") or "—")
            land_dna = doc.get("land_dna") if isinstance(doc.get("land_dna"), dict) else {}
            lambdas = [f"{k}={v}" for k, v in list(land_dna.items())[:2]]
            pressure = " · ".join(lambdas) if lambdas else "Moderate disturbance"
        self._q1.set(f"Preset name: {title} ({preset_id})")
        self._q2.set(f"District read: {district}")
        self._q3.set(f"Topology summary: {_topology_plain_summary(preset_id).replace(chr(10), ' ')}")
        self._q4.set(f"Pressure headline: {pressure}")
        badge = _ship_badge(preset_id, None)
        self._q5.set(f"Ship status: {badge}")
        self._validate_var.set(
            f"Validator: schema {summary.get('validate_status', '—')} — run Validate preset"
        )
        self._on_select(preset_id)

    def _validate_selected(self) -> None:
        pid = self.state.selected_landscape_preset_id
        if not pid:
            set_inline_status(self._validate_lbl, self._validate_var, "Validator: select a preset first", ok=None)
            return
        report = validate_landscape_preset(pid, compression_level=3)
        ok = report.status == "passed"
        self.state.landscape_preset_validate_ok = ok
        badge = _ship_badge(pid, ok)
        self._q5.set(f"Ship status: {badge}")
        prefix = "PASS:" if ok else "FAIL:"
        set_inline_status(
            self._validate_lbl,
            self._validate_var,
            f"{prefix} landscape_grammar — {report.summary}",
            ok=ok,
        )
        self._on_log(f"validate landscape_grammar · {pid} · {report.status}")
