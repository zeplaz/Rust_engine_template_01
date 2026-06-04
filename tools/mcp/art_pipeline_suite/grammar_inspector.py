"""MCP-APS-GRAMMAR-INSPECTOR-001 — rule_chain + tag hints from snapshot."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk
from typing import Any

from rust_engine_mcp import aps_tags


class GrammarInspectorPanel(ttk.LabelFrame):
    def __init__(self, master: tk.Misc) -> None:
        super().__init__(master, text="Grammar inspector", padding=6)
        meta = ttk.Frame(self)
        meta.pack(fill=tk.X, pady=(0, 4))
        self.archetype_var = tk.StringVar(value="—")
        self.district_var = tk.StringVar(value="—")
        self.massing_var = tk.StringVar(value="—")
        ttk.Label(meta, text="Archetype:").grid(row=0, column=0, sticky=tk.W)
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

    def load_snapshot(self, snapshot: dict[str, Any] | None) -> None:
        for row in self.tree.get_children():
            self.tree.delete(row)
        if not snapshot:
            self.archetype_var.set("—")
            self.district_var.set("—")
            self.massing_var.set("—")
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
