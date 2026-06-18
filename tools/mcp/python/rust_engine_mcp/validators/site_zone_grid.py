"""CMCP-SITE-ZONE-VALIDATE-001 — site_zone_grid_v1 taxonomy + layout rules."""

from __future__ import annotations

import json
from collections import deque
from pathlib import Path
from typing import Any

from .report import ValidationIssue, ValidationReport

CANONICAL_ZONES = frozenset(
    {"primary", "loading", "utility", "rail", "service", "parking", "buffer"}
)

ROLE_PRIMARY_MIN: dict[str, float] = {
    "aggregate_mine": 0.25,
    "cement_kiln": 0.15,
    "concrete_mixer": 0.12,
    "integrated_plant": 0.12,
    "bauxite_mine": 0.25,
    "alumina_refinery": 0.12,
    "aluminum_smelter": 0.12,
    "aluminum_fabrication": 0.12,
}

ROLE_UTILITY_MIN: dict[str, float] = {
    "cement_kiln": 0.20,
    "alumina_refinery": 0.25,
    "aluminum_smelter": 0.30,
    "substation": 0.50,
    "power_plant": 0.40,
}

HEAVY_UTILITY_ROLES = frozenset(
    {"cement_kiln", "alumina_refinery", "aluminum_smelter", "substation", "power_plant"}
)

DEFAULT_PILOT_PATHS: tuple[str, ...] = (
    "assets/configs/buildings/pilots/logistics_rail_warehouse_site_v0.json",
    "assets/configs/buildings/pilots/manufacturing_fabrication_hall_site_v0.json",
    "assets/configs/buildings/pilots/fuel_depot_tank_farm_site_v0.json",
    "assets/configs/buildings/pilots/power_substation_yard_site_v0.json",
    "assets/configs/buildings/pilots/concrete_aggregate_mine_site_v0.json",
    "assets/configs/buildings/pilots/concrete_cement_kiln_site_v0.json",
    "assets/configs/buildings/pilots/concrete_mixer_plant_site_v0.json",
)


def _zone_counts(cells: list[str]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for cell in cells:
        counts[cell] = counts.get(cell, 0) + 1
    return counts


def _pct(count: int, total: int) -> float:
    return (count / total) if total > 0 else 0.0


def _loading_touches_perimeter(cells: list[str], width: int, depth: int) -> bool:
    if width <= 0 or depth <= 0:
        return False
    for idx, zone in enumerate(cells):
        if zone != "loading":
            continue
        row, col = divmod(idx, width)
        if row == 0 or row == depth - 1 or col == 0 or col == width - 1:
            return True
    return False


def _rail_contiguous_ok(cells: list[str], width: int, depth: int) -> bool:
    rail_idx = [i for i, z in enumerate(cells) if z == "rail"]
    if not rail_idx:
        return True
    if len(rail_idx) < 2:
        return False
    start = rail_idx[0]
    seen = {start}
    queue: deque[int] = deque([start])
    while queue:
        cur = queue.popleft()
        row, col = divmod(cur, width)
        for dr, dc in ((0, 1), (0, -1), (1, 0), (-1, 0)):
            nr, nc = row + dr, col + dc
            if nr < 0 or nc < 0 or nr >= depth or nc >= width:
                continue
            nidx = nr * width + nc
            if nidx in seen or cells[nidx] != "rail":
                continue
            seen.add(nidx)
            queue.append(nidx)
    return len(seen) == len(rail_idx) and len(rail_idx) >= 2


def _ascii_legend_ok(body: dict[str, Any]) -> bool:
    legend = body.get("zone_legend") or {}
    if not isinstance(legend, dict):
        return False
    chars: set[str] = set()
    for row in body.get("ascii_plan") or []:
        for ch in str(row):
            if ch != " ":
                chars.add(ch)
    for ch in chars:
        if ch not in legend:
            return False
    return True


def validate_site_zone_grid(
    data: dict[str, Any],
    *,
    path: str = "",
    compression_level: int = 3,
) -> ValidationReport:
    issues: list[ValidationIssue] = []
    width = int(data.get("width") or 0)
    depth = int(data.get("depth") or 0)
    cells = list(data.get("cells") or [])
    role = str(data.get("supply_chain_role") or data.get("utility_role") or "")

    if data.get("schema_version") != "site_zone_grid_v1":
        issues.append(
            ValidationIssue(
                kind="SZ-00",
                severity="error",
                file=path,
                hint="schema_version must be site_zone_grid_v1",
            )
        )

    for idx, zone in enumerate(cells):
        if zone not in CANONICAL_ZONES:
            issues.append(
                ValidationIssue(
                    kind="SZ-01",
                    severity="error",
                    file=path,
                    hint=f"cell[{idx}] unknown zone {zone!r}",
                )
            )

    expected = width * depth
    if expected <= 0 or len(cells) != expected:
        issues.append(
            ValidationIssue(
                kind="SZ-02",
                severity="error",
                file=path,
                hint=f"width×depth={expected} but cells={len(cells)}",
            )
        )

    if not _ascii_legend_ok(data):
        issues.append(
            ValidationIssue(
                kind="SZ-08",
                severity="error",
                file=path,
                hint="ascii_plan chars must exist in zone_legend",
            )
        )

    total = len(cells) if cells else 1
    counts = _zone_counts(cells)
    metrics = data.get("metrics") or {}
    primary_pct = float(metrics.get("primary_pct_site") or _pct(counts.get("primary", 0), total))
    utility_pct = _pct(counts.get("utility", 0), total)

    if role in ROLE_PRIMARY_MIN and primary_pct + 1e-9 < ROLE_PRIMARY_MIN[role]:
        issues.append(
            ValidationIssue(
                kind="SZ-03",
                severity="warning",
                file=path,
                hint=f"primary_pct_site {primary_pct:.2%} < min {ROLE_PRIMARY_MIN[role]:.0%} for {role}",
            )
        )

    if role in HEAVY_UTILITY_ROLES and role in ROLE_UTILITY_MIN:
        min_u = ROLE_UTILITY_MIN[role]
        if utility_pct + 1e-9 < min_u:
            issues.append(
                ValidationIssue(
                    kind="SZ-04",
                    severity="warning",
                    file=path,
                    hint=f"utility_pct {utility_pct:.2%} < min {min_u:.0%} for {role}",
                )
            )

    if counts.get("loading", 0) > 0 and width > 0 and depth > 0:
        if not _loading_touches_perimeter(cells, width, depth):
            issues.append(
                ValidationIssue(
                    kind="SZ-05",
                    severity="warning",
                    file=path,
                    hint="loading zone does not touch site perimeter",
                )
            )

    if counts.get("rail", 0) > 0 and width > 0 and depth > 0:
        if not _rail_contiguous_ok(cells, width, depth):
            issues.append(
                ValidationIssue(
                    kind="SZ-06",
                    severity="warning",
                    file=path,
                    hint="rail cells must be contiguous and >= 2",
                )
            )

    errors = [i for i in issues if i.severity == "error"]
    warnings = [i for i in issues if i.severity == "warning"]
    status = "failed" if errors else ("warning" if warnings else "passed")
    return ValidationReport(
        validator="test",
        status=status,
        compression_level=compression_level,
        summary=f"site_zone_grid {path or data.get('site_id', '?')}: {len(errors)} error(s), {len(warnings)} warn(s)",
        error_count=len(errors),
        warning_count=len(warnings),
        errors=issues,
        confidence=0.95,
    )


def validate_site_zone_grid_path(path: Path, *, compression_level: int = 3) -> ValidationReport:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        return ValidationReport(
            validator="test",
            status="failed",
            compression_level=compression_level,
            summary=f"site_zone_grid parse failed: {exc}",
            error_count=1,
            errors=[
                ValidationIssue(kind="parse", severity="error", file=str(path), hint=str(exc)),
            ],
        )
    rel = str(path).replace("\\", "/")
    return validate_site_zone_grid(data, path=rel, compression_level=compression_level)


def scan_site_zone_pilots(
    paths: tuple[str, ...] | None = None,
    *,
    repo_root: Path,
) -> dict[str, Any]:
    rel_paths = paths or DEFAULT_PILOT_PATHS
    rows: list[dict[str, Any]] = []
    for rel in rel_paths:
        path = repo_root / rel
        if not path.is_file():
            rows.append({"path": rel, "green": False, "error": "missing"})
            continue
        report = validate_site_zone_grid_path(path)
        rows.append(
            {
                "path": rel,
                "site_id": json.loads(path.read_text(encoding="utf-8")).get("site_id"),
                "green": report.error_count == 0,
                "status": report.status,
                "error_count": report.error_count,
                "warning_count": report.warning_count,
            }
        )
    green = all(r.get("green") for r in rows) and bool(rows)
    return {
        "task_id": "CMCP-SITE-ZONE-VALIDATE-001",
        "green": green,
        "pilot_count": len(rows),
        "pilots": rows,
    }


def write_site_zone_validate_witness(
    *,
    repo_root: Path | None = None,
    paths: tuple[str, ...] | None = None,
) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness
    from rust_engine_mcp.paths import repo_root as default_root

    root = repo_root or default_root()
    body = scan_site_zone_pilots(paths, repo_root=root)
    return write_aps_live_witness(
        body,
        "debug_runs/site_zone_validate_live.json",
        schema="site_zone_validate_live_v1",
        profile="CMCP_SITE_ZONE_VALIDATE",
        source_system="site_zone_grid_validator",
        ritual="BLANG:WIT-HON CMCP-SITE-ZONE-VALIDATE-001" if body.get("green") else None,
    )
