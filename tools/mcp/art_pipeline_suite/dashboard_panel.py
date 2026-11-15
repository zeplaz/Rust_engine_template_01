"""APS Studio Ops / Dashboard — open HTML reports from JSON (studio_dash kit)."""

from __future__ import annotations

import json
import sys
import webbrowser
from pathlib import Path
from tkinter import ttk
from typing import Callable

from .aps_theme import FONT_UI_BOLD, GAP_MD


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[3]


def _studio_dash_on_path() -> Path:
    root = _repo_root()
    dash = root / "tools" / "studio_dash"
    if str(dash) not in sys.path:
        sys.path.insert(0, str(dash.parent))
    return dash


class DashboardPanel(ttk.Frame):
    def __init__(
        self,
        master,
        *,
        state=None,
        on_log: Callable[[str], None] | None = None,
        **_kwargs,
    ) -> None:
        super().__init__(master)
        self._log = on_log or (lambda _m: None)
        ttk.Label(self, text="Studio dashboards", font=FONT_UI_BOLD).pack(
            anchor="w", padx=GAP_MD, pady=(GAP_MD, 4)
        )
        ttk.Label(
            self,
            text="Static HTML from JSON (not Grafana). Kit coverage, grammar QC, golden seed, jobs, sim review.",
            wraplength=520,
        ).pack(anchor="w", padx=GAP_MD)
        btns = ttk.Frame(self, padding=GAP_MD)
        btns.pack(fill="x")
        ttk.Button(btns, text="Refresh report list", command=self._refresh).pack(side="left", padx=4)
        ttk.Button(btns, text="Render StudioTasks HTML", command=self._render_tasks).pack(
            side="left", padx=4
        )
        ttk.Button(btns, text="Open selected", command=self._open_selected).pack(side="left", padx=4)
        ttk.Button(btns, text="Blender handoff (selected task)", command=self._handoff).pack(
            side="left", padx=4
        )
        self._list = ttk.Treeview(
            self, columns=("path",), show="headings", height=14, selectmode="browse"
        )
        self._list.heading("path", text="Report / JSON")
        self._list.column("path", width=640)
        self._list.pack(fill="both", expand=True, padx=GAP_MD, pady=GAP_MD)
        self._refresh()

    def _known_paths(self) -> list[Path]:
        root = _repo_root()
        candidates = [
            root / "debug_runs" / "studio_dash",
            root / "debug_runs" / "art_pipeline",
            root / "debug_runs" / "sim_review",
            root / "assets" / "staging",
        ]
        out: list[Path] = []
        for base in candidates:
            if not base.is_dir():
                continue
            for pat in ("*.html", "*coverage*.json", "*pcg*.json", "*golden*.json", "matrix.json"):
                out.extend(sorted(base.glob(pat))[:20])
                out.extend(sorted(base.glob(f"**/{pat}"))[:20])
        # de-dupe preserve order
        seen: set[str] = set()
        uniq: list[Path] = []
        for p in out:
            s = str(p.resolve())
            if s not in seen:
                seen.add(s)
                uniq.append(p)
        return uniq[:80]

    def _refresh(self) -> None:
        self._list.delete(*self._list.get_children())
        for p in self._known_paths():
            self._list.insert("", "end", iid=str(p), values=(str(p),))
        self._log(f"Dashboard: listed {len(self._list.get_children())} reports")

    def _selected_path(self) -> Path | None:
        sel = self._list.selection()
        if not sel:
            return None
        return Path(sel[0])

    def _open_selected(self) -> None:
        path = self._selected_path()
        if path is None:
            self._log("Dashboard: nothing selected")
            return
        if path.suffix.lower() == ".json":
            html = self._render_json(path)
            webbrowser.open(html.resolve().as_uri())
            self._log(f"Dashboard: opened {html}")
            return
        webbrowser.open(path.resolve().as_uri())
        self._log(f"Dashboard: opened {path}")

    def _render_json(self, path: Path) -> Path:
        _studio_dash_on_path()
        from studio_dash.render_report import write_report

        data = json.loads(path.read_text(encoding="utf-8"))
        out = _repo_root() / "debug_runs" / "studio_dash" / f"{path.stem}.html"
        return write_report(data, out, title=path.stem)

    def _render_tasks(self) -> None:
        _studio_dash_on_path()
        from studio_dash.render_report import write_report
        from rust_engine_mcp import studio_task

        data = {
            "title": "StudioTasks",
            "status": "ok",
            "tasks": studio_task.list_tasks(limit=40),
        }
        out = _repo_root() / "debug_runs" / "studio_dash" / "studio_tasks.html"
        write_report(data, out)
        webbrowser.open(out.resolve().as_uri())
        self._refresh()
        self._log(f"Dashboard: wrote {out}")

    def _handoff(self) -> None:
        from rust_engine_mcp import studio_task

        tasks = studio_task.list_tasks(limit=1)
        if not tasks:
            self._log("Dashboard: no StudioTasks to hand off — enqueue first")
            return
        tid = tasks[0]["task_id"]
        marker = studio_task.blender_handoff_marker(tid)
        self._log(f"Dashboard: Blender handoff → {marker}")
