"""Shared cross-tab state for Art Pipeline Suite."""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import Any


class ArtDomain(str, Enum):
    """MCP-APS-STATE-SCAFFOLD-001 — domain router scaffold (E1 wires UI)."""

    BUILDINGS = "buildings"
    LANDSCAPE = "landscape"


@dataclass
class SuiteState:
    art_domain: str = ArtDomain.BUILDINGS.value
    selected_landscape_preset_id: str | None = None
    landscape_preset_validate_ok: bool | None = None
    landscape_grammar_saved: bool = False
    landscape_states_ready: bool = False
    landscape_catalog_validate_ok: bool | None = None
    landscape_stamp_registered: bool = False
    selected_module_id: str | None = None
    selected_module_ids: list[str] = field(default_factory=list)
    style_pack_id: str = "style_victorian"
    footprint: str = "4x3"
    floors: int = 2
    seed: int = 42
    assembly_id: str | None = None
    assembly_snapshot_path: str | None = None
    assembly_snapshot_data: dict[str, Any] | None = None
    # APS-UX-PIPELINE-VALIDITY-001 — None = P0 not run yet for the current snapshot,
    # True = P0 gate passed, False = P0 gate failed. Reset to None on generate/load.
    assembly_p0_passed: bool | None = None
    module_ids_in_assembly: list[str] = field(default_factory=list)
    variant_set_path: str | None = None
    variant_set_data: dict[str, Any] | None = None
    selected_variant_key: str | None = None
    tile_batch_path: str | None = None
    atlas_folder: str | None = None
    grammar_set_tier: str = "G0"
    grammar_sweep_stale: bool = False
    log_lines: list[str] = field(default_factory=list)

    def append_log(self, line: str) -> None:
        self.log_lines.append(line)
