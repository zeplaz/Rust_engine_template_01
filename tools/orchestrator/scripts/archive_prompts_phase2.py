#!/usr/bin/env python3
"""Phase 2 — archive prompts/guides + relocate user/reference material.

Keeps a minimal active spine in prompts/; moves closed guides to
docs/archive/2026-06-prompts-guides/ and user/outside docs to docs/reference/.
"""

from __future__ import annotations

import argparse
import json
import shutil
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
ARCHIVE_ROOT = REPO / "docs" / "archive" / "2026-06-prompts-guides"
REFERENCE_ROOT = REPO / "docs" / "reference"

# Stay in prompts/ — agent contracts + code-adjacent runbooks
KEEP_REL: set[str] = {
    "prompts/llm_agent_brief.md",
    "prompts/README.md",
    "prompts/INDEX.md",
    "prompts/guides/README.md",
    "prompts/guides/stage5_convergence_directive_v1.md",
    "prompts/guides/subagent_continuity_playbook_v1.md",
    "prompts/guides/base_finsh_5.md",
    "prompts/guides/operational_readiness_vs_infrastructure_perf_v1.md",
    "prompts/guides/ui_boundary_guide_v1.md",
    "prompts/guides/weather_simulation_runbook_v1.md",
    "prompts/guides/base_fire_sim.md",
    "prompts/designer_questions/README.md",
    "prompts/designer_questions/aps_atlas_legend_brief_v1.md",
    "prompts/designer_questions/aps_bevy_qc_hud_brief_v1.md",
    "prompts/designer_questions/aps_materials_tab_ia_brief_v1.md",
    "prompts/designer_questions/aps_tooltip_copy_brief_v1.md",
    "prompts/designer_questions/aps_tooltip_copy_v1.md",
    "prompts/designer_questions/aps_ux_audit_brief_v1.md",
    "prompts/designer_questions/grammar_iter_wireframe_brief_v1.md",
    "prompts/designer_questions/grammar_iter_wireframe_v1.md",
    "prompts/designer_questions/sim_hud_product_brief_v1.md",
    "prompts/designer_questions/weather_player_read_brief_v1.md",
}

# User / outside / legacy — docs/reference (not workfiles)
REFERENCE_MAP: list[tuple[str, str]] = [
    ("docs/reference/outside/effwecny_mpc_draft.md", "outside/effwecny_mpc_draft.md"),
    ("docs/reference/user/art_design_inbound.md", "user/art_design_inbound.md"),
    ("docs/reference/user/econ_followup.md", "user/econ_followup.md"),
    ("docs/reference/outside/implementation_plan_v1.md", "outside/implementation_plan_v1.md"),
    ("docs/reference/user/designer/art_design.md", "user/designer/art_design.md"),
    ("docs/reference/user/designer/art_extend.md", "user/designer/art_extend.md"),
    (
        "docs/reference/outside/dsm_ops_subagent_tooling.ini",
        "outside/dsm_ops_subagent_tooling.ini",
    ),
    (
        "docs/reference/legacy/bevy_notes_old.md",
        "legacy/bevy_notes_old.md",
    ),
    ("docs/reference/legacy/_legacy_sidrn_file_inventory.txt", "legacy/_legacy_sidrn_file_inventory.txt"),
    ("docs/reference/legacy/_legacy_railhubz_file_inventory.txt", "legacy/_legacy_railhubz_file_inventory.txt"),
    ("docs/reference/legacy/_legacy_razerz_file_inventory.txt", "legacy/_legacy_razerz_file_inventory.txt"),
    ("docs/reference/legacy/legacy_cpp_repos_agent_communication_maps_v1.md", "legacy/legacy_cpp_repos_agent_communication_maps_v1.md"),
    ("docs/reference/user/developer_reflective_brief_v1.md", "user/developer_reflective_brief_v1.md"),
    ("docs/reference/user/developer_reflective_brief_v1.plan.md", "user/developer_reflective_brief_v1.plan.md"),
    ("docs/reference/user/new_proposal_guide_may202608.md", "user/new_proposal_guide_may202608.md"),
]

DESIGNER_DOMAIN_DIRS = (
    "terrain_world",
    "transport",
    "factions",
    "navigation",
    "production_economy",
    "strategic_platforms",
    "tools_ui",
    "map_editor",
)


def _rel(path: Path) -> str:
    return path.relative_to(REPO).as_posix()


def _move_file(src: Path, dest: Path, *, dry_run: bool) -> dict | None:
    if not src.is_file():
        return None
    entry = {"from": _rel(src), "to": _rel(dest)}
    if dry_run:
        return entry
    dest.parent.mkdir(parents=True, exist_ok=True)
    if dest.exists():
        dest.unlink()
    shutil.move(str(src), str(dest))
    return entry


def _move_tree(src_dir: Path, dest_dir: Path, *, dry_run: bool) -> list[dict]:
    moved: list[dict] = []
    if not src_dir.is_dir():
        return moved
    for src in sorted(src_dir.rglob("*")):
        if not src.is_file():
            continue
        rel = src.relative_to(src_dir)
        dest = dest_dir / rel
        entry = _move_file(src, dest, dry_run=dry_run)
        if entry:
            moved.append(entry)
    if not dry_run and src_dir.is_dir():
        for child in sorted(src_dir.rglob("*"), reverse=True):
            if child.is_file():
                continue
            try:
                child.rmdir()
            except OSError:
                pass
        try:
            src_dir.rmdir()
        except OSError:
            pass
    return moved


def collect_prompts_files() -> list[Path]:
    root = REPO / "prompts"
    return sorted(p for p in root.rglob("*") if p.is_file())


def archive_bucket(rel: str) -> str:
    if rel.startswith("prompts/guides/ui/"):
        return "ui_phases"
    if rel.startswith("prompts/matrix/"):
        return "matrix"
    if rel.startswith("prompts/guides/"):
        return "runbooks"
    if rel.startswith("prompts/designer_questions/"):
        return "designer_domains"
    if rel.startswith("prompts/MCP/"):
        return "mcp_drafts"
    return "misc"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    moved: list[dict] = []
    skipped_keep: list[str] = []

    for src_rel, ref_rel in REFERENCE_MAP:
        src = REPO / Path(src_rel)
        dest = REFERENCE_ROOT / ref_rel
        entry = _move_file(src, dest, dry_run=args.dry_run)
        if entry:
            entry["bucket"] = "reference"
            moved.append(entry)

    for domain in DESIGNER_DOMAIN_DIRS:
        src_dir = REPO / "prompts" / "designer_questions" / domain
        dest_dir = REFERENCE_ROOT / "designer_questions" / domain
        for entry in _move_tree(src_dir, dest_dir, dry_run=args.dry_run):
            entry["bucket"] = "reference"
            moved.append(entry)

    for src in collect_prompts_files():
        rel = _rel(src)
        if rel in KEEP_REL:
            skipped_keep.append(rel)
            continue
        if any(m["from"] == rel for m in moved):
            continue
        bucket = archive_bucket(rel)
        dest = ARCHIVE_ROOT / bucket / Path(rel).relative_to("prompts")
        entry = _move_file(src, dest, dry_run=args.dry_run)
        if entry:
            entry["bucket"] = bucket
            moved.append(entry)

    log = {
        "schema": "archive_moved_log_v1",
        "phase": "prompts-guides-phase2",
        "archive_root": _rel(ARCHIVE_ROOT),
        "reference_root": _rel(REFERENCE_ROOT),
        "moved_at": datetime.now(timezone.utc).isoformat(),
        "dry_run": args.dry_run,
        "moved_count": len(moved),
        "kept_count": len(skipped_keep),
        "kept": sorted(skipped_keep),
        "moved": moved,
    }

    if args.dry_run:
        print(json.dumps({"moved_count": len(moved), "kept_count": len(skipped_keep)}, indent=2))
        return 0

    ARCHIVE_ROOT.mkdir(parents=True, exist_ok=True)
    REFERENCE_ROOT.mkdir(parents=True, exist_ok=True)
    log_path = ARCHIVE_ROOT / "MOVED_LOG.json"
    log_path.write_text(json.dumps(log, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {log_path} ({len(moved)} moved, {len(skipped_keep)} kept)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
