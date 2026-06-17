"""MCP-LANDSCAPE-BROWSE-STUB-001 — preset index + validate-report landscape_grammar wrapper."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from rust_engine_mcp.landscape_grammar_presets import INDEX_REL, PRESETS_DIR_REL, iter_ship_preset_jsons
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators.landscape_grammar import validate_landscape_grammar_path
from rust_engine_mcp.validators.report import ValidationReport

INDEX_PATH = INDEX_REL
PRESETS_DIR = PRESETS_DIR_REL


def load_preset_index(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    path = root / INDEX_REL
    if not path.is_file():
        return {}
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {}
    return data if isinstance(data, dict) else {}


def list_landscape_presets(*, repo: Path | None = None) -> dict[str, Any]:
    """Read-only browse — ship + topology ids from _preset_index.json."""
    root = repo or repo_root()
    index = load_preset_index(repo=root)
    ship = [str(x) for x in (index.get("ship_presets") or []) if x]
    topology_block = index.get("topology_presets") or []
    topology_ids = [
        str(x.get("id") if isinstance(x, dict) else x)
        for x in topology_block
        if (isinstance(x, dict) and x.get("id")) or (not isinstance(x, dict) and x)
    ]
    on_disk = sorted(p.stem for p in iter_ship_preset_jsons(root / PRESETS_DIR)) if (root / PRESETS_DIR).is_dir() else []
    return {
        "ok": True,
        "index_path": INDEX_REL,
        "ship_presets": ship,
        "topology_presets": topology_ids,
        "ship_count": len(ship),
        "topology_count": len(topology_ids),
        "preset_files_on_disk": on_disk,
        "index_aligned": not [pid for pid in ship if pid not in on_disk],
    }


def preset_path(preset_id: str, *, repo: Path | None = None) -> Path:
    root = repo or repo_root()
    return root / PRESETS_DIR / f"{preset_id}.json"


def validate_landscape_preset(
    preset_id: str,
    *,
    compression_level: int = 3,
    repo: Path | None = None,
) -> ValidationReport:
    path = preset_path(preset_id, repo=repo)
    if not path.is_file():
        from rust_engine_mcp.validators.report import ValidationIssue, ValidationReport

        return ValidationReport(
            validator="landscape_grammar",
            status="failed",
            compression_level=compression_level,
            summary=f"preset not found: {preset_id}",
            error_count=1,
            errors=[
                ValidationIssue(
                    kind="MissingPreset",
                    severity="error",
                    file=str(path),
                    hint=f"No preset JSON for {preset_id!r}",
                )
            ],
        )
    return validate_landscape_grammar_path(path, compression_level=compression_level)


def preset_summary(preset_id: str, *, repo: Path | None = None) -> dict[str, Any]:
    """Compressed topology summary for APS browse (read-only)."""
    path = preset_path(preset_id, repo=repo)
    if not path.is_file():
        return {"ok": False, "preset_id": preset_id, "error": "missing"}
    doc = json.loads(path.read_text(encoding="utf-8"))
    graph = doc.get("topology_graph") if isinstance(doc.get("topology_graph"), dict) else {}
    nodes = graph.get("nodes") or []
    node_kinds = sorted({str(n.get("kind") or n.get("type") or "") for n in nodes if isinstance(n, dict)})
    report = validate_landscape_preset(preset_id, repo=repo)
    return {
        "ok": True,
        "preset_id": preset_id,
        "path": str(path.relative_to(repo or repo_root())).replace("\\", "/"),
        "topology_node_count": len(nodes),
        "topology_kinds": node_kinds[:8],
        "validate_status": report.status,
        "validate_summary": report.summary,
    }
