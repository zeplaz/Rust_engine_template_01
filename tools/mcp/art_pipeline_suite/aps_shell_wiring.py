"""APSR-S3 — shell event wiring (panels react; app.py publishes only)."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from .state import ArtDomain

if TYPE_CHECKING:
    from .app import ArtPipelineSuiteApp


def wire_shell_events(app: ArtPipelineSuiteApp) -> None:
    """Subscribe panel sync handlers to shell events (LaneChanged, SendToAssembly)."""
    bus = app.event_bus

    def on_lane_changed(payload: dict[str, Any]) -> None:
        lane = str(payload.get("lane") or "")
        if lane == ArtDomain.BUILDINGS.value:
            app.assembly.sync_from_state()
            return
        if lane != ArtDomain.LANDSCAPE.value:
            return

        def refresh_landscape() -> None:
            if app._applied_lane != ArtDomain.LANDSCAPE.value:
                return
            app.landscape_presets.refresh_list()
            app.landscape_grammar.refresh_from_state()
            app.landscape_states.refresh_from_state()

        app.after_idle(refresh_landscape)

    def on_send_to_assembly(_payload: dict[str, Any]) -> None:
        app.assembly.sync_from_state()

    bus.subscribe("LaneChanged", on_lane_changed)
    bus.subscribe("SendToAssembly", on_send_to_assembly)
