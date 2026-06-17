"""DMCP-OVR-P2-IMPL-AUDIT-001 — P2 implementation vs copy pack + G0 ban-list."""

from __future__ import annotations

import json
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Literal

from rust_engine_mcp.aps_uiux_g0_audit import run_ban_list_audit
from rust_engine_mcp.paths import repo_root

WITNESS_REL = "debug_runs/art_pipeline/dmcp_ovr_p2_impl_audit_live.json"
AUDIT_DOC_REL = "src/dev/design_aps_uiux_p2_impl_audit_v1.md"
COPY_PACK_REL = "src/dev/design_aps_uiux_copy_pack_v1.md"
G0_AUDIT_REL = "src/dev/design_aps_uiux_g0_audit_v1.md"
SUITE_REL = "tools/mcp/art_pipeline_suite"

CheckKind = Literal["ui_must", "ui_should", "deferred_p3", "log_ok_old"]


@dataclass(frozen=True)
class CopyCheck:
    check_id: str
    section: str
    path: str
    old: str
    new: str
    kind: CheckKind = "ui_must"
    note: str = ""

    def evaluate(self, root: Path) -> dict[str, Any]:
        rel = self.path.replace("\\", "/")
        fp = root / rel
        if not fp.is_file():
            return {
                "id": self.check_id,
                "section": self.section,
                "path": rel,
                "status": "missing_file",
                "kind": self.kind,
                "note": self.note,
            }
        text = fp.read_text(encoding="utf-8", errors="replace")
        has_old = self.old in text
        has_new = self.new in text
        if self.kind == "deferred_p3":
            status = "deferred" if has_old and not has_new else ("pass" if has_new else "deferred")
        elif self.kind == "log_ok_old":
            status = "pass" if has_new else ("partial" if has_old else "fail")
        else:
            if has_new and not has_old:
                status = "pass"
            elif has_new and has_old:
                status = "partial"
            elif not has_new and has_old:
                status = "fail"
            else:
                status = "fail"
        return {
            "id": self.check_id,
            "section": self.section,
            "path": rel,
            "status": status,
            "kind": self.kind,
            "old": self.old,
            "new": self.new,
            "note": self.note,
        }


COPY_PACK_CHECKS: tuple[CopyCheck, ...] = (
    CopyCheck("CP-01", "§1 global", f"{SUITE_REL}/domain_router.py", "Ship truth:", "What ships:", "ui_must"),
    CopyCheck(
        "CP-02",
        "§1 global",
        f"{SUITE_REL}/pipeline_status_bar.py",
        "Keyframe bake is behind Atlas",
        "You can build, assign materials, and preview without baking tiles",
        "ui_must",
    ),
    CopyCheck(
        "CP-03",
        "§1 global",
        f"{SUITE_REL}/pipeline_pills.py",
        "saved (QC not run)",
        "saved (not checked)",
        "ui_must",
    ),
    CopyCheck(
        "CP-04",
        "§1 global",
        f"{SUITE_REL}/app.py",
        "All actions call rust_engine_mcp CLI/MCP",
        "Every button here runs the same tools the build pipeline uses",
        "deferred_p3",
        note="app.py chrome deferred OVR-P3-LAYOUT-001",
    ),
    CopyCheck(
        "CP-05",
        "§2 assembly",
        f"{SUITE_REL}/assembly_panel.py",
        "Material authority (APS-MAT-AUTH-UI-001)",
        "Where materials come from",
        "ui_must",
    ),
    CopyCheck("CP-06", "§2 assembly", f"{SUITE_REL}/assembly_panel.py", "Generate snapshot", "Generate Assembly", "ui_must"),
    CopyCheck("CP-07", "§2 assembly", f"{SUITE_REL}/assembly_panel.py", 'text="P0 gate"', "Run ship check", "ui_must"),
    CopyCheck("CP-08", "§2 assembly", f"{SUITE_REL}/assembly_panel.py", 'text="Validate"', "Check schema", "ui_must"),
    CopyCheck(
        "CP-09",
        "§2 assembly",
        f"{SUITE_REL}/assembly_panel.py",
        "P0 gate failed",
        "Ship check failed",
        "log_ok_old",
        note="status/log/dialog — artist-visible",
    ),
    CopyCheck(
        "CP-10",
        "§2 assembly",
        f"{SUITE_REL}/grammar_build_set_panel.py",
        "Grammar set (BUILD-SET)",
        "Building style set",
        "ui_must",
    ),
    CopyCheck(
        "CP-11",
        "§2 assembly",
        f"{SUITE_REL}/grammar_dna_panel.py",
        "Store ARCH-DNA",
        "Save shape settings with this building",
        "ui_must",
    ),
    CopyCheck(
        "CP-12",
        "§2 assembly",
        f"{SUITE_REL}/grammar_dna_panel.py",
        "ARCH-DNA (read-only from preset)",
        "Shape profile (from preset, read-only)",
        "ui_must",
    ),
    CopyCheck(
        "CP-13",
        "§3 metadata",
        f"{SUITE_REL}/metadata_flow_panel.py",
        "Metadata → engine (ARCH-MAT-001)",
        "Where this data goes",
        "ui_must",
    ),
    CopyCheck(
        "CP-14",
        "§3 metadata",
        f"{SUITE_REL}/metadata_flow_panel.py",
        "assembly_snapshot (AUTHORITY)",
        "What you save in this Assembly is the source of truth",
        "ui_must",
    ),
    CopyCheck(
        "CP-15",
        "§4 catalog",
        f"{SUITE_REL}/catalog.py",
        "Sidecar tags ≠ ship truth",
        "Tags here are hints only",
        "ui_must",
    ),
    CopyCheck(
        "CP-16",
        "§4 catalog",
        f"{SUITE_REL}/catalog.py",
        "AssetSpec sidecar",
        "Module info (editable)",
        "ui_must",
    ),
    CopyCheck(
        "CP-17",
        "§4 catalog",
        f"{SUITE_REL}/catalog.py",
        "3D preview (trimesh)",
        "Quick 3D preview",
        "ui_must",
    ),
    CopyCheck(
        "CP-18",
        "§5 materials",
        f"{SUITE_REL}/material_preview_modes.py",
        "Preview modes (APS-MAT-002)",
        'text="Preview"',
        "ui_must",
    ),
    CopyCheck(
        "CP-19",
        "§5 materials",
        f"{SUITE_REL}/material_library_widget.py",
        "Regenerate all pilots",
        "Regenerate sample materials",
        "ui_should",
    ),
    CopyCheck(
        "CP-20",
        "§6 variants",
        f"{SUITE_REL}/variants_panel.py",
        "variant_set_v1 — declarative layers",
        "Variant set — states of the same building",
        "ui_must",
    ),
    CopyCheck(
        "CP-21",
        "§6 variants",
        f"{SUITE_REL}/variants_panel.py",
        "Agent patch strip",
        "Ask AI for a variant (advanced)",
        "ui_must",
    ),
    CopyCheck(
        "CP-22",
        "§7 atlas",
        f"{SUITE_REL}/atlas_panel.py",
        "tile_batch_v1",
        "Tile job file",
        "ui_must",
    ),
    CopyCheck(
        "CP-23",
        "§7 atlas",
        f"{SUITE_REL}/atlas_panel.py",
        "Pack atlas (tilemapgen)",
        'text="Pack atlas"',
        "ui_must",
    ),
    CopyCheck(
        "CP-24",
        "§7 atlas",
        f"{SUITE_REL}/atlas_panel.py",
        "RUST_ENGINE_ART_DEBUG_GUI",
        "Blender debug buttons are hidden",
        "ui_should",
    ),
    CopyCheck(
        "CP-25",
        "§8 landscape",
        f"{SUITE_REL}/landscape_presets_panel.py",
        "Must-read (DMCP-E2 preset QC)",
        "Preset summary",
        "ui_must",
    ),
    CopyCheck(
        "CP-26",
        "§8 landscape",
        f"{SUITE_REL}/landscape_grammar_panel.py",
        "land_dna + topology_graph",
        "landscape layout graph (this is what ships)",
        "ui_must",
    ),
    CopyCheck(
        "CP-27",
        "§8 landscape",
        f"{SUITE_REL}/landscape_states_panel.py",
        "succession + disturbance matrix",
        "growth stages & fire",
        "ui_must",
    ),
    CopyCheck(
        "CP-28",
        "§8 landscape",
        f"{SUITE_REL}/landscape_extract_parity_panel.py",
        "route to @coder",
        "flag this to engineering",
        "ui_must",
    ),
    CopyCheck(
        "CP-29",
        "§9 preview",
        f"{SUITE_REL}/slot_preview_panel.py",
        "Selected slot previews (APS-PREVIEW-001)",
        "Selected piece previews",
        "ui_must",
    ),
    CopyCheck(
        "CP-30",
        "§2 assembly",
        f"{SUITE_REL}/footprint_canvas.py",
        "Generate snapshot to show grid",
        "Generate Assembly to show grid",
        "ui_should",
    ),
    CopyCheck(
        "CP-31",
        "§2 assembly",
        f"{SUITE_REL}/grammar_dna_panel.py",
        "Massing pressure (advanced)",
        "Building shape bias (advanced)",
        "ui_should",
        note="outer LabelFrame title; collapsible wrapper uses new copy",
    ),
    CopyCheck(
        "CP-32",
        "§7 atlas",
        f"{SUITE_REL}/atlas_preview_panel.py",
        "(no tile_map_*.png — run Pack atlas)",
        "No packed tile sheet yet",
        "ui_should",
    ),
)


def run_p2_impl_audit(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    ban = run_ban_list_audit(repo=root)
    findings = [c.evaluate(root) for c in COPY_PACK_CHECKS]
    by_status: dict[str, int] = {}
    for f in findings:
        st = str(f.get("status") or "unknown")
        by_status[st] = by_status.get(st, 0) + 1

    ui_fail = [
        f for f in findings if f.get("status") == "fail" and f.get("kind") in ("ui_must", "log_ok_old")
    ]
    ui_should_miss = [f for f in findings if f.get("status") == "fail" and f.get("kind") == "ui_should"]
    ui_partial = [f for f in findings if f.get("status") == "partial"]
    deferred = [f for f in findings if f.get("status") == "deferred" or f.get("kind") == "deferred_p3"]

    ban_clean = int(ban.get("violation_count") or 0) == 0
    copy_pass = by_status.get("pass", 0)
    copy_total = len(findings)

    if not ban_clean:
        verdict = "FAIL"
    elif ui_fail:
        verdict = "FAIL"
    elif ui_partial or ui_should_miss or deferred:
        verdict = "PASS_WITH_NOTES"
    else:
        verdict = "PASS"

    return {
        "gate": "DMCP-OVR-P2-IMPL-AUDIT-001",
        "ban_list": {
            "violation_count": ban.get("violation_count"),
            "verdict": ban.get("verdict"),
            "clean": ban_clean,
        },
        "copy_pack": {
            "total": copy_total,
            "pass": copy_pass,
            "partial": by_status.get("partial", 0),
            "fail": by_status.get("fail", 0),
            "deferred": len(deferred),
            "by_status": by_status,
        },
        "findings": findings,
        "top_misses": [f for f in findings if f.get("status") in ("fail", "partial")][:20],
        "deferred_p3": deferred,
        "audit_complete": True,
        "verdict": verdict,
        "handoff": {
            "p3": "OVR-P3-LAYOUT-001 — app.py flow caveat (CP-04)",
            "p5": "OVR-P5-STYLE-001 — footprint empty copy, atlas preview empty, material regenerate label",
            "logs": "assembly_panel P0 gate strings in status/log (CP-09)",
        },
    }


def refresh_dmcp_ovr_p2_impl_audit_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    audit = run_p2_impl_audit(repo=root)
    green = bool(audit.get("audit_complete")) and audit.get("verdict") != "FAIL"
    body: dict[str, Any] = {
        **audit,
        "green": green,
        "deliverable": AUDIT_DOC_REL,
        "copy_pack_ref": COPY_PACK_REL,
        "g0_audit_ref": G0_AUDIT_REL,
        "guard_test": "tools/mcp/python/tests/test_aps_no_jargon.py",
        "_agent_meta": {
            "schema": "dmcp_ovr_p2_impl_audit_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "DMCP_OVR_P2_IMPL_AUDIT",
            "source_system": "aps_uiux_p2_impl_audit",
            "relative_path": WITNESS_REL,
            "ritual": "BLANG:WIT-HON→Q✓ DMCP-OVR-P2-IMPL-AUDIT-001" if green else "BLANG:WIT-HON FAIL copy-pack",
            "agent": "designer-mcp",
        },
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = WITNESS_REL
    return body
