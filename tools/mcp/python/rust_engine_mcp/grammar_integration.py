"""MCP-INTEGRATE-001 — grammar_integration_validate complex snapshot gate."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from rust_engine_mcp.aps_validator_plain import fix_hint, plain_sentence
from rust_engine_mcp.grammar_build_set import grammar_preset_pair_validate, load_pilot_catalog
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators.assembly_grammar_verify import validate_assembly_p0_gate
from rust_engine_mcp.validators.report import ValidationIssue, ValidationReport

WITNESS_PATH = "debug_runs/grammar_integration_validate_live.json"
DEFAULT_SNAPSHOT = (
    "tools/mcp/schemas/examples/assembly_snapshot_warehouse_industrial_west_production_v1.json"
)


def _resolve(path: str | Path) -> Path:
    p = Path(path)
    if not p.is_absolute():
        p = repo_root() / p
    return p


def _infer_preset_id(snapshot: dict[str, Any]) -> str | None:
    for key in ("arch_dna_preset_id", "arch_build_grammar_preset_id", "arch_dna_preset"):
        val = snapshot.get(key)
        if val:
            return str(val)
    archetype = str(snapshot.get("archetype_id") or "")
    district = str(snapshot.get("district_style") or "")
    if not archetype:
        chain = snapshot.get("grammar_rule_chain") or {}
        if isinstance(chain, dict):
            archetype = str(chain.get("archetype") or "")
    if not archetype:
        return None
    for pilot in load_pilot_catalog():
        if pilot.get("grammar_archetype_id") == archetype:
            if not district or pilot.get("district_style") == district:
                return str(pilot.get("arch_dna_preset") or "")
    return None


def _site_path_for_preset(preset_id: str) -> str | None:
    for pilot in load_pilot_catalog():
        if pilot.get("arch_dna_preset") == preset_id:
            rel = pilot.get("site_json_path")
            if rel:
                return f"assets/configs/buildings/{rel}".replace("\\", "/")
    return None


def grammar_integration_validate(
    path: str | Path,
    *,
    ship: bool = True,
    compression_level: int = 3,
) -> dict[str, Any]:
    """Compose P0 gate + DNA preset pair + optional site path checks."""
    resolved = _resolve(path)
    if not resolved.is_file():
        return {
            "task_id": "MCP-INTEGRATE-001",
            "schema": "grammar_integration_validate_v1",
            "ok": False,
            "green": False,
            "path": str(path),
            "errors": ["snapshot not found"],
            "artist_messages": [
                {
                    "sentence": "Snapshot file not found.",
                    "fix": "Load or Save a valid assembly JSON.",
                    "signature": "grammar_verify_snapshot_missing",
                }
            ],
        }

    try:
        rel = str(resolved.relative_to(repo_root())).replace("\\", "/")
    except ValueError:
        rel = str(resolved)

    snap = json.loads(resolved.read_text(encoding="utf-8"))
    issues: list[ValidationIssue] = []

    p0 = validate_assembly_p0_gate(
        snap,
        snapshot_path=rel,
        ship=ship,
        compression_level=compression_level,
    )
    issues.extend(p0.errors)

    preset_id = _infer_preset_id(snap)
    pair_body: dict[str, Any] | None = None
    if preset_id:
        pair_body = grammar_preset_pair_validate(preset_id=preset_id)
        if not pair_body.get("green"):
            for err in pair_body.get("errors") or []:
                issues.append(
                    ValidationIssue(
                        kind="GrammarPresetPair",
                        severity="error",
                        file=rel,
                        hint=str(err),
                        signature="grammar_integration_preset_pair",
                    )
                )
    elif ship and (snap.get("grammar_rule_chain") or snap.get("procedural_rules_version")):
        issues.append(
            ValidationIssue(
                kind="GrammarPresetPair",
                severity="error",
                file=rel,
                hint="grammar snapshot missing arch_dna preset linkage",
                signature="grammar_integration_preset_missing",
            )
        )

    site_rel: str | None = None
    site_ok: bool | None = None
    if preset_id:
        site_rel = _site_path_for_preset(preset_id)
        if site_rel:
            site_ok = (repo_root() / site_rel).is_file()
            if not site_ok and ship:
                issues.append(
                    ValidationIssue(
                        kind="SiteComposition",
                        severity="error",
                        file=site_rel,
                        hint=f"pilot site JSON missing for preset {preset_id}",
                        signature="grammar_integration_site_missing",
                    )
                )

    errors = [i for i in issues if i.severity == "error"]
    green = len(errors) == 0

    seen: set[str] = set()
    artist_messages: list[dict[str, str]] = []
    for issue in errors:
        dedupe = issue.signature or issue.kind
        if dedupe in seen:
            continue
        seen.add(dedupe)
        artist_messages.append(
            {
                "sentence": plain_sentence(
                    issue.signature or "",
                    issue.kind or "",
                    fallback=issue.hint or issue.kind,
                ).replace("**", ""),
                "fix": fix_hint(
                    issue.signature or "",
                    issue.kind or "",
                    fallback=issue.hint or "",
                ).replace("**", ""),
                "signature": issue.signature or issue.kind,
            }
        )

    return {
        "task_id": "MCP-INTEGRATE-001",
        "schema": "grammar_integration_validate_v1",
        "ok": green,
        "green": green,
        "path": rel,
        "preset_id": preset_id,
        "preset_pair": pair_body,
        "site_json_path": site_rel,
        "site_ok": site_ok,
        "p0_summary": p0.summary,
        "error_count": len(errors),
        "artist_messages": artist_messages,
        "technical": {
            "p0_status": p0.status,
            "compression_level": compression_level,
        },
    }


def validate_grammar_integration_path(
    path: str | Path,
    *,
    ship: bool = True,
    compression_level: int = 3,
) -> ValidationReport:
    body = grammar_integration_validate(path, ship=ship, compression_level=compression_level)
    status = "passed" if body.get("green") else "failed"
    errors = [
        ValidationIssue(
            kind=str(msg.get("signature") or "GrammarIntegration"),
            severity="error",
            file=str(body.get("path") or ""),
            hint=str(msg.get("sentence") or ""),
            signature=str(msg.get("signature") or ""),
        )
        for msg in body.get("artist_messages") or []
    ]
    return ValidationReport(
        validator="grammar_integration",
        status=status,
        compression_level=compression_level,
        summary=f"grammar_integration_validate: {body.get('error_count', 0)} error(s)",
        error_count=int(body.get("error_count") or 0),
        errors=errors[:12],
        confidence=0.95 if body.get("green") else 0.88,
    )


def write_grammar_integration_witness(
    path: str | Path | None = None,
    *,
    ship: bool = True,
) -> dict[str, Any]:
    snap = path or DEFAULT_SNAPSHOT
    body = grammar_integration_validate(snap, ship=ship)
    out = repo_root() / WITNESS_PATH
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = str(out.relative_to(repo_root())).replace("\\", "/")
    return body
