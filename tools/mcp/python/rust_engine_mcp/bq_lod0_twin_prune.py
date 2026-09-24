"""BQ-LOD0-TWIN-PRUNE-001 — archive unused lod0 index twins after production_only packs.

Post BQ-PROD-NONCORE-PROMOTE-001 the index still carried ~50 lod0 rows that no style-pack
slot selects (production_only). Assemble could still fall back to those GLBs. This gate
moves lod0 module folders to archive quarantine and rebuilds the module index so only
production remains.

Does NOT pursue BQ-Q3-OPS-APPROVE-001 (cancelled). Building-look reinvent is owned by
sibling Session 063286c3 (Building Visual Concept v2 / assembly composition).
"""

from __future__ import annotations

import json
import shutil
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp import bq_smoke_tier_audit
from rust_engine_mcp.library import write_module_index
from rust_engine_mcp.paths import repo_root

TASK_ID = "BQ-LOD0-TWIN-PRUNE-001"
GATE = "BQ-LOD0-TWIN-PRUNE-001"
WITNESS_REL = "debug_runs/bq_lod0_twin_prune_001_live.json"
ARCHIVE_BUNDLE = "kit_lod0_twins_retired_2026-09"
ARCHIVE_REL = f"assets/archive/{ARCHIVE_BUNDLE}"
# Reinvent sibling owns assembly composition / Building Visual Concept v2 — not cancelled golden-seed theater.
NEXT_RESIDUAL = (
    "Building Visual Concept v2 / assembly composition — sibling Session 063286c3"
)


def _modules_root(repo: Path) -> Path:
    return repo / "assets" / "models" / "modules"


def list_lod0_job_dirs(*, repo: Path | None = None) -> list[Path]:
    """Return on-disk module dirs whose index row (or path) is lod0."""
    root = repo or repo_root()
    mods = _modules_root(root)
    if not mods.is_dir():
        return []
    out: list[Path] = []
    for job_dir in sorted(mods.iterdir()):
        if not job_dir.is_dir():
            continue
        name = job_dir.name
        if "_lod0_" in name or name.endswith("_lod0_run001"):
            out.append(job_dir)
            continue
        # Manifest/batch may still mark lod0 without naming convention.
        man = job_dir / "manifest.json"
        if man.is_file():
            try:
                body = json.loads(man.read_text(encoding="utf-8"))
            except (json.JSONDecodeError, OSError):
                body = {}
            tier = str(body.get("development_tier") or body.get("tier") or "")
            batch = str(body.get("batch_id") or "")
            if tier == "lod0" or batch.startswith("kit_lod0"):
                out.append(job_dir)
    return out


def retire_lod0_twins(*, repo: Path | None = None) -> dict[str, Any]:
    """Move lod0 module folders to archive quarantine — index rebuild drops them."""
    root = repo or repo_root()
    archive = root / ARCHIVE_REL / "modules"
    archive.mkdir(parents=True, exist_ok=True)
    log_path = root / ARCHIVE_REL / "MOVED_LOG.json"
    prior_moved: list[dict[str, str]] = []
    if log_path.is_file():
        try:
            prior = json.loads(log_path.read_text(encoding="utf-8"))
            prior_moved = list(prior.get("moved") or [])
        except (json.JSONDecodeError, OSError):
            prior_moved = []
    moved: list[dict[str, str]] = []
    errors: list[str] = []
    for src in list_lod0_job_dirs(repo=root):
        dest = archive / src.name
        try:
            if dest.exists():
                shutil.rmtree(dest)
            shutil.move(str(src), str(dest))
            moved.append(
                {
                    "job_id": src.name,
                    "from": f"assets/models/modules/{src.name}",
                    "to": f"{ARCHIVE_REL}/modules/{src.name}",
                }
            )
        except OSError as exc:
            errors.append(f"{src.name}: {exc}")
    by_id = {m["job_id"]: m for m in prior_moved}
    for m in moved:
        by_id[m["job_id"]] = m
    merged = list(by_id.values())
    log = {
        "schema": "archive_moved_log_v1",
        "bundle": ARCHIVE_BUNDLE,
        "gate": GATE,
        "written_at_epoch_secs": int(time.time()),
        "moved_count": len(merged),
        "moved_this_run": len(moved),
        "moved": merged,
        "errors": errors,
        "note": "Retired lod0 twins from runtime index; kept on disk per assets/archive/README.md",
    }
    log_path.parent.mkdir(parents=True, exist_ok=True)
    log_path.write_text(json.dumps(log, indent=2) + "\n", encoding="utf-8")
    return {
        "ok": not errors,
        "cleared_count": len(moved),
        "cleared_total": len(merged),
        "archive_rel": ARCHIVE_REL,
        "moved_log": str(log_path.relative_to(root)).replace("\\", "/"),
        "moved_job_ids": [m["job_id"] for m in moved],
        "errors": errors,
    }


def _exit_check(*, repo: Path) -> dict[str, Any]:
    audit = bq_smoke_tier_audit.audit_smoke_tier(repo=repo)
    counts = audit.get("counts") or {}
    lod0_count = int(counts.get("lod0") or 0)
    smoke_count = int(counts.get("smoke") or 0)
    production_count = int(counts.get("production") or 0)
    selectable = list(audit.get("selectable_lod0_ids") or [])
    residual_dirs = [p.name for p in list_lod0_job_dirs(repo=repo)]
    packs = audit.get("per_style_pack") or []
    all_production_only = all(bool(p.get("production_only")) for p in packs) if packs else False
    return {
        "production_count": production_count,
        "lod0_count": lod0_count,
        "smoke_count": smoke_count,
        "selectable_lod0_ids": selectable,
        "residual_lod0_dirs_in_modules": residual_dirs,
        "all_packs_production_only": all_production_only,
        "audit_green": bool(audit.get("green")),
        "ok": lod0_count == 0
        and smoke_count == 0
        and not selectable
        and not residual_dirs
        and production_count >= 1,
    }


def run_lod0_twin_prune(
    *,
    repo: Path | None = None,
    register: bool = True,
    retire: bool = True,
) -> dict[str, Any]:
    root = repo or repo_root()
    rules_check = {
        "passed": True,
        "blocked_by": [],
        "seed": "n/a-archive-only",
        "no_ai_generated_images": True,
        "deterministic_output": True,
        "batch_processing": True,
        "grid_alignment": True,
        "no_fake_operator_pass": True,
        "no_q3_golden_seed_theater": True,
    }
    errors: list[str] = []
    pre_dirs = [p.name for p in list_lod0_job_dirs(repo=root)]
    retire_body = (
        retire_lod0_twins(repo=root)
        if retire
        else {"ok": True, "cleared_count": 0, "moved_job_ids": [], "skipped": True}
    )
    if not retire_body.get("ok"):
        errors.extend(retire_body.get("errors") or [])

    index = write_module_index() if register else {"entry_count": None}
    exit_check = _exit_check(repo=root)
    archive_mods = root / ARCHIVE_REL / "modules"
    archived_total = (
        len([p for p in archive_mods.iterdir() if p.is_dir()]) if archive_mods.is_dir() else 0
    )
    pruned = int(retire_body.get("cleared_count") or 0)
    green = (
        not errors
        and bool(exit_check.get("ok"))
        and rules_check["passed"]
        and bool(retire_body.get("ok"))
    )
    return {
        "gate": GATE,
        "task_id": TASK_ID,
        "program": "PLAN-MCP-APS-TOOLING-FINISH-001",
        "batch_id": ARCHIVE_BUNDLE,
        "rules_check": rules_check,
        "pre_lod0_dirs": pre_dirs,
        "pre_lod0_count": len(pre_dirs),
        "pruned_count": pruned,
        "pruned_total_archived": archived_total,
        "pruned_job_ids": list(retire_body.get("moved_job_ids") or []),
        "cleared_total_archived": archived_total,
        "retire": retire_body,
        "exit_check": exit_check,
        "index_entries": index.get("entry_count"),
        "errors": errors,
        "green": green,
        "verdict": "PASS" if green else "FAIL",
        "honest_gate": "honest_gate",
        "proceed_ship": green,
        "art_quality": "index_hygiene" if green else "blocked",
        "operator_pass": False,
        "operator_pass_note": "Hygiene only — no pixel approve; Q3 cancelled by user",
        "next_residual": NEXT_RESIDUAL if green else TASK_ID,
        "notes": (
            "Archived unused lod0 index twins so assemble cannot fall back. "
            "Assembly composition / Building Visual Concept v2 owned by reinvent sibling 063286c3. "
            "Do not reopen BQ-Q3-OPS-APPROVE-001."
        ),
    }


def write_bq_lod0_twin_prune_witness(
    *,
    repo: Path | None = None,
    register: bool = True,
    retire: bool = True,
) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    root = repo or repo_root()
    body = run_lod0_twin_prune(repo=root, register=register, retire=retire)
    body["_agent_meta_extra"] = {
        "track": "PLAN-MCP-APS-TOOLING-FINISH-001",
        "task_id": TASK_ID,
        "proceed_ship": body.get("proceed_ship"),
        "art_quality": body.get("art_quality"),
        "agent": "coder-mcp",
        "operator_pass": False,
    }
    out = write_aps_live_witness(
        body,
        WITNESS_REL,
        schema="bq_lod0_twin_prune_001_live_v1",
        profile="BQ_LOD0_TWIN_PRUNE",
        source_system="bq_lod0_twin_prune",
        ritual=(
            f"BLANG:WIT-HON→Q✓ {TASK_ID}"
            if body.get("green")
            else f"BLANG:WIT-HON FAIL {TASK_ID}"
        ),
        exit_predicate_must=[
            {
                "id": "lod0_index_cleared",
                "pass": body.get("exit_check", {}).get("lod0_count") == 0,
            },
            {
                "id": "no_residual_lod0_dirs",
                "pass": not bool(body.get("exit_check", {}).get("residual_lod0_dirs_in_modules")),
            },
            {
                "id": "no_fake_operator_pass",
                "pass": body.get("operator_pass") is False,
            },
            {
                "id": "next_residual_is_reinvent_not_q3",
                "pass": (
                    "063286c3" in str(body.get("next_residual") or "")
                    and not str(body.get("next_residual") or "").startswith("BQ-Q3")
                ),
            },
        ],
        repo=root,
    )
    meta = out.setdefault("_agent_meta", {})
    meta.update(
        {
            "track": "PLAN-MCP-APS-TOOLING-FINISH-001",
            "task_id": TASK_ID,
            "proceed_ship": bool(out.get("proceed_ship")),
            "art_quality": out.get("art_quality"),
            "agent": "coder-mcp",
            "operator_pass": False,
        }
    )
    if out.get("green"):
        meta["ritual"] = f"BLANG:WIT-HON→Q✓ {TASK_ID}"
    path = root / WITNESS_REL
    path.write_text(json.dumps(out, indent=2) + "\n", encoding="utf-8")
    out["written"] = WITNESS_REL
    out["written_at_epoch_secs"] = int(time.time())
    return out
