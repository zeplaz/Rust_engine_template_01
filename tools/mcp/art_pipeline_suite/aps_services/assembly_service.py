"""APSR-S2 — AssemblyService owns ``assembly_*`` SuiteState fields."""

from __future__ import annotations

import copy
from typing import Any

from rust_engine_mcp import assembly

from ..aps_state_writer import SuiteStateWriter
from ..state import SuiteState

OWNER = "AssemblyService"


class AssemblyService:
    def __init__(self, state: SuiteState, writer: SuiteStateWriter) -> None:
        self._state = state
        self._writer = writer

    @property
    def snapshot(self) -> dict[str, Any] | None:
        return self._state.assembly_snapshot_data

    def set_snapshot_data(self, snap: dict[str, Any]) -> dict[str, Any]:
        enriched = assembly.enrich_snapshot(snap)
        self._commit_snapshot_fields(enriched)
        return enriched

    def set_snapshot_path(self, rel: str) -> None:
        self._writer.set(self._state, "assembly_snapshot_path", rel, owner=OWNER)

    def reset_p0_verdict(self) -> None:
        self._writer.set(self._state, "assembly_p0_passed", None, owner=OWNER)

    def set_p0_passed(self, passed: bool) -> None:
        self._writer.set(self._state, "assembly_p0_passed", passed, owner=OWNER)

    def patch_snapshot_from_shell(self) -> dict[str, Any] | None:
        """Align live snapshot metadata with catalog/shell chrome."""
        snap = self.snapshot
        if not snap:
            return None
        patched = copy.deepcopy(snap)
        patched["style_pack_id"] = self._state.style_pack_id
        fp_str = str(self._state.footprint or "").strip().lower()
        if "x" in fp_str:
            w, d = fp_str.split("x", 1)
            footprint = dict(patched.get("footprint") or {})
            footprint["width"] = int(w)
            footprint["depth"] = int(d)
            footprint["floors"] = int(self._state.floors)
            patched["footprint"] = footprint
        patched["seed"] = int(self._state.seed)
        return self.set_snapshot_data(patched)

    def sync_shell_from_snapshot(self, snap: dict[str, Any]) -> None:
        """Mirror snapshot authority into shell chrome (non-owned fields)."""
        self._state.style_pack_id = str(snap.get("style_pack_id") or self._state.style_pack_id)
        fp = snap.get("footprint") or {}
        w, d, f = fp.get("width"), fp.get("depth"), fp.get("floors")
        if w and d:
            self._state.footprint = f"{w}x{d}"
        if f is not None:
            self._state.floors = int(f)
        if snap.get("seed") is not None:
            self._state.seed = int(snap["seed"])
        self._commit_snapshot_fields(snap)

    def generate(
        self,
        *,
        use_grammar: bool,
        seed: int,
        source_tier: str,
        style_pack_id: str = "",
        width: int = 0,
        depth: int = 0,
        floors: int = 0,
        archetype_id: str = "",
        district_style: str = "",
    ) -> dict[str, Any]:
        if use_grammar:
            snap = assembly.generate_assembly_snapshot(
                archetype_id=archetype_id,
                district_style=district_style,
                seed=seed,
                source_tier=source_tier,
            )
        else:
            snap = assembly.generate_assembly_snapshot(
                style_pack_id=style_pack_id,
                width=width,
                depth=depth,
                floors=floors,
                seed=seed,
                source_tier=source_tier,
            )
        return snap

    def _commit_snapshot_fields(self, snap: dict[str, Any]) -> None:
        assembly_id = snap.get("assembly_id")
        if assembly_id is not None:
            self._writer.set(self._state, "assembly_id", str(assembly_id), owner=OWNER)
        rel = snap.get("written_path")
        if rel:
            self._writer.set(self._state, "assembly_snapshot_path", str(rel), owner=OWNER)
        module_ids = sorted(
            {str(p.get("module_id")) for p in snap.get("module_placements") or [] if p.get("module_id")}
        )
        self._writer.set(self._state, "module_ids_in_assembly", module_ids, owner=OWNER)
        self._writer.set(self._state, "assembly_snapshot_data", snap, owner=OWNER)
