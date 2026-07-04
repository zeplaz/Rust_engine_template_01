"""APSR-A1-S2-001 — AssemblyService + shadow _snapshot removal witness."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.suite_state_mutation_inventory import (
    scan_suite_state_mutations,
    suite_state_mutation_inventory,
    sync_mutation_allowlist_from_scan,
)

TASK_ID = "APSR-A1-S2-ASM-001"
WITNESS_REL = "debug_runs/apsr_a1_s2_001_live.json"
ASSEMBLY_FIELDS = frozenset(
    {
        "assembly_id",
        "assembly_snapshot_path",
        "assembly_snapshot_data",
        "module_ids_in_assembly",
        "assembly_p0_passed",
    }
)


def _assembly_direct_write_sites(*, root: Path | None = None) -> list[dict[str, Any]]:
    return [
        {"id": site.site_id, "file": site.file, "line": site.line, "field": site.field}
        for site in scan_suite_state_mutations(root=root)
        if site.field in ASSEMBLY_FIELDS
    ]


def write_apsr_a1_s2_witness(*, repo: Path | None = None, sync_allowlist: bool = True) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    root = repo or repo_root()
    if sync_allowlist:
        sync_mutation_allowlist_from_scan()
    inventory = suite_state_mutation_inventory()
    direct_assembly = _assembly_direct_write_sites()
    svc_path = root / "tools/mcp/art_pipeline_suite/aps_services/assembly_service.py"
    green = (
        inventory.get("green") is True
        and direct_assembly == []
        and svc_path.is_file()
    )
    body: dict[str, Any] = {
        "task_id": TASK_ID,
        "gate": TASK_ID,
        "green": green,
        "mutation_inventory_green": inventory.get("green"),
        "live_mutation_count": inventory.get("live_count"),
        "assembly_direct_write_sites": direct_assembly,
        "assembly_single_owner": direct_assembly == [],
        "assembly_service_module": "tools/mcp/art_pipeline_suite/aps_services/assembly_service.py",
        "enforced_fields": sorted(ASSEMBLY_FIELDS),
        "plan_ref": "src/dev/plan_aps_refactor_v1.md#APSR-S2",
    }
    return write_aps_live_witness(
        body,
        WITNESS_REL,
        schema="apsr_a1_s2_live_v1",
        profile="APSR_A1_S2",
        source_system="apsr_a1_s2",
        ritual=f"BLANG:WIT-HON {TASK_ID}" if green else None,
        repo=root,
    )
