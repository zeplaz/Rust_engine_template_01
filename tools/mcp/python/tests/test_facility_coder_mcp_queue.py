"""CMCP-SITE-ZONE-VALIDATE-001 + facility needs strip tests."""

from __future__ import annotations

from pathlib import Path

from rust_engine_mcp import assembly
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


def test_facility_needs_strip_line4_g3() -> None:
    from art_pipeline_suite.facility_needs_strip import FacilityNeedsStrip, refresh_facility_needs_witness

    import tkinter as tk

    root = tk.Tk()
    root.withdraw()
    strip = FacilityNeedsStrip(root)
    strip.set_grammar_tier("G3")
    strip.refresh(archetype_id="FactoryCluster", lane="buildings")
    line4 = strip._line4_var.get()  # noqa: SLF001
    assert "site:" in line4
    assert "storage" in line4
    root.destroy()
    witness = refresh_facility_needs_witness()
    assert witness["green"] is True


def test_site_preview_panel_witness_green() -> None:
    from art_pipeline_suite.site_preview_panel import SiteLayoutPreviewSection, refresh_site_preview_witness

    import tkinter as tk

    root = tk.Tk()
    root.withdraw()
    section = SiteLayoutPreviewSection(root)
    section.set_grammar_tier("G2")
    section.refresh(archetype_id="RailEdge", lane="buildings")
    root.update_idletasks()
    root.destroy()
    body = refresh_site_preview_witness()
    assert body["green"] is True
    assert body["site_preview_visible"] is True


def test_explain_module_resolve_lod0_fallback_label() -> None:
    body = assembly.explain_module_resolve(
        "wall_brick_1u",
        style_pack_id="style_victorian",
        source_tier="production",
    )
    assert body["ok"] is True
    assert body["reason"] in ("ok_production", "lod0_fallback", "ok_lod0")
    assert body["label"]


def test_grammar_eval_sweep_process_histogram() -> None:
    from rust_engine_mcp import grammar_build_set

    body = grammar_build_set.grammar_eval_sweep()
    hist = body.get("process_histogram") or {}
    assert hist.get("power_tier", {}).get("light", 0) >= 1
    zones = hist.get("zone_coverage") or {}
    assert zones.get("primary", 0) >= 1
    assert "rail" in zones or "utility" in zones


def test_grammar_sweep_process_witness_green() -> None:
    from rust_engine_mcp import grammar_build_set

    body = grammar_build_set.write_grammar_sweep_process_witness()
    assert body.get("green") is True
    assert body.get("witness_honesty", {}).get("status") == "passed"
    hist = body.get("process_histogram") or {}
    assert hist.get("zone_coverage")


def test_veg_catalog_burn_rows_witness_green() -> None:
    from rust_engine_mcp.veg_catalog_loader import refresh_veg_catalog_burn_rows_witness

    body = refresh_veg_catalog_burn_rows_witness()
    assert body.get("green") is True
    assert int(body.get("burn_rows") or 0) >= int(body.get("burn_frame_count") or 8)
