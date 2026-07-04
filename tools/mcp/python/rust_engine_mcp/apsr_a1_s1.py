"""APSR-A1-S1-001 — EventBus + SuiteStateWriter + AtlasService witness."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.suite_state_mutation_inventory import (
    scan_suite_state_mutations,
    suite_state_mutation_inventory,
    sync_mutation_allowlist_from_scan,
)

TASK_ID = "APSR-A1-S1-001"
WITNESS_REL = "debug_runs/apsr_a1_s1_001_live.json"
ATLAS_FIELDS = frozenset({"atlas_folder", "tile_batch_path"})


def _atlas_direct_write_sites(*, root: Path | None = None) -> list[dict[str, Any]]:
    return [
        {"id": site.site_id, "file": site.file, "line": site.line, "field": site.field}
        for site in scan_suite_state_mutations(root=root)
        if site.field in ATLAS_FIELDS
    ]


def write_apsr_a1_s1_witness(*, repo: Path | None = None, sync_allowlist: bool = True) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    root = repo or repo_root()
    if sync_allowlist:
        sync_mutation_allowlist_from_scan()
    inventory = suite_state_mutation_inventory()
    direct_atlas = _atlas_direct_write_sites()
    bus_path = root / "tools/mcp/art_pipeline_suite/aps_event_bus.py"
    writer_path = root / "tools/mcp/art_pipeline_suite/aps_state_writer.py"
    atlas_svc_path = root / "tools/mcp/art_pipeline_suite/aps_services/atlas_service.py"
    green = (
        inventory.get("green") is True
        and direct_atlas == []
        and bus_path.is_file()
        and writer_path.is_file()
        and atlas_svc_path.is_file()
    )
    body: dict[str, Any] = {
        "task_id": TASK_ID,
        "gate": TASK_ID,
        "green": green,
        "mutation_inventory_green": inventory.get("green"),
        "live_mutation_count": inventory.get("live_count"),
        "atlas_direct_write_sites": direct_atlas,
        "atlas_single_owner": direct_atlas == [],
        "event_bus_module": "tools/mcp/art_pipeline_suite/aps_event_bus.py",
        "state_writer_module": "tools/mcp/art_pipeline_suite/aps_state_writer.py",
        "atlas_service_module": "tools/mcp/art_pipeline_suite/aps_services/atlas_service.py",
        "enforced_fields": sorted(ATLAS_FIELDS),
        "plan_ref": "src/dev/plan_aps_refactor_v1.md#APSR-S1",
    }
    return write_aps_live_witness(
        body,
        WITNESS_REL,
        schema="apsr_a1_s1_live_v1",
        profile="APSR_A1_S1",
        source_system="apsr_a1_s1",
        ritual=f"BLANG:WIT-HON {TASK_ID}" if green else None,
        repo=root,
    )
