"""APS-UX-META-001 — where artist metadata flows into the engine."""

from __future__ import annotations

import json
import tkinter as tk
from tkinter import ttk

from rust_engine_mcp.paths import repo_root

_PREFS_PATH = repo_root() / "debug_runs/aps_ui_prefs.json"


def _load_prefs() -> dict:
    if not _PREFS_PATH.is_file():
        return {}
    try:
        data = json.loads(_PREFS_PATH.read_text(encoding="utf-8"))
        return data if isinstance(data, dict) else {}
    except (OSError, json.JSONDecodeError):
        return {}


def _save_prefs(prefs: dict) -> None:
    _PREFS_PATH.parent.mkdir(parents=True, exist_ok=True)
    _PREFS_PATH.write_text(json.dumps(prefs, indent=2) + "\n", encoding="utf-8")


def _initial_expanded(context: str) -> bool:
    prefs = _load_prefs()
    seen_key = f"metadata_flow_seen_{context}"
    if not prefs.get(seen_key):
        prefs[seen_key] = True
        prefs[f"metadata_flow_expanded_{context}"] = True
        _save_prefs(prefs)
        return True
    return bool(prefs.get(f"metadata_flow_expanded_{context}", False))


class MetadataFlowPanel(ttk.LabelFrame):
    """Collapsible guide: snapshot authority → worker → atlas → runtime."""

    def __init__(self, master: tk.Misc, *, context: str = "assembly") -> None:
        super().__init__(master, text="Metadata → engine (ARCH-MAT-001)", padding=6)
        self._context = context
        self._expanded = tk.BooleanVar(value=_initial_expanded(context))
        head = ttk.Frame(self)
        head.pack(fill=tk.X)
        chk = ttk.Checkbutton(
            head,
            text="Show how tags & materials reach runtime",
            variable=self._expanded,
            command=self._toggle,
        )
        chk.pack(side=tk.LEFT)
        from .aps_tooltips import bind_aps_tooltip

        bind_aps_tooltip(chk, "meta_flow")
        self._collapsed_hint = ttk.Label(
            head,
            text="Snapshot is ship authority — expand for metadata flow diagram.",
            font=("Segoe UI", 9),
            foreground="#0a4a7a",
            wraplength=680,
        )
        self._body = ttk.Frame(self)
        self._text = tk.Text(
            self._body,
            height=10,
            wrap=tk.WORD,
            font=("Segoe UI", 9),
            background="#f8f8f8",
            relief=tk.FLAT,
        )
        self._text.pack(fill=tk.BOTH, expand=True)
        self._fill_content()
        if self._expanded.get():
            self._body.pack(fill=tk.BOTH, expand=True, pady=(6, 0))
        self._sync_collapsed_hint()

    def _sync_collapsed_hint(self) -> None:
        if self._expanded.get():
            self._collapsed_hint.pack_forget()
        else:
            self._collapsed_hint.pack(side=tk.LEFT, padx=(12, 0), fill=tk.X, expand=True)

    def _fill_content(self) -> None:
        blocks = {
            "assembly": """assembly_snapshot (AUTHORITY)
  material_profile  → Blender/Bevy worker applies at bake/preview (not assigned in DCC)
  semantic_tags     → procedural rules, facade/location filters in engine
  variant_tags      → variant_set expansion → tile states
  module_placements → GLB resolve + grid position → construction/render extract
  grammar_rule_chain → APS inspector only today; drives generator seed path

Validate (P0) before bake. Save snapshot after every material/tag edit.""",
            "materials": """material_profiles registry (assets/materials/profiles/)
  profile_id        → written on placement.material_profile in snapshot
  albedo/normal/...   → worker material bind + runtime lookup
  category            → APS browse only; engine uses profile_id string

Assign on Assembly tab — Materials tab does not ship alone.""",
            "catalog": """module index + AssetSpec sidecar
  module_id, category, batch_id → assembly module resolver
  tags in sidecar     → optional hints; assembly semantic_tags are ship truth
  GLB path            → placement.glb_path after generate/build

Validate GLB before adding modules to style packs / assemblies.""",
            "atlas": """atlas_meta.json + tile_map PNG
  variant_key, grid, uv → runtime tile atlas lookup (map stamp)
  tile_id / atlas_id    → registry registration (--register)

Independent of per-placement material_profile — building tile vs module materials.""",
            "variants": """variant_set JSON
  state keys (clean_day, damaged_…) → tile_batch variants → atlas cells
  Derived from assembly_id + snapshot variant_tags

Bake variants prepares tile_batch — does not replace snapshot authority.""",
        }
        body = blocks.get(self._context, blocks["assembly"])
        self._text.configure(state=tk.NORMAL)
        self._text.delete("1.0", tk.END)
        self._text.insert("1.0", body)
        self._text.configure(state=tk.DISABLED)

    def _toggle(self) -> None:
        if self._expanded.get():
            self._body.pack(fill=tk.BOTH, expand=True, pady=(6, 0))
        else:
            self._body.pack_forget()
        self._sync_collapsed_hint()
        prefs = _load_prefs()
        prefs[f"metadata_flow_expanded_{self._context}"] = self._expanded.get()
        _save_prefs(prefs)
