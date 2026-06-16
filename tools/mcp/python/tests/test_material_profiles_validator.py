"""APS-MAT-008 material_profiles validator tests."""

from __future__ import annotations

import json
from pathlib import Path

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators.material_profiles import (
    validate_assembly_material_profiles,
    write_material_validation_witness,
)

WAREHOUSE = (
    repo_root()
    / "tools/mcp/schemas/examples/assembly_snapshot_warehouse_industrial_west_production_v1.json"
)


def test_warehouse_snapshot_passes_material_gate() -> None:
    snap = json.loads(WAREHOUSE.read_text(encoding="utf-8"))
    report = validate_assembly_material_profiles(snap, snapshot_path=str(WAREHOUSE), ship=True)
    assert report.status == "passed", [e.hint for e in report.errors]


def test_missing_material_profile_fails() -> None:
    snap = json.loads(WAREHOUSE.read_text(encoding="utf-8"))
    placements = list(snap.get("module_placements") or [])
    if placements:
        placements[0] = dict(placements[0])
        placements[0].pop("material_profile", None)
    snap["module_placements"] = placements
    report = validate_assembly_material_profiles(snap, ship=True)
    assert report.status != "passed"
    assert any(e.kind == "MissingMaterialProfile" for e in report.errors)


def test_material_validation_witness_written() -> None:
    snap = json.loads(WAREHOUSE.read_text(encoding="utf-8"))
    report = validate_assembly_material_profiles(snap, snapshot_path=str(WAREHOUSE), ship=True)
    path = write_material_validation_witness(report, snapshot_path=str(WAREHOUSE))
    assert path.is_file()
    body = json.loads(path.read_text(encoding="utf-8"))
    assert body.get("gate_id") == "APS-MAT-008"
    assert body.get("ok") == (report.status == "passed")
