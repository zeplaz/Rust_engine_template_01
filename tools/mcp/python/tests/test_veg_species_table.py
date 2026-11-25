"""PLAN-REF-BOOKS-UPGRADE-v3 X3 — veg species table APS brief + validate_report."""

from pathlib import Path

from rust_engine_mcp.validators import run_validator
from rust_engine_mcp.validators.studio_dcc import validate_veg_species_path
from rust_engine_mcp.veg_species_table import brief_veg_species_table


def test_brief_veg_species_table_green() -> None:
    brief = brief_veg_species_table()
    assert brief["green"] is True
    assert brief["species_count"] >= 4
    assert "oak" in brief["species_ids"]
    assert "ref_books_dual_cite" not in brief


def test_validate_veg_species_report_passes() -> None:
    report = validate_veg_species_path(compression_level=3)
    assert report.status == "passed"
    assert report.validator == "veg_species"
    assert run_validator("veg_species").status == "passed"


def test_validate_veg_species_path_honors_bad_file(tmp_path: Path) -> None:
    """CQ-11: path argument is validated, not silently swapped for the default RON."""
    bad = tmp_path / "broken_veg.ron"
    bad.write_text('VegSpeciesTable(\n  seed: 1,\n  species: [\n    (id: "x", profile: NotAProfile, tiers: 9),\n  ],\n)\n', encoding="utf-8")
    report = validate_veg_species_path(bad, compression_level=3)
    assert report.status == "failed"
    assert any("profile" in (i.hint or "").lower() or "tiers" in (i.hint or "").lower() for i in report.errors)