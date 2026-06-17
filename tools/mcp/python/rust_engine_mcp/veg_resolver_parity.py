"""CDR-B-VEG-RESOLVER-PARITY-001 — catalog variant_key byte parity vs engine resolver."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.veg_catalog_loader import load_vegetation_variant_catalog

WITNESS_REL = "debug_runs/art_pipeline/veg_resolver_parity_live.json"
DOC_REL = "src/dev/veg_resolver_known_keys_v1.md"

# Byte-aligned with landscape_grammar_burn.rs + landscape_atlas_registry.rs
ENGINE_VEG_RESOLVER_KEYS: tuple[str, ...] = (
    "veg_clean_day",
    "veg_old_growth",
    "veg_damaged",
    "veg_regrowth_nuclei",
    "veg_regrowth_front",
    *(f"veg_burn_{i:02}" for i in range(8)),
)

ENGINE_TOPOLOGY_STAMP_KEYS: tuple[str, ...] = (
    "topology_patch",
    "topology_corridor",
    "topology_ring",
)


def engine_known_keys() -> dict[str, list[str]]:
    return {
        "veg_resolver": list(ENGINE_VEG_RESOLVER_KEYS),
        "topology_stamp": list(ENGINE_TOPOLOGY_STAMP_KEYS),
    }


def catalog_keys_by_prefix(catalog: dict[str, Any]) -> dict[str, list[str]]:
    veg: list[str] = []
    topology: list[str] = []
    for entry in catalog.get("entries") or []:
        if not isinstance(entry, dict):
            continue
        key = str(entry.get("variant_key") or "")
        if key.startswith("veg_"):
            veg.append(key)
        elif key.startswith("topology_"):
            topology.append(key)
    return {"veg": sorted(veg), "topology": sorted(topology)}


def check_resolver_catalog_parity(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    catalog = load_vegetation_variant_catalog(repo=root)
    by_prefix = catalog_keys_by_prefix(catalog)
    catalog_veg = set(by_prefix["veg"])
    engine_veg = set(ENGINE_VEG_RESOLVER_KEYS)
    missing_in_catalog = sorted(engine_veg - catalog_veg)
    extra_in_catalog = sorted(catalog_veg - engine_veg)
    stamp_missing = sorted(set(ENGINE_TOPOLOGY_STAMP_KEYS) - set(by_prefix["topology"]))
    green = not missing_in_catalog and not extra_in_catalog and not stamp_missing
    return {
        "slice_id": "CDR-B-VEG-RESOLVER-PARITY-001",
        "green": green,
        "engine_veg_keys": list(ENGINE_VEG_RESOLVER_KEYS),
        "engine_topology_stamp_keys": list(ENGINE_TOPOLOGY_STAMP_KEYS),
        "catalog_veg_count": len(catalog_veg),
        "catalog_topology_count": len(by_prefix["topology"]),
        "missing_in_catalog": missing_in_catalog,
        "extra_in_catalog": extra_in_catalog,
        "stamp_keys_missing_in_catalog": stamp_missing,
        "byte_parity": green,
        "charter": "src/dev/plan_veg_variant_key_naming_v1.md",
        "catalog_path": "assets/configs/landscape/_vegetation_variant_catalog.ron",
    }


def write_veg_resolver_known_keys_doc(*, repo: Path | None = None) -> Path:
    root = repo or repo_root()
    out = root / DOC_REL
    parity = check_resolver_catalog_parity(repo=root)
    lines = [
        "# veg_resolver_known_keys_v1 — VegetationExtractFrame authority",
        "",
        "| Field | Value |",
        "|:---|:---|",
        "| **Slice** | `CDR-B-VEG-RESOLVER-PARITY-001` |",
        "| **Engine** | `variant_key_for_burn_row` · `topology_kind_to_variant_key` |",
        "| **Catalog** | `assets/configs/landscape/_vegetation_variant_catalog.ron` |",
        f"| **Parity** | {'PASS' if parity.get('green') else 'FAIL'} |",
        "",
        "## Veg resolver keys (`veg_*`)",
        "",
        "Emitted by `variant_key_for_burn_row` in `src/systems/ecology/landscape_grammar_burn.rs`:",
        "",
    ]
    for key in ENGINE_VEG_RESOLVER_KEYS:
        lines.append(f"- `{key}`")
    lines.extend(
        [
            "",
            "## Topology stamp keys (`topology_*`)",
            "",
            "Emitted by `topology_kind_to_variant_key` in `src/systems/ecology/landscape_atlas_registry.rs`:",
            "",
        ]
    )
    for key in ENGINE_TOPOLOGY_STAMP_KEYS:
        lines.append(f"- `{key}`")
    lines.extend(
        [
            "",
            "## Expanded atlas topology rows",
            "",
            "LG-5 expanded cells (`topology_*_scar`, `topology_*_burn_*`, regrowth suffixes) are catalog + tile_batch authority — not burn resolver output.",
            "",
            f"Catalog topology row count: **{parity.get('catalog_topology_count')}**.",
            "",
            "## Parity rule",
            "",
            "Authored `veg_*` catalog keys must match engine resolver keys **byte-for-byte** (no extras, no omissions).",
            "",
        ]
    )
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return out


def refresh_veg_resolver_parity_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    doc = write_veg_resolver_known_keys_doc(repo=root)
    body = check_resolver_catalog_parity(repo=root)
    body["deliverable"] = DOC_REL
    body["deliverable_written"] = str(doc.relative_to(root)).replace("\\", "/")
    body["_agent_meta"] = {
        "schema": "veg_resolver_parity_live_v1",
        "written_at_epoch_secs": int(time.time()),
        "profile": "CDR_B_VEG_RESOLVER_PARITY",
        "source_system": "veg_resolver_parity",
        "relative_path": WITNESS_REL,
        "ritual": "BLANG:Q✓ CDR-B-VEG-RESOLVER-PARITY-001" if body.get("green") else None,
        "agent": "coder-mcp",
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = str(out.relative_to(root)).replace("\\", "/")
    return body
