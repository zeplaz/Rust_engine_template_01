"""APS material library — thin wrapper around shared MaterialLibraryWidget."""

from __future__ import annotations

from .material_library_widget import MaterialLibraryWidget


class MaterialBrowserPanel(MaterialLibraryWidget):
    """Assembly tab material picker — assigns to selected placement."""

    def __init__(
        self,
        master,
        *,
        on_apply_material,
        on_log=None,
        layout: str = "vertical",
    ) -> None:
        super().__init__(
            master,
            mode="assign",
            on_apply_material=on_apply_material,
            on_log=on_log,
            layout=layout,
        )
