"""APSR-P1 — AssemblyValidationSectionMixin."""
from __future__ import annotations

import json
import tkinter as tk
from pathlib import Path
from tkinter import filedialog, messagebox, ttk
from typing import Any

from rust_engine_mcp import aps_tags, arch_build_grammar, assembly, building_grammar, grammar_build_set, library
from rust_engine_mcp.aps_grammar_labels import human_label
from rust_engine_mcp.aps_mat_auth_ui import save_hint
from rust_engine_mcp.aps_validator_plain import format_p0_display
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators.assembly_production import validate_assembly_snapshot_path
from rust_engine_mcp.validators.assembly_grammar_verify import validate_assembly_p0_gate

from .aps_collapsible import CollapsibleSection
from .aps_inline_feedback import set_inline_status
from .aps_tooltips import bind_aps_tooltip
from .assembly_panel_common import MATERIAL_AUTHORITY_COPY, grammar_combo_maps, is_dark_color


class AssemblyValidationSectionMixin:
    def _set_validation_result(self, text: str, *, ok: bool | None = None) -> None:
        set_inline_status(self._validation_lbl, self.validation_var, text, ok=ok)

    def _p0_report(self):
        import tempfile

        from rust_engine_mcp.validators.report import ValidationReport

        if not self._snapshot:
            return ValidationReport(
                validator="assembly_p0",
                status="failed",
                compression_level=3,
                summary="no snapshot",
                error_count=1,
                errors=[],
            )
        path = self.state.assembly_snapshot_path
        if path and (repo_root() / path).is_file():
            snap_path = str(path)
        else:
            tmp = Path(tempfile.gettempdir()) / "_aps_assembly_p0_validate.json"
            tmp.write_text(json.dumps(self._snapshot, indent=2), encoding="utf-8")
            snap_path = str(tmp)
        rep = validate_assembly_p0_gate(
            self._snapshot,
            snapshot_path=snap_path.replace("\\", "/"),
            ship=True,
        )
        # APS-UX-PIPELINE-VALIDITY-001 — record the live P0 verdict so the pipeline
        # bar can show ✓ only when the gate actually passed for this snapshot.
        self._assembly.set_p0_passed(rep.status == "passed")
        return rep

    def _format_validation_hints(rep) -> str:
        return format_p0_display(rep, limit=20)[:1200]

    def _run_p0_or_block(self, action: str) -> bool:
        rep = self._p0_report()
        if rep.status == "passed":
            self._set_validation_result(f"Ship check passed — {action} OK", ok=True)
            return True
        hints = self._format_validation_hints(rep)
        self._set_validation_result(f"Ship check failed: {hints[:200]}", ok=False)
        return messagebox.askyesno(
            f"Ship check failed — {action} anyway?",
            f"{hints}\n\nProceed anyway? (Not recommended before you ship.)",
        )

    def on_generate(self) -> None:
        seed = int(self.seed_var.get())
        tier = self.tier_var.get().strip() or "lod0"
        try:
            if self.use_grammar_var.get():
                archetype = self._resolve_archetype_id()
                district = self._resolve_district_id()
                self._on_log(f"assembly-snapshot-generate grammar {archetype}/{district} seed={seed}")
                snap = self._assembly.generate(
                    use_grammar=True,
                    seed=seed,
                    source_tier=tier,
                    archetype_id=archetype,
                    district_style=district,
                )
            else:
                style = self.style_var.get().strip()
                fp = self.footprint_var.get().strip().lower()
                w, d = fp.split("x")
                width, depth = int(w), int(d)
                floors = int(self.floors_var.get())
                self._on_log(f"assembly-snapshot-generate {style} {width}x{depth} tier={tier}")
                snap = self._assembly.generate(
                    use_grammar=False,
                    seed=seed,
                    source_tier=tier,
                    style_pack_id=style,
                    width=width,
                    depth=depth,
                    floors=floors,
                )
        except Exception as exc:  # noqa: BLE001
            self._set_validation_result(f"Generate failed: {exc}", ok=False)
            return
        snap = self._apply_grammar_dna_from_ui(snap)
        self._load_snapshot_into_ui(snap, path_hint=str(snap.get("written_path") or ""))
        self._on_log(f"wrote {self.state.assembly_snapshot_path}")
        # P7 Slice B — the pipeline spine owns the "what's next" walkthrough; this
        # stays a short in-context hint about the work area, not a second pipeline nav.
        self.next_step_var.set(
            "Select a footprint cell to assign a material (Catalog tags are hints only)."
        )
        bind_aps_tooltip(self._next_step_lbl, "asm_save_reminder")
        rep = self._p0_report()
        if rep.status == "passed":
            self._set_validation_result(
                f"Assembly saved · {self.state.assembly_id} · ship check passed",
                ok=True,
            )
        else:
            hints = self._format_validation_hints(rep)
            self._set_validation_result(f"Ship check failed: {hints[:200]}", ok=False)
            self._on_log(f"generate ship check failed: {hints[:400]}")

    def on_load(self) -> None:
        initial = repo_root() / "assets" / "staging" / "assemblies"
        path = filedialog.askopenfilename(
            title="Load assembly snapshot",
            initialdir=str(initial) if initial.is_dir() else str(repo_root()),
            filetypes=[("JSON", "*.json"), ("All", "*.*")],
        )
        if not path:
            return
        try:
            snap = assembly.load_assembly_snapshot(path)
        except Exception as exc:  # noqa: BLE001
            self._set_validation_result(f"Load failed: {exc}", ok=False)
            return
        rel = Path(path).resolve().relative_to(repo_root()).as_posix()
        snap["written_path"] = rel
        self._load_snapshot_into_ui(snap, path_hint=rel)
        self._on_log(f"loaded {rel}")

    def on_save(self) -> None:
        if not self._snapshot:
            self._set_validation_result("Generate or load a snapshot first.", ok=False)
            return
        if not self._run_p0_or_block("Save"):
            self._on_log("save cancelled — ship check failed")
            return
        try:
            snap = self._apply_grammar_dna_from_ui(self._snapshot)
            out = assembly.save_assembly_snapshot(snap)
        except Exception as exc:  # noqa: BLE001
            self._set_validation_result(f"Save failed: {exc}", ok=False)
            return
        rel = str(out.relative_to(repo_root())).replace("\\", "/")
        snap["written_path"] = rel
        self._assembly.set_snapshot_path(rel)
        self._commit_snapshot(snap)
        self.path_var.set(rel)
        self._on_log(f"saved {rel}")
        self.save_hint_var.set(save_hint(snap))
        hint = save_hint(snap)
        msg = f"Saved {rel}"
        if "missing material_profile" in hint:
            msg += f" · {hint}"
        self._set_validation_result(msg, ok=True)

    def on_validate(self) -> None:
        if not self._snapshot:
            self._set_validation_result("No snapshot loaded.", ok=False)
            return
        path = self.state.assembly_snapshot_path
        if path:
            rep = validate_assembly_snapshot_path(repo_root() / path, ship=True)
        else:
            import tempfile

            tmp = Path(tempfile.gettempdir()) / "_aps_assembly_validate.json"
            tmp.write_text(json.dumps(self._snapshot, indent=2), encoding="utf-8")
            rep = validate_assembly_snapshot_path(tmp, ship=True)
        self._show_validation_report(rep, title="Check schema")

    def on_validate_p0(self) -> None:
        if not self._snapshot:
            self._set_validation_result("No snapshot loaded.", ok=False)
            return
        rep = self._p0_report()
        self._show_validation_report(rep, title="Ship check")

    def _show_validation_report(self, rep, *, title: str) -> None:
        if rep.status == "passed":
            self._set_validation_result(f"{title}: passed", ok=True)
            self._on_log(f"{title}: passed")
        else:
            hints = self._format_validation_hints(rep)
            self._set_validation_result(f"{title} failed: {hints[:200]}", ok=False)
            self._on_log(f"{title} failed: {hints[:800]}")
