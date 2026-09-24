"""VSS-T4-005 — Effect library widget (browse + honesty + preview ladder + assign).

Tk-only. No Bevy / fire_vfx imports. Preview uses thumbs / capture badges — never live particles.
"""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk
from typing import Any, Callable

from rust_engine_mcp.effect_registry_browse import (
    EffectBrowseEntry,
    REFERENCE_BATCH_ID,
    honesty_label,
    list_batch_ids,
    list_effect_entries,
)
from rust_engine_mcp.scenario_trigger_write import AssignBlockedError, assign_effect_to_marker, can_assign

from .aps_paned import add_pane, horizontal_paned
from .aps_theme import (
    COLOR_MUTED,
    COLOR_TEXT_HINT,
    COLOR_TEXT_SUBTLE,
    FONT_CAPTION,
    FONT_SMALL,
    FONT_UI_BOLD,
)
from .aps_tooltips import bind_aps_tooltip, bind_many

OnAssignFn = Callable[[str, str], None]
OnLogFn = Callable[[str], None]


class EffectLibraryWidget(ttk.Frame):
    """Browse registry effects, surface honest_gate, assign to scenario markers."""

    RECENT_CAP = 12

    def __init__(
        self,
        master: tk.Misc,
        *,
        mode: str = "browse",
        on_assign_effect: OnAssignFn | None = None,
        on_log: OnLogFn | None = None,
        layout: str = "browse",
        **_kwargs: Any,
    ) -> None:
        super().__init__(master, padding=4)
        self._mode = mode
        self._on_assign = on_assign_effect or (lambda _eid, _mid: None)
        self._on_log = on_log or (lambda _line: None)
        self._layout = layout
        self._entries: list[EffectBrowseEntry] = []
        self._filtered: list[EffectBrowseEntry] = []
        self._recent: list[str] = []
        self._selected_id: str | None = None
        self._search_after: str | None = None
        self._preview_after: str | None = None
        self._build()

    def _build(self) -> None:
        filter_row = ttk.Frame(self)
        filter_row.pack(fill=tk.X, pady=2)
        ttk.Label(filter_row, text="Search").pack(side=tk.LEFT)
        self._search_var = tk.StringVar(value="")
        self._search_entry = ttk.Entry(filter_row, textvariable=self._search_var, width=22)
        self._search_entry.pack(side=tk.LEFT, padx=4)
        self._search_var.trace_add("write", lambda *_: self._schedule_filter())
        self._search_entry.bind("<Escape>", self._clear_search)

        ttk.Label(filter_row, text="Batch").pack(side=tk.LEFT, padx=(8, 0))
        self._batch_var = tk.StringVar(value="All")
        self._batch_combo = ttk.Combobox(
            filter_row,
            textvariable=self._batch_var,
            values=["All", "Staging", "Recent"],
            width=28,
            state="readonly",
        )
        self._batch_combo.pack(side=tk.LEFT, padx=4)
        self._batch_var.trace_add("write", lambda *_: self._apply_filters())

        body = horizontal_paned(self)
        body.pack(fill=tk.BOTH, expand=True, pady=4)

        list_wrap = ttk.Frame(body, padding=2)
        add_pane(body, list_wrap, weight=3, minsize=260)
        cols = ("name", "lane", "honesty")
        self._tree = ttk.Treeview(list_wrap, columns=cols, show="headings", height=14, selectmode="browse")
        self._tree.heading("name", text="Effect")
        self._tree.heading("lane", text="Lane")
        self._tree.heading("honesty", text="Honesty")
        self._tree.column("name", width=200, stretch=True)
        self._tree.column("lane", width=90, stretch=False)
        self._tree.column("honesty", width=180, stretch=True)
        yscroll = ttk.Scrollbar(list_wrap, orient=tk.VERTICAL, command=self._tree.yview)
        self._tree.configure(yscrollcommand=yscroll.set)
        self._tree.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        yscroll.pack(side=tk.RIGHT, fill=tk.Y)
        self._tree.bind("<<TreeviewSelect>>", self._on_select)
        self._tree.bind("<Up>", lambda e: self._nudge_selection(-1))
        self._tree.bind("<Down>", lambda e: self._nudge_selection(1))
        self._tree.bind("<Return>", lambda e: self._on_select())

        right = ttk.Frame(body, padding=2)
        add_pane(body, right, weight=2, minsize=240)
        self._build_preview(right)
        self._build_assign(right)

        self._status_var = tk.StringVar(value="")
        ttk.Label(self, textvariable=self._status_var, foreground=COLOR_TEXT_SUBTLE, font=FONT_SMALL).pack(
            anchor=tk.W, pady=2
        )
        self.reload_catalog()

    def _build_preview(self, parent: tk.Misc) -> None:
        frame = ttk.LabelFrame(parent, text="Preview ladder", padding=6)
        frame.pack(fill=tk.BOTH, expand=True, pady=(0, 6))
        bind_aps_tooltip(frame, "eff_preview_ladder")

        self._honesty_var = tk.StringVar(value="○ no preview witness")
        self._honesty_lbl = ttk.Label(frame, textvariable=self._honesty_var, font=FONT_UI_BOLD)
        self._honesty_lbl.pack(anchor=tk.W)
        bind_aps_tooltip(self._honesty_lbl, "eff_honesty_pending")

        self._p0_var = tk.StringVar(value="P0 — select an effect")
        self._p1_var = tk.StringVar(value="P1 —")
        self._p2_var = tk.StringVar(value="P2 —")
        self._p3_var = tk.StringVar(value="P3 —")
        self._p4_var = tk.StringVar(value="P4 —")
        for var in (self._p0_var, self._p1_var, self._p2_var, self._p3_var, self._p4_var):
            ttk.Label(frame, textvariable=var, wraplength=280, justify=tk.LEFT, font=FONT_SMALL).pack(
                anchor=tk.W, pady=1
            )

        self._ladder_hint = tk.StringVar(
            value="Preview unlocks when honest_gate is honest — run effect preview / promote capture."
        )
        ttk.Label(
            frame,
            textvariable=self._ladder_hint,
            wraplength=280,
            foreground=COLOR_TEXT_HINT,
            font=FONT_CAPTION,
        ).pack(anchor=tk.W, pady=(6, 0))

    def _build_assign(self, parent: tk.Misc) -> None:
        frame = ttk.LabelFrame(parent, text="Assign to marker", padding=6)
        frame.pack(fill=tk.X)
        row = ttk.Frame(frame)
        row.pack(fill=tk.X)
        ttk.Label(row, text="Marker id").pack(side=tk.LEFT)
        self._marker_var = tk.StringVar(value="")
        self._marker_entry = ttk.Entry(row, textvariable=self._marker_var, width=24)
        self._marker_entry.pack(side=tk.LEFT, padx=4, fill=tk.X, expand=True)

        self._assign_btn = ttk.Button(frame, text="Assign to marker…", command=self._assign_selected)
        self._assign_btn.pack(anchor=tk.W, pady=(6, 0))
        bind_aps_tooltip(self._assign_btn, "eff_assign_marker")
        self._assign_hint = tk.StringVar(value="")
        ttk.Label(
            frame,
            textvariable=self._assign_hint,
            wraplength=280,
            foreground=COLOR_MUTED,
            font=FONT_CAPTION,
        ).pack(anchor=tk.W, pady=2)

        if self._mode == "assign":
            self._assign_btn.state(["!disabled"])

    def bind_tooltips(self) -> None:
        bind_many(
            [
                (self._search_entry, "eff_preview_ladder"),
                (self._assign_btn, "eff_assign_marker"),
            ]
        )

    def reload_catalog(self) -> None:
        self._entries = list_effect_entries()
        batches = ["All", "Staging", "Recent"] + list_batch_ids(self._entries)
        self._batch_combo.configure(values=batches)
        self._apply_filters()
        self._status_var.set(f"{len(self._entries)} effects · registry + staging")

    def _schedule_filter(self) -> None:
        if self._search_after:
            self.after_cancel(self._search_after)
        self._search_after = self.after(200, self._apply_filters)

    def _clear_search(self, _event=None) -> None:
        self._search_var.set("")
        return "break"

    def _apply_filters(self) -> None:
        q = self._search_var.get().strip().lower()
        batch = self._batch_var.get()
        rows: list[EffectBrowseEntry] = []
        for e in self._entries:
            if batch == "Staging" and e.source != "staging":
                continue
            if batch == "Recent" and e.effect_id not in self._recent:
                continue
            if batch not in ("All", "Staging", "Recent") and e.batch_id != batch:
                continue
            if q:
                hay = f"{e.effect_id} {e.display_name} {e.batch_id} {e.lane} {e.lane_caption}".lower()
                if q not in hay:
                    continue
            rows.append(e)
        self._filtered = rows
        self._rebuild_tree()

    def _rebuild_tree(self) -> None:
        self._tree.delete(*self._tree.get_children())
        for e in self._filtered:
            self._tree.insert(
                "",
                tk.END,
                iid=e.effect_id,
                values=(e.display_name, e.lane_caption, e.honesty_label),
            )
        if self._selected_id and self._tree.exists(self._selected_id):
            self._tree.selection_set(self._selected_id)
            self._tree.focus(self._selected_id)

    def _nudge_selection(self, delta: int) -> str:
        kids = list(self._tree.get_children())
        if not kids:
            return "break"
        cur = self._tree.focus() or (self._tree.selection()[0] if self._tree.selection() else "")
        idx = kids.index(cur) if cur in kids else 0
        idx = max(0, min(len(kids) - 1, idx + delta))
        self._tree.selection_set(kids[idx])
        self._tree.focus(kids[idx])
        self._on_select()
        return "break"

    def _selected_entry(self) -> EffectBrowseEntry | None:
        if not self._selected_id:
            return None
        for e in self._entries:
            if e.effect_id == self._selected_id:
                return e
        return None

    def _on_select(self, _event=None) -> None:
        sel = self._tree.selection()
        if not sel:
            return
        eid = sel[0]
        self._selected_id = eid
        if eid in self._recent:
            self._recent.remove(eid)
        self._recent.insert(0, eid)
        self._recent = self._recent[: self.RECENT_CAP]
        if not self._marker_var.get().strip():
            self._marker_var.set(eid)
        elif self._marker_var.get().strip() in {e.effect_id for e in self._entries}:
            self._marker_var.set(eid)
        if self._preview_after:
            self.after_cancel(self._preview_after)
        self._ladder_hint.set("⟳ Loading preview…")
        self._preview_after = self.after(300, self._refresh_preview)

    def _refresh_preview(self) -> None:
        entry = self._selected_entry()
        if not entry:
            self._honesty_var.set(honesty_label(None))
            self._p0_var.set("P0 — select an effect")
            self._assign_btn.state(["disabled"])
            self._assign_hint.set("")
            return

        gate = entry.honest_gate
        self._honesty_var.set(entry.honesty_label)
        tip_key = {
            "honest": "eff_honesty_honest",
            "pending": "eff_honesty_pending",
            "dishonest_gate": "eff_honesty_dishonest",
        }.get(gate, "eff_honesty_pending")
        bind_aps_tooltip(self._honesty_lbl, tip_key)

        # P0 — spawn hook + tile token
        self._p0_var.set(f"P0 — spawn_hook={entry.spawn_hook or '—'} · tile token")

        # P1 — thumb or lane placeholder
        if entry.staging_thumb:
            self._p1_var.set(f"P1 — thumb {entry.staging_thumb}")
        else:
            self._p1_var.set(f"P1 — placeholder ({entry.lane_caption} · seed from spec)")

        # P2 — emit / lod strip
        emit = entry.emit
        dur = emit.get("duration_secs", "—")
        rate = emit.get("rate", "—")
        lod_keys = ", ".join(k for k, v in entry.lod_tiers.items() if isinstance(v, dict) and v.get("enabled"))
        self._p2_var.set(f"P2 — duration={dur}s · rate={rate} · lod[{lod_keys or '—'}]")

        # P3 — capture frames
        if gate == "dishonest_gate":
            self._p3_var.set("P3 — ⊘ dishonest — frames locked")
        elif entry.frames_captured >= 1:
            self._p3_var.set(f"P3 — {entry.frames_captured} capture frame(s)")
        else:
            self._p3_var.set("P3 — no frames captured yet")

        # P4 — ship chip
        if gate == "honest" and entry.development_tier == "production":
            self._p4_var.set("P4 — ✓ production ship chip")
        elif gate == "honest":
            self._p4_var.set(f"P4 — honest but tier={entry.development_tier} (ship chip locked)")
        else:
            self._p4_var.set("P4 — locked until honest_gate=honest + production tier")

        if gate == "honest":
            self._ladder_hint.set("Preview honesty passed — thumbs/captures only (not live particles).")
        else:
            self._ladder_hint.set(
                "Preview unlocks when honest_gate is honest — run effect preview / promote capture."
            )

        if can_assign(entry):
            self._assign_btn.state(["!disabled"])
            self._assign_hint.set("")
        else:
            self._assign_btn.state(["disabled"])
            self._assign_hint.set(
                f"⊘ Assign blocked — preview honesty is {gate}. Capture frames before binding to a scenario."
            )

    def _assign_selected(self) -> None:
        entry = self._selected_entry()
        if not entry:
            self._on_log("⊘ Assign — no effect selected")
            return
        marker = self._marker_var.get().strip() or entry.effect_id
        try:
            result = assign_effect_to_marker(entry.effect_id, marker, entry=entry)
        except AssignBlockedError as exc:
            self._status_var.set(str(exc))
            self._on_log(str(exc))
            self._refresh_preview()
            return
        except (OSError, ValueError, FileNotFoundError) as exc:
            self._status_var.set(f"Assign failed: {exc}")
            self._on_log(f"Assign failed: {exc}")
            return
        msg = (
            f"✓ Assigned {result['display_name']} → trigger {result['marker_id']}. "
            "Scenario will TriggerEffect this id — game draw uses existing fire_vfx spine."
        )
        self._status_var.set(msg)
        self._on_log(f"✓ effect assign effect_id={result['effect_id']} marker_id={result['marker_id']} → {result['path']}")
        self._on_assign(result["effect_id"], result["marker_id"])

    def selected_effect_id(self) -> str | None:
        return self._selected_id

    def list_reference_ids(self) -> list[str]:
        return [e.effect_id for e in self._entries if e.batch_id == REFERENCE_BATCH_ID]
