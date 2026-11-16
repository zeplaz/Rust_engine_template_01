"""PLAN-REF-BOOKS-UPGRADE-v3 X3 — veg species table APS brief + validate_report."""

from rust_engine_mcp.validators import run_validator
from rust_engine_mcp.validators.studio_dcc import validate_veg_species_path
from rust_engine_mcp.veg_species_table import brief_veg_species_table


def test_brief_veg_species_table_green() -> None:
    brief = brief_veg_species_table()
    assert brief["green"] is True
    assert brief["species_count"] >= 4
    assert "oak" in brief["species_ids"]
    assert brief["ref_books_dual_cite"]["new_rows"] == ["PCG-05"]


def test_validate_veg_species_report_passes() -> None:
    report = validate_veg_species_path(compression_level=3)
    assert report.status == "passed"
    assert report.validator == "veg_species"
    assert run_validator("veg_species").status == "passed"