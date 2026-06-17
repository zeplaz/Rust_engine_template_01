"""APS-EVO-E5-EXTRACT-PARITY-001 — authored catalog keys vs extract/resolver authority."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.veg_catalog_loader import load_vegetation_variant_catalog
from rust_engine_mcp.veg_resolver_parity import (
    ENGINE_VEG_RESOLVER_KEYS,
    check_resolver_catalog_parity,
)

APS_VEG_EXTRACT_PARITY_WITNESS = "debug_runs/aps_veg_extract_parity_live.json"
EXTRACT_WITNESS_REL = "debug_runs/landscape_grammar_extract_live.json"
ENGINE_AUTHORITY = "vegetation_extract_frame"
DESIGN_REF = "src/dev/plan_aps_veg_parity_engine_authority_v1.md"

ENGINE_READ_PATH = (
    "Runtime: SuccessionState + ActiveBurn → VegetationExtractFrame::BuildProfiles "
    "(rows[].variant_key) → landscape_chunk_atlas_stamp / LG-5 atlas index. "
    "Authored keys live in _vegetation_variant_catalog.ron — not RepresentationResult "
    "or Blender viewport."
)


def _ensure_aps_suite_path() -> None:
    suite_root = repo_root() / "tools/mcp"
    if str(suite_root) not in sys.path:
        sys.path.insert(0, str(suite_root))


def authored_veg_keys(*, repo: Path | None = None) -> list[str]:
    catalog = load_vegetation_variant_catalog(repo=repo)
    keys: list[str] = []
    for entry in catalog.get("entries") or []:
        if not isinstance(entry, dict):
            continue
        key = str(entry.get("variant_key") or "")
        if key.startswith("veg_"):
            keys.append(key)
    return sorted(keys)


def load_extract_witness(*, repo: Path | None = None) -> dict[str, Any] | None:
    root = repo or repo_root()
    path = root / EXTRACT_WITNESS_REL
    if not path.is_file():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    return data if isinstance(data, dict) else None


def extract_sample_keys(extract: dict[str, Any] | None) -> list[str]:
    if not extract:
        return []
    raw = extract.get("sample_variant_keys") or extract.get("resolver_keys") or []
    if not isinstance(raw, list):
        return []
    return sorted({str(k) for k in raw if k})


def _verify_panel_wired() -> bool:
    _ensure_aps_suite_path()
    try:
        from art_pipeline_suite.landscape_extract_parity_panel import LandscapeExtractParityPanel
    except ImportError:
        return False
    return hasattr(LandscapeExtractParityPanel, "refresh_parity") and bool(ENGINE_READ_PATH)


def check_veg_extract_parity(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    authored = authored_veg_keys(repo=root)
    resolver_keys = list(ENGINE_VEG_RESOLVER_KEYS)
    resolver_set = set(resolver_keys)
    authored_set = set(authored)
    missing_from_resolver = sorted(authored_set - resolver_set)
    extra_authored_vs_resolver = sorted(resolver_set - authored_set)
    subset_ok = not missing_from_resolver and not extra_authored_vs_resolver

    resolver_parity = check_resolver_catalog_parity(repo=root)
    resolver_parity_ok = bool(resolver_parity.get("green"))

    extract = load_extract_witness(repo=root)
    extract_witness_present = extract is not None
    extract_witness_green = bool(extract and extract.get("green") is True)
    extract_keys = extract_sample_keys(extract)

    panel_wired = _verify_panel_wired()
    parity_green = bool(
        subset_ok and resolver_parity_ok and extract_witness_green and panel_wired
    )
    return {
        "slice_id": "APS-EVO-E5-EXTRACT-PARITY-001",
        "engine_authority": ENGINE_AUTHORITY,
        "engine_read_path": ENGINE_READ_PATH,
        "authored_keys": authored,
        "resolver_keys": resolver_keys,
        "authored_count": len(authored),
        "resolver_count": len(resolver_keys),
        "missing_from_resolver": missing_from_resolver,
        "extra_authored_vs_resolver": extra_authored_vs_resolver,
        "subset_ok": subset_ok,
        "resolver_parity_ok": resolver_parity_ok,
        "extract_witness": EXTRACT_WITNESS_REL,
        "extract_witness_present": extract_witness_present,
        "extract_witness_green": extract_witness_green,
        "extract_sample_keys": extract_keys,
        "extract_row_count": int((extract or {}).get("row_count") or 0),
        "panel_wired": panel_wired,
        "parity_green": parity_green,
        "catalog_path": resolver_parity.get("catalog_path"),
        "design_ref": DESIGN_REF,
        "cdr_b_witness": "debug_runs/art_pipeline/veg_resolver_parity_live.json",
    }


def refresh_aps_veg_extract_parity_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    check = check_veg_extract_parity(repo=root)
    green = bool(check.get("parity_green"))
    body: dict[str, Any] = {
        "gate": "APS-EVO-E5-EXTRACT-PARITY-001",
        "program_id": "APS-E5",
        "green": green,
        **check,
    }
    return write_aps_live_witness(
        body,
        APS_VEG_EXTRACT_PARITY_WITNESS,
        schema="aps_veg_extract_parity_live_v1",
        profile="APS_E5_VEG_EXTRACT_PARITY",
        source_system="aps_veg_extract_parity",
        ritual="BLANG:WIT-HON APS-EVO-E5-EXTRACT-PARITY-001" if green else None,
        exit_predicate_must=[
            {"path": "parity_green", "eq": True},
            {"path": "subset_ok", "eq": True},
            {"path": "resolver_parity_ok", "eq": True},
            {"path": "extract_witness_green", "eq": True},
            {"path": "panel_wired", "eq": True},
        ],
        repo=root,
    )
