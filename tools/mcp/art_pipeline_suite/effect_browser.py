"""VSS-T4-005 — single effect library mount entry (browse + assign).

Mount points:
  - Effects tab: ``mount_effect_library(..., mount="browse")``
  - Assign strip / remount: ``mount_effect_library(..., mount="assign")``

Both route through ``EffectBrowserPanel`` — the only wrapper over ``EffectLibraryWidget``.
"""

from __future__ import annotations

from typing import Any, TypedDict

from .effect_library_widget import EffectLibraryWidget

MOUNT_BROWSE = "browse"
MOUNT_ASSIGN = "assign"


class EffectMountConfig(TypedDict):
    mode: str
    layout: str


EFFECT_MOUNT_CONFIG: dict[str, EffectMountConfig] = {
    MOUNT_BROWSE: {"mode": "browse", "layout": "browse"},
    MOUNT_ASSIGN: {"mode": "assign", "layout": "assign"},
}


class EffectBrowserPanel(EffectLibraryWidget):
    """Canonical APS effect library wrapper — use ``mount_effect_library`` at call sites."""

    def __init__(
        self,
        master,
        *,
        mount: str = MOUNT_BROWSE,
        on_assign_effect=None,
        on_log=None,
        layout: str | None = None,
        mode: str | None = None,
        **kwargs: Any,
    ) -> None:
        preset = EFFECT_MOUNT_CONFIG.get(mount, EFFECT_MOUNT_CONFIG[MOUNT_BROWSE])
        super().__init__(
            master,
            mode=mode or preset["mode"],
            layout=layout or preset["layout"],
            on_assign_effect=on_assign_effect,
            on_log=on_log,
            **kwargs,
        )


def mount_effect_library(master, *, mount: str = MOUNT_BROWSE, **kwargs: Any) -> EffectBrowserPanel:
    """Single factory for all APS effect library mount points."""
    return EffectBrowserPanel(master, mount=mount, **kwargs)
