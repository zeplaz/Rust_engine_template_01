"""Art Pipeline Suite — Catalog | Assembly | Variants | Atlas."""

from __future__ import annotations

import tkinter as tk
from tkinter import messagebox, ttk

from .assembly_panel import AssemblyPanel
from .atlas_panel import AtlasPanel
from .catalog import CatalogPanel
from .scrollable import ScrollableFrame
from .state import SuiteState
from .materials_panel import MaterialsPanel
from .variants_panel import VariantsPanel


class ArtPipelineSuiteApp(tk.Tk):
    def __init__(self) -> None:
        super().__init__()
        self.title("Rust Engine — Art Pipeline Suite")
        self.geometry("1180x760")
        self.minsize(960, 600)
        self.state = SuiteState()
        self.status_var = tk.StringVar(value="Ready")
        self._build_flow_bar()
        self._build_tabs()
        self._build_status_log()

    def _add_scrollable_tab(self, panel_cls, text: str, **panel_kw):
        """Notebook page with vertical scroll (horizontal when content overflows width)."""
        tab_root = ttk.Frame(self.notebook)
        scroll = ScrollableFrame(tab_root, enable_horizontal=True)
        scroll.pack(fill=tk.BOTH, expand=True)
        panel = panel_cls(scroll.interior, **panel_kw)
        panel.pack(fill=tk.BOTH, expand=True)
        self.notebook.add(tab_root, text=text)
        panel._aps_tab_root = tab_root
        return panel

    def _build_flow_bar(self) -> None:
        bar = ttk.Frame(self, padding=8)
        bar.pack(fill=tk.X)
        ttk.Label(bar, text="Flow:").pack(side=tk.LEFT)
        ttk.Button(bar, text="Send to Assembly", command=self.on_send_to_assembly).pack(
            side=tk.LEFT, padx=4
        )
        ttk.Button(bar, text="Bake variants", command=self.on_bake_variants).pack(side=tk.LEFT, padx=4)
        ttk.Button(bar, text="Pack atlas", command=self.on_pack_atlas).pack(side=tk.LEFT, padx=4)
        ttk.Separator(bar, orient=tk.VERTICAL).pack(side=tk.LEFT, fill=tk.Y, padx=8)
        ttk.Label(
            bar,
            text="All actions call rust_engine_mcp CLI/MCP — agents use the same APIs.",
            foreground="#444",
        ).pack(side=tk.LEFT)

    def _build_tabs(self) -> None:
        self.notebook = ttk.Notebook(self)
        self.notebook.pack(fill=tk.BOTH, expand=True, padx=8, pady=4)

        self.catalog = self._add_scrollable_tab(
            CatalogPanel, "Catalog", state=self.state, on_select=self._on_catalog_select
        )
        self.assembly = self._add_scrollable_tab(
            AssemblyPanel, "Assembly", state=self.state, on_log=self._log
        )
        self.materials = self._add_scrollable_tab(
            MaterialsPanel,
            "Materials",
            state=self.state,
            on_log=self._log,
            on_open_in_assembly=self._open_material_in_assembly,
        )
        self.variants = self._add_scrollable_tab(
            VariantsPanel, "Variants", state=self.state, on_log=self._log
        )
        self.atlas = self._add_scrollable_tab(AtlasPanel, "Atlas", state=self.state, on_log=self._log)

    def _build_status_log(self) -> None:
        frame = ttk.Frame(self, padding=(8, 0, 8, 8))
        frame.pack(fill=tk.X)
        ttk.Label(frame, textvariable=self.status_var, foreground="#333").pack(anchor=tk.W)

    def _log(self, line: str) -> None:
        self.state.append_log(line)
        self.status_var.set(line[:240])

    def _on_catalog_select(self, _rec) -> None:
        self._log(f"catalog select: {self.state.selected_module_id}")

    def _open_material_in_assembly(self, profile_id: str) -> None:
        self.notebook.select(self.assembly._aps_tab_root)
        self.assembly.material_browser.highlight_profile(profile_id)
        self._log(f"materials → assembly · {profile_id}")

    def on_send_to_assembly(self) -> None:
        self.assembly.sync_from_state()
        self.notebook.select(self.assembly._aps_tab_root)
        self._log(f"assembly ← style {self.state.style_pack_id}")

    def on_bake_variants(self) -> None:
        if not self.state.variant_set_data and not self.state.assembly_id:
            messagebox.showinfo(
                "Bake variants",
                "Create assembly snapshot and variant set first (Assembly → Variants).",
            )
            return
        self.notebook.select(self.variants._aps_tab_root)
        if not self.state.variant_set_data:
            self.variants.on_new_from_assembly()
        self.atlas.on_batch_from_variant_set()
        self.notebook.select(self.atlas._aps_tab_root)
        self._log("bake variants → tile_batch prepared on Atlas tab")

    def on_pack_atlas(self) -> None:
        self.atlas.sync_folder_from_state()
        self.notebook.select(self.atlas._aps_tab_root)
        folder = self.state.atlas_folder
        if folder:
            self.atlas.on_pack()
        else:
            messagebox.showinfo("Pack atlas", "Run tile batch or set PNG folder on Atlas tab.")
        self._log("pack atlas")


def run_app() -> None:
    app = ArtPipelineSuiteApp()
    app.mainloop()
