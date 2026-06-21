"""CMCP-SITE-ZONE-VALIDATE-001 + facility needs strip tests."""

from __future__ import annotations

from pathlib import Path

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators.site_zone_grid import (
    validate_site_zone_grid_path,
    write_site_zone_validate_witness,
)


def test_site_zone_grid_mixer_pilot_passes() -> None:
    path = repo_root() / "assets/configs/buildings/pilots/concrete_mixer_plant_site_v0.json"
    report = validate_site_zone_grid_path(path)
    assert report.status in ("passed", "warning")
    assert report.error_count == 0


def test_site_zone_validate_witness_green() -> None:
    body = write_site_zone_validate_witness()
    assert body.get("pilot_count", 0) >= 7
    assert body.get("green") is True


def test_facility_needs_strip_factory_cluster() -> None:
    from art_pipeline_suite.facility_needs_strip import FacilityNeedsStrip

    import tkinter as tk

    root = tk.Tk()
    root.withdraw()
    strip = FacilityNeedsStrip(root)
    strip.set_grammar_tier("G2")
    strip.refresh(archetype_id="FactoryCluster", lane="buildings")
    assert strip._line1_var.get()  # noqa: SLF001
    assert "Concrete" in strip._line2_var.get() or "Cement" in strip._line2_var.get()  # noqa: SLF001
    root.destroy()


def test_grammar_eval_sweep_process_histogram() -> None:
    from rust_engine_mcp import grammar_build_set

    body = grammar_build_set.grammar_eval_sweep()
    hist = body.get("process_histogram") or {}
    assert hist.get("power_tier", {}).get("light", 0) >= 1


def test_veg_catalog_burn_rows_witness_green() -> None:
    from rust_engine_mcp.veg_catalog_loader import refresh_veg_catalog_burn_rows_witness

    body = refresh_veg_catalog_burn_rows_witness()
    assert body.get("green") is True
    assert int(body.get("burn_rows") or 0) >= int(body.get("burn_frame_count") or 8)
