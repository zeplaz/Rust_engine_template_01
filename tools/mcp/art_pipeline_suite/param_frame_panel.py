"""S1 — ParamFrame DCC: grammar / modifier / connection browsers (no live bpy)."""

from __future__ import annotations

import json
import tkinter as tk
from pathlib import Path
from tkinter import ttk
from typing import Any, Callable

from rust_engine_mcp import param_frame_studio
from rust_engine_mcp.paths import jobs_root, repo_root

from .aps_inline_feedback import set_inline_status
from .aps_theme import FONT_SMALL, FONT_UI_BOLD, GAP_MD


def _repo_root() -> Path:
    return repo_root()


class ParamFramePanel(ttk.Frame):
    """Studio tools that edit ParamFrame JSON on disk — catalog browse only."""

    def __init__(
        self,
        master,
        *,
        state=None,
        on_log: Callable[[str], None] | None = None,
        **_kwargs,
    ) -> None:
        super().__init__(master)
        self.state = state
        self._log = on_log or (lambda _m: None)
        self._frame: dict[str, Any] = param_frame_studio.load_param_frame()
        self._grammar_rows: list[dict[str, Any]] = []
        self._modifier_rows: list[dict[str, Any]] = []
        self._connection_rows: list[dict[str, Any]] = []

        ttk.Label(self, text="Parametric frame (DCC)", font=FONT_UI_BOLD).pack(
            anchor="w", padx=GAP_MD, pady=(GAP_MD, 2)
        )
        ttk.Label(
            self,
            text="Browse grammars, modifiers, and connection graphs without bpy. "
            "PCG-08 generate-and-test still gates enqueue/escalate.",
            wraplength=720,
            font=FONT_SMALL,
        ).pack(anchor="w", padx=GAP_MD)

        self._status_var = tk.StringVar(value="Frame loaded from S0 example.")
        self._status_lbl = ttk.Label(self, textvariable=self._status_var, wraplength=700, font=FONT_SMALL)
        self._status_lbl.pack(anchor="w", padx=GAP_MD, pady=4)

        cols = ttk.Frame(self, padding=GAP_MD)
        cols.pack(fill=tk.BOTH, expand=True)

        self._grammar_tree = self._make_tree(cols, "Grammars (PCG-06)", ("label", "step", "grammar_id"), width=220)
        self._modifier_tree = self._make_tree(cols, "Modifiers", ("label", "modifier_id"), width=180)
        self._connection_tree = self._make_tree(
            cols, "Connection graphs", ("connection_graph_id", "source"), width=200
        )

        btns = ttk.Frame(self, padding=GAP_MD)
        btns.pack(fill=tk.X)
        ttk.Button(btns, text="Add grammar step", command=self._on_add_grammar).pack(side=tk.LEFT, padx=2)
        ttk.Button(btns, text="Add modifier", command=self._on_add_modifier).pack(side=tk.LEFT, padx=2)
        ttk.Button(btns, text="Toggle connection", command=self._on_toggle_connection).pack(side=tk.LEFT, padx=2)
        ttk.Button(btns, text="Save frame JSON", command=self._on_save).pack(side=tk.LEFT, padx=8)
        ttk.Button(btns, text="Enqueue StudioTask", command=self._on_enqueue).pack(side=tk.LEFT, padx=2)

        self._refresh_catalogs()
        self._refresh_frame_preview()

    def _make_tree(self, parent: ttk.Frame, title: str, columns: tuple[str, ...], *, width: int) -> ttk.Treeview:
        wrap = ttk.LabelFrame(parent, text=title, padding=4)
        wrap.pack(side=tk.LEFT, fill=tk.BOTH, expand=True, padx=4)
        tree = ttk.Treeview(wrap, columns=columns, show="headings", height=12, selectmode="browse")
        for col in columns:
            tree.heading(col, text=col.replace("_", " "))
            tree.column(col, width=width // len(columns), stretch=True)
        tree.pack(fill=tk.BOTH, expand=True)
        return tree

    def refresh_from_state(self) -> None:
        self._refresh_catalogs()

    def get_frame(self) -> dict[str, Any]:
        return dict(self._frame)

    def _refresh_catalogs(self) -> None:
        self._grammar_rows = param_frame_studio.browse_grammar_catalog()
        self._modifier_rows = param_frame_studio.browse_modifier_catalog()
        self._connection_rows = param_frame_studio.browse_connection_graph_ids()
        self._fill_tree(self._grammar_tree, self._grammar_rows, ("label", "step", "grammar_id"))
        self._fill_tree(self._modifier_tree, self._modifier_rows, ("label", "modifier_id"))
        conn_display = [
            {
                "connection_graph_id": r["connection_graph_id"],
                "source": r.get("source_file") or "",
            }
            for r in self._connection_rows
        ]
        self._fill_tree(self._connection_tree, conn_display, ("connection_graph_id", "source"))

    @staticmethod
    def _fill_tree(tree: ttk.Treeview, rows: list[dict[str, Any]], keys: tuple[str, ...]) -> None:
        tree.delete(*tree.get_children())
        for row in rows:
            tree.insert("", tk.END, values=tuple(str(row.get(k, "")) for k in keys))

    def _selected_index(self, tree: ttk.Treeview) -> int | None:
        sel = tree.selection()
        if not sel:
            return None
        return tree.index(sel[0])

    def _refresh_frame_preview(self) -> None:
        chain = len(self._frame.get("grammar_chain") or [])
        mods = len(self._frame.get("modifiers") or [])
        graphs = len(self._frame.get("connection_graph_ids") or [])
        gate = param_frame_studio.pcg08_promote_gate_snapshot()
        gate_txt = "PCG-08 green" if gate.get("ok") else str(gate.get("blocked_by"))
        text = (
            f"frame_id={self._frame.get('frame_id')} · chain={chain} · modifiers={mods} · "
            f"graphs={graphs} · {gate_txt}"
        )
        set_inline_status(self._status_lbl, self._status_var, text, ok=gate.get("ok"))

    def _on_add_grammar(self) -> None:
        idx = self._selected_index(self._grammar_tree)
        if idx is None or idx >= len(self._grammar_rows):
            self._log("param-frame: select a grammar row")
            return
        entry = self._grammar_rows[idx]
        self._frame = param_frame_studio.upsert_grammar_chain_step(self._frame, entry)
        self._log(f"param-frame grammar step {entry.get('step')} ← {entry.get('grammar_id')}")
        self._refresh_frame_preview()

    def _on_add_modifier(self) -> None:
        idx = self._selected_index(self._modifier_tree)
        if idx is None or idx >= len(self._modifier_rows):
            self._log("param-frame: select a modifier")
            return
        mid = str(self._modifier_rows[idx]["modifier_id"])
        self._frame = param_frame_studio.upsert_modifier(self._frame, mid, enabled=True)
        self._log(f"param-frame modifier ← {mid}")
        self._refresh_frame_preview()

    def _on_toggle_connection(self) -> None:
        idx = self._selected_index(self._connection_tree)
        if idx is None or idx >= len(self._connection_rows):
            self._log("param-frame: select a connection graph")
            return
        gid = str(self._connection_rows[idx]["connection_graph_id"])
        self._frame = param_frame_studio.toggle_connection_graph_id(self._frame, gid)
        self._log(f"param-frame toggle connection {gid}")
        self._refresh_frame_preview()

    def _save_path(self) -> Path:
        out_dir = jobs_root() / "param_frames"
        out_dir.mkdir(parents=True, exist_ok=True)
        return out_dir / f"{self._frame.get('frame_id', 'param_frame')}.json"

    def _on_save(self) -> None:
        path = self._save_path()
        param_frame_studio.write_param_frame(self._frame, path)
        self._log(f"param-frame saved {path.relative_to(_repo_root())}")
        self._refresh_frame_preview()

    def _on_enqueue(self) -> None:
        path = self._save_path()
        param_frame_studio.write_param_frame(self._frame, path)
        try:
            task = param_frame_studio.enqueue_param_frame_task(path)
        except ValueError as exc:
            self._log(f"param-frame enqueue blocked: {exc}")
            self._refresh_frame_preview()
            return
        self._log(f"param-frame StudioTask queued {task.get('task_id')}")
        self._refresh_frame_preview()