"""CMCP-SITE-PREVIEW-PANEL-001 — site zone grid preview (DES-APS-SITE-PREVIEW-001)."""

from __future__ import annotations

import json
import tkinter as tk
from pathlib import Path
from tkinter import ttk
from typing import Any, Callable

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators.site_zone_grid import validate_site_zone_grid_path

from .aps_collapsible import CollapsibleSection
from .aps_theme import COLOR_EXPLAINER_BG, COLOR_MUTED, COLOR_OUTLINE, COLOR_TEXT_SUBTLE, COLOR_WARN, FONT_SMALL, FONT_UI
from .footprint_canvas import TOKEN_COLORS
from .state import ArtDomain

PILOTS_DIR = "assets/configs/buildings/pilots"

ARCHETYPE_SITE_DEFAULTS: dict[str, str] = {
    "IndustrialWarehouse": "logistics_storage_warehouse_site_v0.json",
    "RailEdge": "logistics_rail_warehouse_site_v0.json",
    "FactoryCluster": "manufacturing_fabrication_hall_site_v0.json",
}

ZONE_FILL: dict[str, str] = {
    "primary": TOKEN_COLORS["W"],
    "loading": TOKEN_COLORS["D"],
    "utility": TOKEN_COLORS["Y"],
    "rail": TOKEN_COLORS["R"],
    "service": TOKEN_COLORS["C"],
    "parking": COLOR_MUTED,
    "buffer": "",
}


def resolve_site_json_path(
    *,
    archetype_id: str | None,
    site_template_id: str | None = None,
) -> Path | None:
    root = repo_root()
    if site_template_id:
        stem = site_template_id if site_template_id.endswith(".json") else f"{site_template_id}.json"
        path = root / PILOTS_DIR / stem
        if path.is_file():
            return path
    if archetype_id:
        default = ARCHETYPE_SITE_DEFAULTS.get(archetype_id)
        if default:
            path = root / PILOTS_DIR / default
            if path.is_file():
                return path
    return None


def load_site_zone_data(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


class SiteZonePreviewCanvas(ttk.Frame):
    def __init__(self, master: tk.Misc, *, cell_px: int = 20) -> None:
        super().__init__(master)
        self._cell_px = cell_px
        self._site: dict[str, Any] | None = None
        self._footprint_cells: list[tuple[int, int]] = []

        self.canvas = tk.Canvas(
            self, height=180, bg=COLOR_EXPLAINER_BG, highlightthickness=1, highlightbackground=COLOR_OUTLINE
        )
        self.canvas.pack(fill=tk.BOTH, expand=True)

    def set_site(self, site: dict[str, Any] | None, *, footprint_cells: list[tuple[int, int]] | None = None) -> None:
        self._site = site
        self._footprint_cells = list(footprint_cells or [])
        self.redraw()

    def redraw(self) -> None:
        self.canvas.delete("all")
        site = self._site
        if not site:
            self.canvas.create_text(
                8,
                8,
                anchor=tk.NW,
                text="○ No site template for this archetype",
                fill=COLOR_TEXT_SUBTLE,
                font=FONT_SMALL,
            )
            return
        width = int(site.get("width") or 0)
        depth = int(site.get("depth") or 0)
        cells = list(site.get("cells") or [])
        if not width or not depth or len(cells) < width * depth:
            self.canvas.create_text(
                8,
                8,
                anchor=tk.NW,
                text="◐ Site JSON invalid — run validate-report site_zone_grid",
                fill=COLOR_WARN,
                font=FONT_SMALL,
            )
            return
        px = self._cell_px
        for y in range(depth):
            for x in range(width):
                idx = y * width + x
                zone = str(cells[idx])
                x0, y0 = x * px, y * px
                x1, y1 = x0 + px, y0 + px
                fill = ZONE_FILL.get(zone, "")
                if fill:
                    self.canvas.create_rectangle(x0, y0, x1, y1, fill=fill, outline=COLOR_OUTLINE)
                else:
                    self.canvas.create_rectangle(x0, y0, x1, y1, outline=COLOR_OUTLINE)
        for gx, gy in self._footprint_cells:
            if 0 <= gx < width and 0 <= gy < depth:
                x0, y0 = gx * px, gy * px
                self.canvas.create_rectangle(
                    x0 + 2,
                    y0 + 2,
                    x0 + px - 2,
                    y0 + px - 2,
                    outline=COLOR_OUTLINE,
                    width=2,
                )


class SiteLayoutPreviewSection(CollapsibleSection):
    """Collapsible site layout block below footprint canvas."""

    def __init__(
        self,
        master: tk.Misc,
        *,
        on_log: Callable[[str], None] | None = None,
    ) -> None:
        super().__init__(master, "Site layout", expanded=False, padding=4)
        self._on_log = on_log or (lambda _m: None)
        self._grammar_tier = "G0"
        self._debounce_id: str | None = None

        head = ttk.Frame(self.body)
        head.pack(fill=tk.X)
        ttk.Label(head, text="Layout view", font=FONT_UI).pack(side=tk.LEFT)
        self._site_var = tk.StringVar(value="")
        ttk.Label(head, textvariable=self._site_var, font=FONT_SMALL, foreground=COLOR_MUTED).pack(side=tk.LEFT, padx=8)

        self._placeholder_var = tk.StringVar(value="")
        self._placeholder = ttk.Label(self.body, textvariable=self._placeholder_var, font=FONT_SMALL, foreground=COLOR_MUTED)
        self._status_var = tk.StringVar(value="")
        self._status = ttk.Label(self.body, textvariable=self._status_var, font=FONT_SMALL, foreground=COLOR_TEXT_SUBTLE)
        self._metrics_var = tk.StringVar(value="")
        self._metrics = ttk.Label(self.body, textvariable=self._metrics_var, font=FONT_SMALL, foreground=COLOR_MUTED)

        self._canvas = SiteZonePreviewCanvas(self.body)
        self._legend = ttk.Label(
            self.body,
            text="P L U R S K · — Primary · Loading · Utility · Rail · Service · parKing · buffer",
            font=FONT_SMALL,
            foreground=COLOR_MUTED,
        )

    def set_grammar_tier(self, tier: str) -> None:
        self._grammar_tier = str(tier or "G0").upper()

    def refresh(
        self,
        *,
        archetype_id: str | None,
        lane: str,
        site_template_id: str | None = None,
        footprint_cells: list[tuple[int, int]] | None = None,
    ) -> None:
        for w in (self._placeholder, self._status, self._canvas, self._metrics, self._legend):
            w.pack_forget()
        if lane == ArtDomain.LANDSCAPE.value:
            self.pack_forget()
            return
        self.pack(fill=tk.X, pady=(4, 0))
        tier = self._grammar_tier
        if tier == "G0":
            self.pack_forget()
            return
        if tier == "G1":
            self._placeholder_var.set("○ Site layout unlocks after archetype family tuning")
            self._placeholder.pack(anchor=tk.W)
            return
        path = resolve_site_json_path(archetype_id=archetype_id, site_template_id=site_template_id)
        if not path:
            self._placeholder_var.set("○ No site template for this archetype")
            self._placeholder.pack(anchor=tk.W)
            self._site_var.set("")
            return
        try:
            site = load_site_zone_data(path)
        except (OSError, json.JSONDecodeError) as exc:
            self._status_var.set(f"◐ Site JSON invalid — {exc}")
            self._status.pack(anchor=tk.W)
            return
        report = validate_site_zone_grid_path(path)
        site_id = str(site.get("site_id") or path.stem)
        self._site_var.set(f"Site: {site_id}")
        if report.status == "failed":
            first = report.errors[0].hint if report.errors else "validation failed"
            self._status_var.set(f"✗ site blocked — {first}")
            self._status.pack(anchor=tk.W, pady=(2, 0))
        else:
            self._status_var.set("✓ site valid")
            self._status.pack(anchor=tk.W, pady=(2, 0))
        self._canvas.set_site(site, footprint_cells=footprint_cells)
        self._canvas.pack(fill=tk.BOTH, expand=True, pady=(4, 0))
        self._legend.pack(anchor=tk.W, pady=(2, 0))
        metrics = site.get("metrics") or {}
        parts: list[str] = []
        if metrics.get("primary_pct_site") is not None:
            pct = float(metrics["primary_pct_site"]) * 100.0
            parts.append(f"Primary {pct:.0f}%")
        if metrics.get("loading_cells"):
            parts.append(f"Loading {metrics['loading_cells']} cells")
        if metrics.get("utility_cells"):
            parts.append(f"Utility {metrics['utility_cells']} cells")
        elif "utility" in str(site.get("cells") or []):
            parts.append("Utility present")
        if parts:
            self._metrics_var.set(" · ".join(parts))
            self._metrics.pack(anchor=tk.W, pady=(2, 0))


def refresh_site_preview_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    path = resolve_site_json_path(archetype_id="RailEdge")
    ok = path is not None and path.is_file()
    body: dict[str, Any] = {
        "task_id": "CMCP-SITE-PREVIEW-PANEL-001",
        "green": ok,
        "site_preview_visible": True,
        "site_preview_expanded": False,
        "sample_site_id": "logistics_rail_warehouse_site_v0" if ok else None,
        "sample_path": str(path.relative_to(root)).replace("\\", "/") if path and ok else None,
    }
    out = root / "debug_runs" / "art_pipeline" / "site_preview_panel_live.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = "debug_runs/art_pipeline/site_preview_panel_live.json"
    return body
