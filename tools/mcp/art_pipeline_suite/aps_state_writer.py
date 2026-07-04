"""APSR-S1 — owner-checked SuiteState field writes."""

from __future__ import annotations

from typing import Any

from .aps_event_bus import EventBus
from .state import SuiteState

FIELD_OWNERS: dict[str, str] = {
    "atlas_folder": "AtlasService",
    "tile_batch_path": "AtlasService",
    "assembly_id": "AssemblyService",
    "assembly_snapshot_path": "AssemblyService",
    "assembly_snapshot_data": "AssemblyService",
    "module_ids_in_assembly": "AssemblyService",
    "assembly_p0_passed": "AssemblyService",
}

ENFORCED_FIELDS = frozenset(FIELD_OWNERS.keys())


class SuiteStateWriteError(PermissionError):
    """Raised when a panel/service writes a field it does not own."""


class SuiteStateWriter:
    def __init__(self, bus: EventBus | None = None) -> None:
        self._bus = bus or EventBus()

    @property
    def bus(self) -> EventBus:
        return self._bus

    def owner_for(self, field: str) -> str | None:
        return FIELD_OWNERS.get(field)

    def set(self, state: SuiteState, field: str, value: Any, *, owner: str) -> None:
        expected = FIELD_OWNERS.get(field)
        if expected is None:
            setattr(state, field, value)
            return
        if owner != expected:
            raise SuiteStateWriteError(
                f"field {field!r} owned by {expected!r}, not {owner!r}"
            )
        setattr(state, field, value)
        self._bus.publish(
            "StateChanged",
            {"field": field, "value": value, "owner": owner},
        )
