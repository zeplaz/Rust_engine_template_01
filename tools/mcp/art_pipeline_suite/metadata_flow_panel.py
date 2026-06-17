"""Where artist metadata flows into the engine."""

from __future__ import annotations

import json
import tkinter as tk
from tkinter import ttk

from rust_engine_mcp.paths import repo_root

_PREFS_PATH = repo_root() / "debug_runs/aps_ui_prefs.json"

_ASSEMBLY_FLOW = """What you save in this Assembly is the source of truth.
• Materials → used when the building is baked or previewed.
• Tags → drive how the engine filters and places pieces.
• Variant tags → expand into tile states later.
Run the Ship check before baking. Save after every material or tag change."""

_LANDSCAPE_STATES_FLOW = (
    "The game looks at each vegetation patch's growth stage and fire state to pick the "
    "matching tile from the landscape atlas. Those states are authored in the catalog file "
    "here — not in Blender."
)


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
    _ = context
    return False


class MetadataFlowPanel(ttk.LabelFrame):
    """Collapsible guide: Assembly authority → worker → atlas → runtime."""

    def __init__(self, master: tk.Misc, *, context: str = "assembly") -> None:
        super().__init__(master, text="Where this data goes", padding=6)
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
            text="The Assembly is the source of truth — expand for how data flows.",
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
            "assembly": _ASSEMBLY_FLOW,
            "materials": """Material library on disk.
Pick a material id and assign it on the Assembly step.
Materials you set here do not ship until saved on the Assembly.""",
            "catalog": """Module kit index and editable sidecar.
Tags here are hints only. The tags and materials you set in the Assembly are what actually ship.""",
            "atlas": """Packed tile sheet and atlas metadata.
Registers tile lookup for the map after you pass ship checks.""",
            "variants": """Variant layers expand into tile states for baking.
The Assembly remains the source of truth for materials and tags.""",
            "landscape_presets": """Browse and validate landscape presets.
The preset you select is what ships for layout and disturbance.""",
            "landscape_grammar": """Edit the landscape layout graph.
This is separate from the building footprint grid.""",
            "landscape_states": _LANDSCAPE_STATES_FLOW,
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
