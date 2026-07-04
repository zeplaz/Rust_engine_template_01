"""APSR-P3 — single material library mount entry (assign + studio presets).

Mount points:
  - Assembly tab: ``mount_material_library(..., mount="assign")``
  - Materials tab: ``mount_material_library(..., mount="studio")``

Both route through ``MaterialBrowserPanel`` — the only wrapper over ``MaterialLibraryWidget``.
"""

from __future__ import annotations

from typing import Any, TypedDict

from .material_library_widget import MaterialLibraryWidget

MOUNT_ASSIGN = "assign"
MOUNT_STUDIO = "studio"


class MaterialMountConfig(TypedDict):
    mode: str
    layout: str


MATERIAL_MOUNT_CONFIG: dict[str, MaterialMountConfig] = {
    MOUNT_ASSIGN: {"mode": "assign", "layout": "vertical"},
    MOUNT_STUDIO: {"mode": "studio", "layout": "studio_tree"},
}


class MaterialBrowserPanel(MaterialLibraryWidget):
    """Canonical APS material library wrapper — use ``mount_material_library`` at call sites."""

    def __init__(
        self,
        master,
        *,
        mount: str = MOUNT_ASSIGN,
        on_apply_material=None,
        on_log=None,
        layout: str | None = None,
        mode: str | None = None,
        **kwargs: Any,
    ) -> None:
        preset = MATERIAL_MOUNT_CONFIG.get(mount, MATERIAL_MOUNT_CONFIG[MOUNT_ASSIGN])
        super().__init__(
            master,
            mode=mode or preset["mode"],
            layout=layout or preset["layout"],
            on_apply_material=on_apply_material,
            on_log=on_log,
            **kwargs,
        )


def mount_material_library(master, *, mount: str = MOUNT_ASSIGN, **kwargs: Any) -> MaterialBrowserPanel:
    """Single factory for all APS material library mount points."""
    return MaterialBrowserPanel(master, mount=mount, **kwargs)
