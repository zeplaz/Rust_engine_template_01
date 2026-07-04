"""APSR-S1 — AtlasService owns ``atlas_folder`` + ``tile_batch_path``."""

from __future__ import annotations

from rust_engine_mcp.paths import repo_root

from ..aps_state_writer import SuiteStateWriter
from ..state import SuiteState

OWNER = "AtlasService"


class AtlasService:
    def __init__(self, state: SuiteState, writer: SuiteStateWriter) -> None:
        self._state = state
        self._writer = writer

    def set_atlas_folder(self, folder: str | None) -> None:
        self._writer.set(self._state, "atlas_folder", folder, owner=OWNER)

    def set_tile_batch_path(self, path: str | None) -> None:
        self._writer.set(self._state, "tile_batch_path", path, owner=OWNER)

    def set_folder_from_assembly_id(self, assembly_id: str) -> str:
        folder = str((repo_root() / "assets/staging/tiles" / assembly_id).resolve())
        self.set_atlas_folder(folder)
        return folder

    def set_folder_from_tile_batch_id(self, batch_id: str) -> str | None:
        folder = repo_root() / "assets/staging/tiles" / batch_id
        if not folder.is_dir():
            return None
        path = str(folder.resolve())
        self.set_atlas_folder(path)
        return path
