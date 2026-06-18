"""MCP-APS-GRAMMAR-INSPECTOR-001 — rule_chain + tag hints from snapshot."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk
from typing import Any, Callable

from rust_engine_mcp import aps_tags


class GrammarInspectorPanel(ttk.LabelFrame):
    def __init__(
        self,
        master: tk.Misc,
        *,
        on_rule_select: Callable[[str, str], None] | None = None,
    ) -> None:
        super().__init__(master, text="Grammar inspector", padding=6)
        self._on_rule_select = on_rule_select
        meta = ttk.Frame(self)
        meta.pack(fill=tk.X, pady=(0, 4))
        self.archetype_var = tk.StringVar(value="—")
        self.district_var = tk.StringVar(value="—")
        self.massing_var = tk.StringVar(value="—")
        ttk.Label(meta, text="Building type:").grid(row=0, column=0, sticky=tk.W)
        ttk.Label(meta, textvariable=self.archetype_var, font=("Consolas", 9)).grid(
            row=0, column=1, sticky=tk.W, padx=4
        )
        ttk.Label(meta, text="District:").grid(row=0, column=2, sticky=tk.W, padx=(8, 0))
        ttk.Label(meta, textvariable=self.district_var, font=("Consolas", 9)).grid(
            row=0, column=3, sticky=tk.W, padx=4
        )
        ttk.Label(meta, text="Massing:").grid(row=1, column=0, sticky=tk.W)
        ttk.Label(meta, textvariable=self.massing_var, font=("Consolas", 9)).grid(
            row=1, column=1, sticky=tk.W, padx=4
        )
        cols = ("layer", "rule_id", "detail", "tags")
        self.tree = ttk.Treeview(self, columns=cols, show="headings", height=6)
        for col, w in zip(cols, (90, 120, 220, 160)):
            self.tree.heading(col, text=col.replace("_", " ").title())
            self.tree.column(col, width=w, stretch=col == "detail")
        self.tree.pack(fill=tk.BOTH, expand=True)
        if self._on_rule_select is not None:
            self.tree.bind("<<TreeviewSelect>>", self._on_tree_select)

    def _on_tree_select(self, _event: object | None = None) -> None:
        if self._on_rule_select is None:
            return
        selected = self.tree.selection()
        if not selected:
            return
        values = self.tree.item(selected[0], "values")
        if len(values) < 2:
            return
        layer = str(values[0] or "")
        rule_id = str(values[1] or "")
        if rule_id:
            self._on_rule_select(layer, rule_id)

    def load_snapshot(self, snapshot: dict[str, Any] | None) -> None:
        for row in self.tree.get_children():
            self.tree.delete(row)
        if not snapshot:
            self.archetype_var.set("—")
            self.district_var.set("—")
            self.massing_var.set("—")
            return
        if _is_landscape_preset(snapshot):
            self._load_landscape_preset_summary(snapshot)
            return
        self.archetype_var.set(str(snapshot.get("archetype_id") or "—"))
        self.district_var.set(str(snapshot.get("district_style") or "—"))
        chain_obj = snapshot.get("grammar_rule_chain") or {}
        self.massing_var.set(str(chain_obj.get("massing") or "—"))
        ref = snapshot.get("reference_tags") or []
        chain_steps = _rule_chain_steps(snapshot, ref)
        for step in chain_steps:
            self.tree.insert(
                "",
                tk.END,
                values=(
                    step.get("layer", ""),
                    step.get("rule_id", ""),
                    step.get("detail", ""),
                    step.get("tags", ""),
                ),
            )

    def _load_landscape_preset_summary(self, snapshot: dict[str, Any]) -> None:
        """MCP-APS-GRAMMAR-INSPECT-LAND-001 — read-only landscape preset branch."""
        preset_id = str(
            snapshot.get("landscape_preset_id")
            or snapshot.get("preset_id")
            or snapshot.get("id")
            or "landscape_preset"
        )
        self.archetype_var.set("landscape")
        self.district_var.set(preset_id)
        land_dna = snapshot.get("land_dna") if isinstance(snapshot.get("land_dna"), dict) else {}
        self.massing_var.set(str(land_dna.get("pressure_profile") or land_dna.get("biome") or "—"))
        kinds = _landscape_topology_kinds(snapshot)
        summary = ", ".join(kinds[:6]) if kinds else "no topology nodes"
        self.tree.insert("", tk.END, values=("topology", preset_id, summary, "read-only"))


def _is_landscape_preset(snapshot: dict[str, Any]) -> bool:
    schema = str(snapshot.get("schema") or "")
    return bool(
        snapshot.get("land_dna")
        or snapshot.get("topology_graph")
        or snapshot.get("landscape_preset_id")
        or schema.startswith("landscape_grammar")
    )


def _landscape_topology_kinds(snapshot: dict[str, Any]) -> list[str]:
    graph = snapshot.get("topology_graph") if isinstance(snapshot.get("topology_graph"), dict) else {}
    nodes = graph.get("nodes") or []
    kinds: list[str] = []
    for node in nodes:
        if not isinstance(node, dict):
            continue
        kind = str(node.get("kind") or node.get("type") or "").strip()
        if kind and kind not in kinds:
            kinds.append(kind)
    return kinds


def _rule_chain_steps(snapshot: dict[str, Any], ref: list[Any]) -> list[dict[str, str]]:
    steps: list[dict[str, str]] = []
    chain_obj = snapshot.get("grammar_rule_chain")
    if isinstance(chain_obj, dict) and chain_obj:
        layer_order = (
            ("archetype", "archetype"),
            ("district_style", "district_style"),
            ("massing", "massing"),
            ("roof", "roof"),
            ("facade", "facade"),
            ("detail", "detail"),
            ("age", "age"),
        )
        for layer, key in layer_order:
            val = chain_obj.get(key)
            if val:
                steps.append(
                    {
                        "layer": layer,
                        "rule_id": str(val),
                        "detail": "",
                        "tags": _tags_for_layer(layer),
                    }
                )
        if chain_obj.get("footprint_mode"):
            steps.append(
                {
                    "layer": "footprint_mode",
                    "rule_id": str(chain_obj["footprint_mode"]),
                    "detail": "massing footprint mode",
                    "tags": "",
                }
            )
        return steps
    for tag in ref:
        text = str(tag)
        if not text.startswith("chain:"):
            continue
        parts = text.split(":", 2)
        if len(parts) < 3:
            continue
        layer, rule_id = parts[1], parts[2]
        steps.append(
            {
                "layer": layer,
                "rule_id": rule_id,
                "detail": "",
                "tags": _tags_for_layer(layer),
            }
        )
    return steps


def _tags_for_layer(layer: str) -> str:
    if layer in ("facade", "district_style"):
        cats = ("location", "architectural")
    elif layer == "detail":
        cats = ("detail",)
    elif layer == "age":
        cats = ("condition",)
    else:
        return ""
    bits: list[str] = []
    for cat in cats:
        ids = aps_tags.grammar_tags_for_category(cat)[:4]
        if ids:
            bits.append(f"{cat}: {', '.join(ids)}")
    return "; ".join(bits)
