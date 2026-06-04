"""Atlas meta v2 validator — TILE-FIX-002 lookup completeness (not PNG-exists)."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .report import ValidationIssue, ValidationReport

try:
    import jsonschema
except ImportError:  # pragma: no cover
    jsonschema = None  # type: ignore[assignment]


def _load_schema() -> dict[str, Any]:
    from rust_engine_mcp.paths import schemas_dir

    return json.loads((schemas_dir() / "atlas_meta_v2.schema.json").read_text(encoding="utf-8"))


def _required_lookup_keys(
    visual: dict[str, Any] | None,
    *,
    facings: int,
    minimum_g4: bool = False,
) -> set[tuple[str, int, int]]:
    """State × facing × frame 0; fire states add frames 1..frame_count-1.

    When ``minimum_g4`` is true (or visual defines ``minimum_g4_facings``), only
    ``ship_minimum_states`` × pilot facings are required (24 cells).
    """
    if not visual:
        return set()
    pilot_facings = visual.get("minimum_g4_facings")
    if minimum_g4 or pilot_facings:
        states = [
            str(s)
            for s in (
                visual.get("minimum_g4_states")
                or visual.get("ship_minimum_states")
                or []
            )
        ]
        facing_list = (
            [int(f) for f in pilot_facings]
            if pilot_facings
            else list(range(int((visual.get("render_contract") or {}).get("facings") or 8)))
        )
        return {(state, facing, 0) for state in states for facing in facing_list}

    states = [str(s) for s in (visual.get("states") or [])]
    fire = visual.get("fire") or {}
    prefix = str(fire.get("key_prefix") or "burning_")
    frame_count = int(fire.get("frame_count") or 0)
    keys: set[tuple[str, int, int]] = set()
    for state in states:
        for facing in range(facings):
            keys.add((state, facing, 0))
    if frame_count > 1:
        for i in range(frame_count):
            variant = f"{prefix}{i:02d}"
            for facing in range(facings):
                keys.add((variant, facing, 0))
    return keys


def validate_atlas_meta_v2(
    path: Path,
    *,
    visual_config_path: Path | None = None,
    compression_level: int = 3,
) -> ValidationReport:
    issues: list[ValidationIssue] = []
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        return ValidationReport(
            validator="atlas_meta_v2",
            status="failed",
            errors=[
                ValidationIssue(
                    kind="SchemaInvalid",
                    severity="error",
                    file=str(path),
                    hint=str(exc),
                    signature="atlas_meta_v2_parse",
                )
            ],
            known_fixes=[],
            summary=str(exc),
            compression_level=compression_level,
        )

    if int(data.get("schema_version") or 0) != 2:
        issues.append(
            ValidationIssue(
                kind="SchemaInvalid",
                severity="error",
                file=str(path),
                field="schema_version",
                hint="atlas_meta v2 required (v1 greybox frozen — TILE-FIX-001)",
                signature="atlas_meta_v2_version",
            )
        )

    if jsonschema is not None and not issues:
        try:
            jsonschema.validate(instance=data, schema=_load_schema())
        except jsonschema.ValidationError as exc:
            issues.append(
                ValidationIssue(
                    kind="SchemaInvalid",
                    severity="error",
                    file=str(path),
                    hint=str(exc.message),
                    signature="atlas_meta_v2_jsonschema",
                )
            )

    render = data.get("render_contract") or {}
    facings = int(render.get("facings") or 0)
    if facings not in (4, 8):
        issues.append(
            ValidationIssue(
                kind="MissingField",
                severity="error",
                file=str(path),
                field="render_contract.facings",
                hint="facings must be 4 or 8",
                signature="atlas_meta_v2_facings",
            )
        )

    lookups = data.get("lookups") or []
    present: set[tuple[str, int, int]] = set()
    for row in lookups:
        if not isinstance(row, dict):
            continue
        facing_raw = row.get("facing")
        frame_raw = row.get("frame")
        present.add(
            (
                str(row.get("variant") or ""),
                int(facing_raw) if facing_raw is not None else -1,
                int(frame_raw) if frame_raw is not None else 0,
            )
        )

    visual: dict[str, Any] | None = None
    vc_rel = str(data.get("visual_config") or "")
    vc_path = visual_config_path
    if vc_path is None and vc_rel:
        from rust_engine_mcp.paths import repo_root

        vc_path = repo_root() / vc_rel.replace("/", "\\").replace("\\", "/")
    if vc_path and vc_path.is_file():
        from .visual_config import load_visual_config

        visual = load_visual_config(vc_path)

    minimum_g4 = bool(data.get("minimum_g4_ship")) or str(data.get("lookup_mode") or "") == "minimum_g4"
    if facings in (4, 8) and visual:
        required = _required_lookup_keys(visual, facings=facings, minimum_g4=minimum_g4)
        missing = required - present
        if missing:
            sample = ", ".join(f"{v}/f{f}/fr{fr}" for v, f, fr in sorted(missing)[:8])
            issues.append(
                ValidationIssue(
                    kind="LookupIncomplete",
                    severity="error",
                    file=str(path),
                    hint=f"missing {len(missing)} lookups (e.g. {sample})",
                    signature="atlas_meta_v2_lookup_incomplete",
                )
            )

    errors = [i for i in issues if i.severity == "error"]
    status = "failed" if errors else ("warning" if issues else "passed")
    return ValidationReport(
        validator="atlas_meta_v2",
        status=status,
        errors=issues,
        known_fixes=[],
        summary=f"{path.name}: lookups={len(lookups)} facings={facings}",
        compression_level=compression_level,
    )
