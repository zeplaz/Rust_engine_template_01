"""Deterministic terrain-authority honesty lint (RPC-1-B / F2–F3).

Scans fixed paths for known dishonest labels — no LLM judgment.
Agents call this in L0 CHEAP before renaming or baking work.
"""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

WITNESS_PATH = "debug_runs/terrain_honesty_lint_live.json"

DOC_LIES: list[tuple[str, str, str]] = [
    (
        "src/dev/gpu_todos_v1.md",
        "terrain_source: gpu_atlas",
        "minimap terrain_source is world_raster until real GPU bake ships (RPC-1)",
    ),
    (
        "src/dev/visual_test_runbook_v1.md",
        "gpu_atlas",
        "runbook expects gpu_atlas; live label is world_raster (see minimap_terrain_source_label)",
    ),
]


def _read(rel: str) -> str | None:
    path = repo_root() / rel
    if not path.is_file():
        return None
    return path.read_text(encoding="utf-8")


def _authority_helpers(text: str) -> dict[str, Any]:
    sprite_includes_instanced = bool(
        re.search(
            r"fn uses_gpu_sprite_display[\s\S]{0,400}?GpuInstancedAtlas",
            text,
        )
    )
    return {
        "uses_gpu_sprite_display_includes_GpuInstancedAtlas": sprite_includes_instanced,
        "has_GpuTilemap_variant": "GpuTilemap" in text,
        "has_GpuInstancedAtlas_variant": "GpuInstancedAtlas" in text,
    }


def _gputilemap_constructed() -> list[str]:
    root = repo_root()
    hits: list[str] = []
    for path in (root / "src").rglob("*.rs"):
        rel = path.relative_to(root).as_posix()
        if rel.endswith("terrain_render_authority.rs"):
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except OSError:
            continue
        if "GpuTilemap" not in text:
            continue
        for i, line in enumerate(text.splitlines(), 1):
            if "GpuTilemap" not in line:
                continue
            stripped = line.strip()
            if stripped.startswith("//") or stripped.startswith("*"):
                continue
            if "matches!" in stripped or "Self::GpuTilemap" in stripped:
                continue
            if "TerrainRenderAuthority::GpuTilemap" in stripped and (
                "=" in stripped or "return" in stripped
            ):
                hits.append(f"{rel}:{i}")
    return hits[:20]


def _minimap_label_honest(text: str | None) -> tuple[bool, str]:
    if not text:
        return False, "minimap pass.rs missing"
    if 'return "gpu_atlas"' in text or '"gpu_atlas"' in text:
        return False, "gpu_atlas"
    has_world = '"world_raster"' in text
    has_gpu_bake = '"gpu_bake"' in text
    if has_world and has_gpu_bake:
        return True, "world_raster|gpu_bake"
    if has_world:
        return True, "world_raster"
    return False, "world_raster label not found"


def _issue(
    *,
    kind: str,
    severity: str,
    file: str,
    hint: str,
    line: int = 0,
) -> dict[str, Any]:
    d: dict[str, Any] = {
        "kind": kind,
        "severity": severity,
        "file": file,
        "hint": hint,
    }
    if line:
        d["line"] = line
    return d


def terrain_honesty_lint(*, write_witness: bool = True) -> dict[str, Any]:
    """Return ValidationReport-shaped dict + honesty facts (deterministic)."""
    issues: list[dict[str, Any]] = []
    facts: dict[str, Any] = {}

    auth_text = _read("src/render/core/terrain_render_authority.rs")
    if auth_text is None:
        issues.append(
            _issue(
                kind="terrain_authority_missing",
                severity="error",
                file="src/render/core/terrain_render_authority.rs",
                hint="file missing",
            )
        )
    else:
        helpers = _authority_helpers(auth_text)
        facts["authority"] = helpers
        if helpers["uses_gpu_sprite_display_includes_GpuInstancedAtlas"]:
            issues.append(
                _issue(
                    kind="dishonest_variant_name",
                    severity="warning",
                    file="src/render/core/terrain_render_authority.rs",
                    hint=(
                        "GpuInstancedAtlas covered by uses_gpu_sprite_display() — "
                        "RPC-1-B rename toward TerrainSourceMode::{CpuRaster,GpuBake}"
                    ),
                )
            )

    pass_text = _read("src/render/minimap_compositor/pass.rs")
    ok_label, label = _minimap_label_honest(pass_text)
    facts["minimap_terrain_source"] = label
    if not ok_label:
        issues.append(
            _issue(
                kind="minimap_label_drift",
                severity="error",
                file="src/render/minimap_compositor/pass.rs",
                hint=f"unexpected label: {label}",
            )
        )

    gputile_hits = _gputilemap_constructed()
    facts["GpuTilemap_construction_sites"] = gputile_hits
    if not gputile_hits:
        issues.append(
            _issue(
                kind="gputilemap_never_constructed",
                severity="warning",
                file="src/render/core/terrain_render_authority.rs",
                hint="retire or milestone-gate GpuTilemap until DR-MIG-TILEMAP unblocks",
            )
        )

    for rel, needle, hint in DOC_LIES:
        text = _read(rel)
        if text is None:
            continue
        if needle not in text:
            continue
        line_no = 0
        for i, line in enumerate(text.splitlines(), 1):
            if needle in line:
                line_no = i
                break
        issues.append(
            _issue(
                kind="doc_promises_gpu_atlas",
                severity="warning",
                file=rel,
                line=line_no,
                hint=hint,
            )
        )

    errors = [i for i in issues if i["severity"] == "error"]
    warnings = [i for i in issues if i["severity"] == "warning"]
    status = "failed" if errors else ("warning" if warnings else "passed")

    body: dict[str, Any] = {
        "schema_version": 1,
        "validator": "terrain_honesty",
        "status": status,
        "compression_level": 3,
        "summary": (
            f"terrain honesty: {len(errors)} errors, {len(warnings)} warnings; "
            f"minimap={label}; GpuTilemap_sites={len(gputile_hits)}"
        ),
        "error_count": len(errors),
        "warning_count": len(warnings),
        "errors": errors,
        "warnings": warnings,
        "known_fixes": [
            {"signature": "doc_gpu_atlas", "fix": "RPC-1-002 update docs to world_raster", "confidence": 0.9},
            {"signature": "rename_authority", "fix": "RPC-1-003 GpuBake/CpuRaster rename", "confidence": 0.85},
            {"signature": "real_bake", "fix": "RPC-1-004/005 GPU bake then flip label", "confidence": 0.8},
        ],
        "confidence": 0.95,
        "facts": facts,
        "program": "PLAN-RENDER-PROD-CLEANUP-v1 / RPC-1 A+B",
        "ok": len(errors) == 0,
        "honest_gate": (
            "honest"
            if not issues
            else ("dishonest_docs" if not errors else "failed")
        ),
    }

    if write_witness:
        out = repo_root() / WITNESS_PATH
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
        body["witness_path"] = WITNESS_PATH

    return body


def validate_terrain_honesty_report(*, write_witness: bool = True, compress: int = 3) -> dict[str, Any]:
    body = terrain_honesty_lint(write_witness=write_witness)
    body["compression_level"] = max(1, min(4, compress))
    if body["compression_level"] >= 4:
        body.pop("errors", None)
        body.pop("warnings", None)
        body.pop("facts", None)
    return body
