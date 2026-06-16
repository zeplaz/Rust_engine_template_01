#!/usr/bin/env python3
"""Archive superseded fleet/planner/dispatch docs out of src/dev and prompts drafts.

Writes docs/archive/2026-06-fleet-drain/MOVED_LOG.json. Idempotent: skips missing sources.
"""

from __future__ import annotations

import argparse
import json
import shutil
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
ARCHIVE_ROOT = REPO / "docs" / "archive" / "2026-06-fleet-drain"

# (source relative to repo, archive subfolder under ARCHIVE_ROOT)
MANIFEST: list[tuple[str, str]] = [
    # Planner audits v5–v15 (v16+ remain active ledger chain)
    *[(f"src/dev/planner_status_audit_v{v}.md", "planner_audits") for v in range(5, 16)],
    # Ledger checklists superseded by v16–v19 chain
    ("docs/archive/2026-06-fleet-drain/planner_audits/plan_ledger_refresh_010_checklist_v1.md", "planner_audits"),
    ("docs/archive/2026-06-fleet-drain/planner_audits/plan_ledger_refresh_015_checklist_v1.md", "planner_audits"),
    # Closed fleet snapshots + wave dispatches (20260527–60528)
    ("docs/archive/2026-06-fleet-drain/fleet_closed/fleet_snapshot_20260527_v1.md", "fleet_closed"),
    ("docs/archive/2026-06-fleet-drain/fleet_closed/fleet_snapshot_20260528_v1.md", "fleet_closed"),
    ("docs/archive/2026-06-fleet-drain/fleet_closed/fleet_snapshot_20260528_v2.md", "fleet_closed"),
    ("docs/archive/2026-06-fleet-drain/fleet_closed/fleet_signoff_wave_closure_20260527_v1.md", "fleet_closed"),
    ("docs/archive/2026-06-fleet-drain/fleet_closed/fleet_signoff_wave3_closure_20260527_v1.md", "fleet_closed"),
    ("docs/archive/2026-06-fleet-drain/fleet_closed/fleet_signoff_wave4_coder_partial_20260527_v1.md", "fleet_closed"),
    ("docs/archive/2026-06-fleet-drain/fleet_closed/fleet_wave3_assignments_20260527_v1.md", "fleet_closed"),
    ("docs/archive/2026-06-fleet-drain/fleet_closed/fleet_wave4_assignments_20260527_v1.md", "fleet_closed"),
    ("docs/archive/2026-06-fleet-drain/fleet_closed/fleet_wave5_coder_dispatch_v1.md", "fleet_closed"),
    ("docs/archive/2026-06-fleet-drain/fleet_closed/fleet_wave6_coder_dispatch_v1.md", "fleet_closed"),
    ("docs/archive/2026-06-fleet-drain/fleet_closed/fleet_wave7_coder_dispatch_v1.md", "fleet_closed"),
    ("docs/archive/2026-06-fleet-drain/fleet_closed/fleet_coder_workboard_20260528_v3.md", "fleet_closed"),
    ("docs/archive/2026-06-fleet-drain/fleet_closed/fleet_coder_workload_queue_20260602_v1.md", "fleet_closed"),
    ("docs/archive/2026-06-fleet-drain/fleet_closed/fleet_planner_designer_prompts_20260602_v2.md", "fleet_closed"),
    ("docs/archive/2026-06-fleet-drain/fleet_closed/fleet_maturity_signoff_routing_20260527_v1.md", "fleet_closed"),
    ("docs/archive/2026-06-fleet-drain/fleet_closed/fleet_stability_coder_dispatch_v1.md", "fleet_closed"),
    ("docs/archive/2026-06-fleet-drain/fleet_closed/fleet_stability_phase2_dispatch_v1.md", "fleet_closed"),
    ("docs/archive/2026-06-fleet-drain/fleet_closed/mcp_fleet_wave2_orders_v1.md", "fleet_closed"),
    ("docs/archive/2026-06-fleet-drain/fleet_closed/coder_fleet_return_recap_wave3_v1.md", "fleet_closed"),
    ("docs/archive/2026-06-fleet-drain/fleet_closed/coder_fleet_active_assignments_v1.md", "fleet_closed"),
    ("docs/archive/2026-06-fleet-drain/fleet_closed/coder_fleet_multistage_matrix_v1.md", "fleet_closed"),
    ("docs/archive/2026-06-fleet-drain/fleet_closed/coder_wave3_full_todos_v1.md", "fleet_closed"),
    # Dated dispatch snapshots (superseded by post_drain queues)
    ("docs/archive/2026-06-fleet-drain/dev_dispatch/orchestrator_collective_dispatch_20260607_v1.md", "dev_dispatch"),
    ("docs/archive/2026-06-fleet-drain/dev_dispatch/orchestrator_coder_dispatch_20260603_v1.md", "dev_dispatch"),
    ("docs/archive/2026-06-fleet-drain/dev_dispatch/snapshot_drain_review_20260607_v1.md", "dev_dispatch"),
    ("docs/archive/2026-06-fleet-drain/dev_dispatch/agent_prompt_pack_20260607_v1.md", "dev_dispatch"),
    # Prompt drafts (skills canonical in .cursor/skills)
    ("docs/archive/2026-06-fleet-drain/prompts_drafts/mcp_drafts.md", "prompts_drafts"),
    ("docs/archive/2026-06-fleet-drain/prompts_drafts/rules_skills_draft.md", "prompts_drafts"),
    ("docs/archive/2026-06-fleet-drain/prompts_drafts/planner_fix_auto_build.md", "prompts_drafts"),
    ("docs/archive/2026-06-fleet-drain/prompts_drafts/base_visual_dev01_plan_status.md", "prompts_drafts"),
]

ROUGH_AGENTS = REPO / "prompts" / "rough_agents"


def _rel(path: Path) -> str:
    return path.relative_to(REPO).as_posix()


def archive_one(src: Path, bucket: str, *, dry_run: bool) -> dict | None:
    if not src.is_file():
        return None
    dest_dir = ARCHIVE_ROOT / bucket
    dest = dest_dir / src.name
    entry = {
        "from": _rel(src),
        "to": _rel(dest),
        "bucket": bucket,
    }
    if dry_run:
        return entry
    dest_dir.mkdir(parents=True, exist_ok=True)
    if dest.exists():
        dest.unlink()
    shutil.move(str(src), str(dest))
    return entry


def archive_rough_agents(*, dry_run: bool) -> list[dict]:
    moved: list[dict] = []
    if not ROUGH_AGENTS.is_dir():
        return moved
    bucket = "prompts_rough_agents"
    dest_root = ARCHIVE_ROOT / bucket
    for src in sorted(ROUGH_AGENTS.rglob("*")):
        if not src.is_file():
            continue
        rel = src.relative_to(ROUGH_AGENTS)
        dest = dest_root / rel
        entry = {"from": _rel(src), "to": _rel(dest), "bucket": bucket}
        moved.append(entry)
        if dry_run:
            continue
        dest.parent.mkdir(parents=True, exist_ok=True)
        if dest.exists():
            dest.unlink()
        shutil.move(str(src), str(dest))
    if not dry_run and ROUGH_AGENTS.is_dir():
        try:
            ROUGH_AGENTS.rmdir()
        except OSError:
            pass
    return moved


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    moved: list[dict] = []
    skipped: list[str] = []

    for rel, bucket in MANIFEST:
        src = REPO / rel.replace("/", "\\") if "\\" in rel else REPO / rel
        src = REPO / Path(rel)
        entry = archive_one(src, bucket, dry_run=args.dry_run)
        if entry:
            moved.append(entry)
        else:
            skipped.append(rel)

    moved.extend(archive_rough_agents(dry_run=args.dry_run))

    log = {
        "schema": "archive_moved_log_v1",
        "archive_root": _rel(ARCHIVE_ROOT),
        "moved_at": datetime.now(timezone.utc).isoformat(),
        "dry_run": args.dry_run,
        "moved_count": len(moved),
        "skipped_missing": skipped,
        "moved": moved,
    }

    if not args.dry_run:
        ARCHIVE_ROOT.mkdir(parents=True, exist_ok=True)
        log_path = ARCHIVE_ROOT / "MOVED_LOG.json"
        log_path.write_text(json.dumps(log, indent=2) + "\n", encoding="utf-8")
        print(f"wrote {log_path} ({len(moved)} files)")
    else:
        print(json.dumps(log, indent=2))

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
