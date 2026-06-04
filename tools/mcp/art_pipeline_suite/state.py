"""Shared cross-tab state for Art Pipeline Suite."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any


@dataclass
class SuiteState:
    selected_module_id: str | None = None
    selected_module_ids: list[str] = field(default_factory=list)
    style_pack_id: str = "style_victorian"
    footprint: str = "4x3"
    floors: int = 2
    seed: int = 42
    assembly_id: str | None = None
    assembly_snapshot_path: str | None = None
    assembly_snapshot_data: dict[str, Any] | None = None
    module_ids_in_assembly: list[str] = field(default_factory=list)
    variant_set_path: str | None = None
    variant_set_data: dict[str, Any] | None = None
    selected_variant_key: str | None = None
    tile_batch_path: str | None = None
    atlas_folder: str | None = None
    log_lines: list[str] = field(default_factory=list)

    def append_log(self, line: str) -> None:
        self.log_lines.append(line)
