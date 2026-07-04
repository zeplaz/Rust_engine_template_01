"""APS-E2-GRAMMAR-PANEL-001 — Landscape Grammar tab (topology graph; not Assembly)."""

from __future__ import annotations

import json
import tkinter as tk
from tkinter import ttk

from rust_engine_mcp.landscape_preset_browse import preset_path, validate_landscape_preset

from . import aps_theme
from .aps_inline_feedback import set_inline_status
from .aps_paned import add_pane, horizontal_paned
from .aps_theme import FONT_HINT, FONT_SMALL
from .aps_tooltips import bind_aps_tooltip
from .aps_tk import themed_text
from .aps_workflow_layout import workflow_intro, workflow_primary_row
from .state import SuiteState

_NODE_KIND_LABELS = {
    "NETWORK": "network",
    "CORRIDOR": "corridor",
    "RING": "ring",
    "PATCH": "patch",
    "CLUSTER": "cluster",
    "FRINGE": "fringe",
}


class LandscapeGrammarPanel(ttk.Frame):
    """Topology-graph workspace — design_aps_grammar_panel_v1.md (no footprint grid)."""

    def __init__(self, master: tk.Misc, state: SuiteState, *, on_log) -> None:
        super().__init__(master, padding=4)
        self.state = state
        self._on_log = on_log
        self._node_rows: list[dict] = []
        self._build()
        self.refresh_from_state()

    def _build(self) -> None:
        workflow_intro(
            self,
            "Inspect the topology graph for the selected preset — validate before States and Atlas.",
        )

        actions = workflow_primary_row(self)
        ttk.Label(actions, text="Grammar graph", font=("Segoe UI", 9, "bold")).pack(side=tk.LEFT, padx=(0, 8))
        validate_btn = ttk.Button(actions, text="Validate schema", command=self._validate_schema)
        validate_btn.pack(side=tk.LEFT, padx=(0, 6))
        bind_aps_tooltip(validate_btn, "landscape_validate_schema")
        self._validate_var = tk.StringVar(value="")
        self._validate_lbl = ttk.Label(actions, textvariable=self._validate_var, font=FONT_HINT)
        self._validate_lbl.pack(side=tk.LEFT, padx=6)

        paned = horizontal_paned(self)
        paned.pack(fill=tk.BOTH, expand=True, pady=4)

        left = ttk.Frame(paned, padding=4)
        add_pane(paned, left, weight=1, minsize=200)
        ttk.Label(left, text="Topology tree", font=FONT_SMALL).pack(anchor=tk.W)
        tree_wrap = ttk.Frame(left)
        tree_wrap.pack(fill=tk.BOTH, expand=True)
        tree_scroll = ttk.Scrollbar(tree_wrap, orient=tk.VERTICAL)
        tree_scroll.pack(side=tk.RIGHT, fill=tk.Y)
        self._tree = ttk.Treeview(tree_wrap, show="tree", yscrollcommand=tree_scroll.set)
        self._tree.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        tree_scroll.configure(command=self._tree.yview)
        self._tree.bind("<<TreeviewSelect>>", self._on_tree_select)

        mid = ttk.Frame(paned, padding=4)
        add_pane(paned, mid, weight=2, minsize=240)
        ttk.Label(mid, text="Graph preview (schematic)", font=FONT_SMALL).pack(anchor=tk.W)
        self._canvas = tk.Canvas(mid, height=180, background=aps_theme.COLOR_CARD_BG, highlightthickness=1)
        self._canvas.pack(fill=tk.BOTH, expand=True, pady=4)
        self._canvas.bind("<Button-1>", self._on_canvas_click)

        right = ttk.Frame(paned, padding=4)
        add_pane(paned, right, weight=1, minsize=220)
        ttk.Label(right, text="Selected node", font=FONT_SMALL).pack(anchor=tk.W)
        self._inspector = themed_text(
            right, height=10, wrap=tk.WORD, font=FONT_SMALL
        )
        self._inspector.pack(fill=tk.BOTH, expand=True)

        self._status_var = tk.StringVar(value="Pick a preset on the Presets tab first")
        ttk.Label(self, textvariable=self._status_var, font=FONT_HINT, foreground=aps_theme.COLOR_MUTED).pack(
            anchor=tk.W, pady=(4, 0)
        )

    def _on_tree_select(self, _event=None) -> None:
        sel = self._tree.selection()
        if not sel:
            return
        iid = sel[0]
        if iid.startswith("node:"):
            idx = int(iid.split(":")[1])
            self._show_node(self._node_rows[idx])

    def _on_canvas_click(self, event) -> None:
        items = self._canvas.find_closest(event.x, event.y)
        if not items:
            return
        tags = self._canvas.gettags(items[0])
        for tag in tags:
            if tag.startswith("node:"):
                idx = int(tag.split(":")[1])
                self._show_node(self._node_rows[idx])
                return

    def _show_node(self, node: dict) -> None:
        kind = str(node.get("kind") or node.get("type") or "node")
        kind_word = _NODE_KIND_LABELS.get(kind.upper(), kind.lower())
        stack = node.get("operator_stack") or []
        lines = [
            f"Kind: {kind_word}",
            f"ID: {node.get('id', '—')}",
            f"Scale: {node.get('scale_band', '—')}",
            f"Parent: {node.get('parent_id', '—')}",
            "",
            "Operator stack:",
        ]
        if isinstance(stack, list) and stack:
            for i, op in enumerate(stack[:6], 1):
                lines.append(f"  {i}. {op}")
        else:
            lines.append("  (scaffold — no operators authored)")
        self._inspector.configure(state=tk.NORMAL)
        self._inspector.delete("1.0", tk.END)
        self._inspector.insert("1.0", "\n".join(lines))
        self._inspector.configure(state=tk.DISABLED)

    def _draw_graph(self, nodes: list[dict]) -> None:
        self._canvas.delete("all")
        self._node_rows = [n for n in nodes if isinstance(n, dict)]
        if not self._node_rows:
            self._canvas.create_text(120, 60, text="(no topology nodes)", fill="#666")
            return
        x0, y0 = 24, 40
        dx = 90
        for idx, node in enumerate(self._node_rows[:6]):
            x = x0 + (idx % 3) * dx
            y = y0 + (idx // 3) * 50
            kind = str(node.get("kind") or "node")[:8]
            nid = str(node.get("id") or idx)[:12]
            tag = f"node:{idx}"
            self._canvas.create_rectangle(x, y, x + 72, y + 28, fill="#e8eef5", outline="#4a5568", tags=(tag,))
            self._canvas.create_text(x + 36, y + 14, text=f"{kind}", tags=(tag,))

    def refresh_from_state(self) -> None:
        pid = self.state.selected_landscape_preset_id
        self._tree.delete(*self._tree.get_children())
        self._node_rows = []
        self._canvas.delete("all")
        self._inspector.configure(state=tk.NORMAL)
        self._inspector.delete("1.0", tk.END)
        if not pid:
            self._inspector.insert("1.0", "Select a preset on Presets tab.")
            self._inspector.configure(state=tk.DISABLED)
            self._status_var.set("○ Grammar pending — no preset")
            return
        path = preset_path(pid)
        if not path.is_file():
            self._inspector.insert("1.0", f"Missing preset file for {pid}")
            self._inspector.configure(state=tk.DISABLED)
            return
        doc = json.loads(path.read_text(encoding="utf-8"))
        graph = doc.get("topology_graph") if isinstance(doc.get("topology_graph"), dict) else {}
        nodes = graph.get("nodes") or []
        root_id = self._tree.insert("", tk.END, text=f"preset: {pid}", open=True)
        for idx, node in enumerate(nodes):
            if not isinstance(node, dict):
                continue
            kind = str(node.get("kind") or node.get("type") or "NODE")
            label = _NODE_KIND_LABELS.get(kind.upper(), kind.lower())
            nid = str(node.get("id") or f"node_{idx}")
            self._tree.insert(root_id, tk.END, iid=f"node:{idx}", text=f"{label} · {nid}")
        self._draw_graph(nodes)
        land_dna = doc.get("land_dna") if isinstance(doc.get("land_dna"), dict) else {}
        if self.state.landscape_grammar_saved:
            self._status_var.set(f"◐ Grammar saved (QC not run) · {len(nodes)} nodes")
        elif self.state.landscape_preset_validate_ok is True:
            self._status_var.set(f"◐ Grammar loaded · {len(nodes)} topology nodes")
        else:
            self._status_var.set(f"○ Grammar pending · {len(nodes)} nodes")

    def _validate_schema(self) -> None:
        pid = self.state.selected_landscape_preset_id
        if not pid:
            set_inline_status(self._validate_lbl, self._validate_var, "Select a preset first", ok=None)
            return
        report = validate_landscape_preset(pid, compression_level=3)
        ok = report.status == "passed"
        set_inline_status(
            self._validate_lbl,
            self._validate_var,
            report.summary,
            ok=ok,
        )
        if ok:
            self._status_var.set("✓ Grammar valid")
        else:
            self._status_var.set("✗ Grammar blocked")
        self._on_log(f"grammar validate · {pid} · {report.status}")

    def mark_saved(self) -> None:
        self.state.landscape_grammar_saved = True
        self.refresh_from_state()
        self._on_log("landscape grammar · saved (scaffold)")
