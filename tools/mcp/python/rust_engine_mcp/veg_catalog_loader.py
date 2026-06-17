"""Load vegetation variant catalog for APS States tab + validators."""

from __future__ import annotations

import re
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.vegetation_variant_catalog import (
    CATALOG_RON_REL,
    build_catalog_body,
    validate_catalog_body,
)

_VARIANT_KEY_RE = re.compile(r'variant_key:\s*"([^"]+)"')


def load_vegetation_variant_catalog(*, repo: Path | None = None) -> dict[str, Any]:
    """Return catalog body — prefer on-disk RON when byte-aligned with build_catalog_body."""
    root = repo or repo_root()
    body = build_catalog_body(repo=root)
    ron_path = root / CATALOG_RON_REL
    if ron_path.is_file():
        disk_keys = _VARIANT_KEY_RE.findall(ron_path.read_text(encoding="utf-8"))
        built_keys = [str(e.get("variant_key")) for e in body.get("entries") or []]
        if sorted(disk_keys) == sorted(built_keys):
            return body
    return body


def catalog_validator_report(*, repo: Path | None = None) -> dict[str, Any]:
    body = load_vegetation_variant_catalog(repo=repo)
    try:
        validation = validate_catalog_body(body, repo=repo)
        green = validation.get("status") == "passed"
    except Exception as exc:  # noqa: BLE001
        return {
            "green": False,
            "status": "failed",
            "error": str(exc),
            "catalog_path": CATALOG_RON_REL,
        }
    return {
        "green": green,
        "catalog_path": CATALOG_RON_REL,
        **validation,
    }


def _entry_axis(entry: dict[str, Any]) -> str:
    key = str(entry.get("variant_key") or "")
    resolver = entry.get("resolver") if isinstance(entry.get("resolver"), dict) else {}
    kind = str(resolver.get("kind") or "")
    if key.startswith("veg_burn_"):
        return "burn"
    if kind == "active_burn_frame":
        return "burn"
    if kind in ("succession_stage", "regrowth_macro", "default") and key.startswith("veg_"):
        return "succession"
    if key.startswith("topology_"):
        return "topology_state"
    return "other"


def state_axis_rows(*, repo: Path | None = None) -> list[dict[str, Any]]:
    """Rows for Landscape States tab — succession + burn + topology matrix."""
    catalog = load_vegetation_variant_catalog(repo=repo)
    axes = catalog.get("axes") if isinstance(catalog.get("axes"), dict) else {}
    burn_count = int(axes.get("burn_frame_count") or 8)
    rows: list[dict[str, Any]] = []
    for entry in catalog.get("entries") or []:
        if not isinstance(entry, dict):
            continue
        key = str(entry.get("variant_key") or "")
        resolver = entry.get("resolver") if isinstance(entry.get("resolver"), dict) else {}
        rows.append(
            {
                "variant_key": key,
                "axis": _entry_axis(entry),
                "resolver_kind": str(resolver.get("kind") or ""),
                "sim_tags": list(entry.get("sim_tags") or []),
            }
        )
    rows.sort(key=lambda r: (r["axis"], r["variant_key"]))
    return rows


def burn_variant_keys(*, repo: Path | None = None) -> list[str]:
    return [r["variant_key"] for r in state_axis_rows(repo=repo) if r["axis"] == "burn"]


def catalog_axis_summary(*, repo: Path | None = None) -> dict[str, Any]:
    catalog = load_vegetation_variant_catalog(repo=repo)
    axes = catalog.get("axes") if isinstance(catalog.get("axes"), dict) else {}
    rows = state_axis_rows(repo=repo)
    return {
        "burn_frame_count": int(axes.get("burn_frame_count") or 8),
        "succession_stages": list(axes.get("succession_stages") or []),
        "regrowth_macro_phases": list(axes.get("regrowth_macro_phases") or []),
        "entry_count": len(rows),
        "burn_rows": sum(1 for r in rows if r["axis"] == "burn"),
        "succession_rows": sum(1 for r in rows if r["axis"] == "succession"),
        "topology_rows": sum(1 for r in rows if r["axis"] == "topology_state"),
    }
