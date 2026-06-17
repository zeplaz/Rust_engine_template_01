"""MCP-LG-VALID-PRESET-001 — batch landscape_grammar_v0 preset validation + witness."""

from __future__ import annotations

import json
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators.landscape_grammar import validate_landscape_grammar_path
from rust_engine_mcp.validators.report import ValidationIssue, ValidationReport

PRESETS_DIR_REL = "assets/configs/landscape/presets"
INDEX_REL = "assets/configs/landscape/_preset_index.json"
SCHEMA_REL = "tools/mcp/schemas/landscape_grammar_v0.schema.json"
SIGN_WITNESS_REL = "debug_runs/mcp_landscape_grammar_sign_live.json"
BATCH_WITNESS_REL = "debug_runs/mcp_landscape_grammar_preset_batch_live.json"


def _load_index(root: Path) -> dict[str, Any]:
    path = root / INDEX_REL
    if not path.is_file():
        return {}
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {}
    return data if isinstance(data, dict) else {}


def landscape_grammar_presets_batch(*, repo: Path | None = None) -> dict[str, Any]:
    """Validate every ship preset JSON under assets/configs/landscape/presets/."""
    root = repo or repo_root()
    presets_dir = root / PRESETS_DIR_REL
    index = _load_index(root)
    ship_ids = [str(x) for x in (index.get("ship_presets") or []) if x]
    topology_count = len(index.get("topology_presets") or [])
    preset_results: list[dict[str, Any]] = []
    for path in sorted(presets_dir.glob("*.json")):
        report = validate_landscape_grammar_path(path, compression_level=3)
        preset_results.append(
            {
                "file": path.name,
                "preset_id": path.stem,
                "status": report.status,
                "error_count": report.error_count,
            }
        )

    on_disk = {p.stem for p in presets_dir.glob("*.json")}
    missing_ship = [pid for pid in ship_ids if pid not in on_disk]
    orphan_files = sorted(on_disk - set(ship_ids))
    failed = [r for r in preset_results if r["status"] != "passed"]
    green = (
        len(failed) == 0
        and not missing_ship
        and len(preset_results) >= len(ship_ids)
        and topology_count == 30
        and len(ship_ids) >= 5
    )
    return {
        "schema": "landscape_grammar_presets_batch_v1",
        "green": green,
        "presets_dir": PRESETS_DIR_REL,
        "schema_path": SCHEMA_REL,
        "preset_validation": {
            "total": len(preset_results),
            "passed": sum(1 for r in preset_results if r["status"] == "passed"),
            "failed": len(failed),
            "results": preset_results,
        },
        "index": {
            "path": INDEX_REL,
            "topology_preset_count": topology_count,
            "ship_preset_count": len(ship_ids),
            "missing_ship_files": missing_ship,
            "orphan_preset_files": orphan_files,
        },
        "errors": [
            *(f"preset failed: {r['file']}" for r in failed),
            *(f"ship preset missing on disk: {pid}" for pid in missing_ship),
        ],
    }


def validate_landscape_grammar_presets(*, compression_level: int = 3) -> ValidationReport:
    body = landscape_grammar_presets_batch()
    issues = [
        ValidationIssue(kind="landscape_grammar_presets", severity="error", hint=str(msg))
        for msg in (body.get("errors") or [])
    ]
    green = bool(body.get("green"))
    passed = (body.get("preset_validation") or {}).get("passed", 0)
    total = (body.get("preset_validation") or {}).get("total", 0)
    return ValidationReport(
        validator="mcp_schema",
        status="passed" if green else "failed",
        compression_level=compression_level,
        summary=f"landscape_grammar_presets: {passed}/{total} passed",
        error_count=len(issues),
        errors=issues,
    ).compress(compression_level)


def write_landscape_grammar_presets_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    batch = landscape_grammar_presets_batch(repo=root)
    body: dict[str, Any] = {
        "gate": "MCP-LG-VALID-PRESET-001",
        "green": batch.get("green"),
        "validator": "validate-report landscape_grammar_presets",
        "batch": batch,
        "_agent_meta": {
            "schema": "mcp_landscape_grammar_preset_batch_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "MCP_LG_VALID_PRESET",
            "source_system": "landscape_grammar_presets",
            "relative_path": BATCH_WITNESS_REL,
        },
    }
    out = root / BATCH_WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    body["written"] = BATCH_WITNESS_REL
    return body


def refresh_mcp_landscape_grammar_sign_witness(*, repo: Path | None = None) -> dict[str, Any]:
    """Refresh SIGN witness (MCP-LG-SIGN-004) from batch results."""
    root = repo or repo_root()
    batch = landscape_grammar_presets_batch(repo=root)
    index = batch.get("index") if isinstance(batch.get("index"), dict) else {}
    index_doc = _load_index(root)
    green = bool(batch.get("green")) and index_doc.get("signed") is True
    body: dict[str, Any] = {
        "gate": "MCP-LANDSCAPE-GRAMMAR-SIGN-001",
        "signed": green,
        "signed_at": datetime.now(timezone.utc).isoformat(),
        "schema": SCHEMA_REL,
        "validator": "validate-report landscape_grammar",
        "preset_validation": batch.get("preset_validation"),
        "topology_preset_count": index.get("topology_preset_count"),
        "ship_preset_count": index.get("ship_preset_count"),
        "green": green,
    }
    out = root / SIGN_WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    body["written"] = SIGN_WITNESS_REL
    return body


def run_landscape_grammar_post_build_hook(*, repo: Path | None = None) -> bool:
    """post_build hook — batch witness + sign witness refresh."""
    root = repo or repo_root()
    batch_body = write_landscape_grammar_presets_witness(repo=root)
    sign_body = refresh_mcp_landscape_grammar_sign_witness(repo=root)
    return bool(batch_body.get("green")) and bool(sign_body.get("green"))
