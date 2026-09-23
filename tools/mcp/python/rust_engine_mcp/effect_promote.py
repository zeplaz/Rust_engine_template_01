"""VSS-T4-003 — EffectSpec wgsl_pack staging + promote to assets/effects/registry/."""

from __future__ import annotations

import hashlib
import json
import shutil
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from .paths import repo_root, staging_root
from .schemas import load_json_file
from .validators.effect_spec import validate_effect_spec

REFERENCE_BATCH_ID = "vss_t4_reference_effects_v1"
REGISTRY_REL = "assets/effects/registry"
WITNESS_REL = "debug_runs/artist_vfx_pipeline_live.json"

REFERENCE_SPEC_RELS = (
    "tools/mcp/schemas/examples/effect_spec_spark_shower_v1.json",
    "tools/mcp/schemas/examples/effect_spec_smoke_column_v1.json",
    "tools/mcp/schemas/examples/effect_spec_rain_streaks_v1.json",
)

PACK_MANIFEST = "pack_manifest.json"
EFFECT_SPEC_NAME = "effect_spec.json"
PROMOTE_MANIFEST = "manifest.json"


def _utc_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def _resolve_spec(spec_path: str | Path) -> Path:
    p = Path(spec_path)
    if not p.is_absolute():
        p = repo_root() / p
    if not p.is_file():
        raise FileNotFoundError(f"EffectSpec not found: {p}")
    return p


def _sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _shader_pack_entries(spec: dict[str, Any]) -> list[tuple[str, str]]:
    visual = spec.get("visual")
    if not isinstance(visual, dict):
        return []
    pack = visual.get("shader_pack")
    if not isinstance(pack, dict):
        return []
    entries: list[tuple[str, str]] = []
    for role in ("compute", "draw", "particle"):
        rel = pack.get(role)
        if isinstance(rel, str) and rel.strip():
            entries.append((role, rel.strip().replace("\\", "/")))
    return entries


def _require_rules_check(spec: dict[str, Any], *, path: Path) -> None:
    rc = spec.get("rules_check")
    if not isinstance(rc, dict) or rc.get("passed") is not True:
        raise ValueError(f"rules_check.passed must be true before pack/promote: {path.name}")


def _require_validate(spec_path: Path) -> dict[str, Any]:
    report = validate_effect_spec(spec_path, compression_level=4)
    if report.status == "failed":
        hints = [e.hint for e in report.errors if e.severity == "error"][:3]
        raise ValueError(f"EffectSpec validation failed: {hints or report.summary}")
    return load_json_file(spec_path)


def preview_witness_stub(
    *,
    effect_id: str,
    staging_rel: str,
    promoted_rel: str = "",
) -> dict[str, Any]:
    """Honest_gate pending until G4 preview worker runs (VSS-T4-003 stub envelope)."""
    body: dict[str, Any] = {
        "witness_schema": "artist_vfx_preview_witness_v1",
        "witness_path": WITNESS_REL,
        "preview_job_id": f"{effect_id}_preview_stub",
        "frames_captured": 0,
        "staging_path": staging_rel,
        "honest_gate": "pending",
        "captured_at_utc": _utc_now(),
        "toolchain": {
            "mcp_tool": "effect_promote",
            "cli_parity": True,
            "blender_present": False,
        },
        "_note": "G4 preview worker not run — capture_hash omitted until frames captured",
    }
    if promoted_rel:
        body["promoted_path"] = promoted_rel
    return body


def pack_effect_spec(spec_path: str | Path, *, force: bool = False) -> dict[str, Any]:
    """Copy shader_pack WGSL into assets/staging/<effect_id>/ with SHA-256 manifest (R-SCHEMA-3)."""
    path = _resolve_spec(spec_path)
    spec = _require_validate(path)
    _require_rules_check(spec, path=path)

    visual = spec.get("visual") or {}
    source = str(visual.get("source") or "")
    if source != "wgsl_pack":
        raise ValueError(f"pack supports wgsl_pack only (got {source!r}) for {path.name}")

    effect_id = str(spec["effect_id"])
    batch_id = str(spec.get("batch_id") or "")
    staging_dir = staging_root() / effect_id
    shaders_dir = staging_dir / "shaders"
    shaders_dir.mkdir(parents=True, exist_ok=True)

    shader_hashes: dict[str, Any] = {}
    hash_parts: list[str] = []
    for role, rel in _shader_pack_entries(spec):
        src = repo_root() / rel
        if not src.is_file():
            raise FileNotFoundError(f"shader_pack.{role} missing on disk: {rel}")
        dest_name = Path(rel).name
        dest = shaders_dir / dest_name
        digest = _sha256_file(src)
        if dest.is_file() and not force:
            if _sha256_file(dest) != digest:
                raise ValueError(
                    f"staging shader hash drift for {effect_id}/{dest_name} — use force=True to overwrite"
                )
        else:
            shutil.copy2(src, dest)
        shader_hashes[role] = {
            "source": rel,
            "dest": f"shaders/{dest_name}",
            "sha256": digest,
        }
        hash_parts.append(f"{role}:{digest}")

    if not shader_hashes:
        raise ValueError(f"shader_pack empty for {effect_id}")

    pack_hash = hashlib.sha256("|".join(sorted(hash_parts)).encode("utf-8")).hexdigest()
    manifest = {
        "schema": "effect_pack_manifest_v1",
        "effect_id": effect_id,
        "batch_id": batch_id,
        "packed_at_utc": _utc_now(),
        "spec_source": str(path.relative_to(repo_root())).replace("\\", "/"),
        "shader_hashes": shader_hashes,
        "pack_hash": pack_hash,
    }
    (staging_dir / PACK_MANIFEST).write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    shutil.copy2(path, staging_dir / EFFECT_SPEC_NAME)

    staging_rel = f"assets/staging/{effect_id}/"
    return {
        "effect_id": effect_id,
        "batch_id": batch_id,
        "phase": "pack",
        "staging_dir": staging_rel,
        "pack_manifest": f"{staging_rel}{PACK_MANIFEST}",
        "shader_count": len(shader_hashes),
        "pack_hash": pack_hash,
        "preview_witness": preview_witness_stub(effect_id=effect_id, staging_rel=staging_rel),
    }


def _load_pack_manifest(staging_dir: Path) -> dict[str, Any]:
    manifest_path = staging_dir / PACK_MANIFEST
    if not manifest_path.is_file():
        raise FileNotFoundError(f"Missing {PACK_MANIFEST} under {staging_dir} — run pack first")
    return json.loads(manifest_path.read_text(encoding="utf-8"))


def _verify_staging_hashes(staging_dir: Path, manifest: dict[str, Any]) -> None:
    for role, entry in (manifest.get("shader_hashes") or {}).items():
        if not isinstance(entry, dict):
            raise ValueError(f"invalid shader_hashes.{role}")
        dest_rel = str(entry.get("dest") or "")
        expected = str(entry.get("sha256") or "")
        dest = staging_dir / dest_rel
        if not dest.is_file():
            raise FileNotFoundError(f"staged shader missing: {dest_rel}")
        actual = _sha256_file(dest)
        if actual != expected:
            raise ValueError(f"staging hash mismatch for {role}: expected {expected[:12]}… got {actual[:12]}…")


def _promote_gate(spec: dict[str, Any], *, force: bool) -> None:
    pw = spec.get("preview_witness")
    if isinstance(pw, dict):
        gate = str(pw.get("honest_gate") or "pending")
        if gate == "dishonest_gate" and not force:
            raise ValueError("preview_witness.honest_gate is dishonest_gate — promote blocked")
        tier = str(spec.get("development_tier") or "smoke")
        if tier == "production" and gate != "honest" and not force:
            raise ValueError(
                "production tier requires preview_witness.honest_gate=honest (or force=True)"
            )


def promote_effect(spec_path: str | Path, *, force: bool = False, pack_first: bool = True) -> dict[str, Any]:
    """Promote staged wgsl pack bundle to assets/effects/registry/<effect_id>/."""
    path = _resolve_spec(spec_path)
    if pack_first:
        pack_effect_spec(path, force=force)
    spec = _require_validate(path)
    _require_rules_check(spec, path=path)
    _promote_gate(spec, force=force)

    effect_id = str(spec["effect_id"])
    batch_id = str(spec.get("batch_id") or "")
    staging_dir = staging_root() / effect_id
    manifest = _load_pack_manifest(staging_dir)
    if manifest.get("effect_id") != effect_id:
        raise ValueError("pack_manifest.effect_id mismatch")
    _verify_staging_hashes(staging_dir, manifest)

    registry_dir = repo_root() / REGISTRY_REL / effect_id
    registry_dir.mkdir(parents=True, exist_ok=True)

    for role, entry in (manifest.get("shader_hashes") or {}).items():
        dest_rel = str(entry.get("dest") or "")
        src = staging_dir / dest_rel
        dest = registry_dir / Path(dest_rel).name
        shutil.copy2(src, dest)

    shutil.copy2(staging_dir / EFFECT_SPEC_NAME, registry_dir / EFFECT_SPEC_NAME)
    promoted_rel = f"{REGISTRY_REL}/{effect_id}/"
    promote_body = {
        "schema": "effect_promote_manifest_v1",
        "effect_id": effect_id,
        "batch_id": batch_id,
        "promoted_at_utc": _utc_now(),
        "registry_dir": promoted_rel,
        "pack_hash": manifest.get("pack_hash"),
        "shader_hashes": manifest.get("shader_hashes"),
        "development_tier": spec.get("development_tier"),
        "lane": spec.get("lane"),
        "preview_witness": preview_witness_stub(
            effect_id=effect_id,
            staging_rel=f"assets/staging/{effect_id}/",
            promoted_rel=promoted_rel,
        ),
    }
    (registry_dir / PROMOTE_MANIFEST).write_text(json.dumps(promote_body, indent=2) + "\n", encoding="utf-8")

    return {
        "effect_id": effect_id,
        "batch_id": batch_id,
        "phase": "promote",
        "registry_dir": promoted_rel,
        "pack_hash": manifest.get("pack_hash"),
        "preview_witness": promote_body["preview_witness"],
    }


def _reference_specs_for_batch(batch_id: str) -> list[Path]:
    root = repo_root()
    specs: list[Path] = []
    for rel in REFERENCE_SPEC_RELS:
        path = root / rel
        if not path.is_file():
            continue
        data = load_json_file(path)
        if str(data.get("batch_id") or "") == batch_id:
            specs.append(path)
    if not specs and batch_id == REFERENCE_BATCH_ID:
        specs = [_resolve_spec(rel) for rel in REFERENCE_SPEC_RELS]
    if not specs:
        raise FileNotFoundError(f"No EffectSpec examples found for batch_id={batch_id!r}")
    return specs


def pack_reference_batch(batch_id: str = REFERENCE_BATCH_ID, *, force: bool = False) -> dict[str, Any]:
    results = [pack_effect_spec(p, force=force) for p in _reference_specs_for_batch(batch_id)]
    return {
        "batch_id": batch_id,
        "phase": "batch_pack",
        "count": len(results),
        "effects": results,
    }


def promote_reference_batch(
    batch_id: str = REFERENCE_BATCH_ID,
    *,
    force: bool = False,
    pack_first: bool = True,
) -> dict[str, Any]:
    specs = _reference_specs_for_batch(batch_id)
    results = [promote_effect(p, force=force, pack_first=pack_first) for p in specs]
    return {
        "batch_id": batch_id,
        "phase": "batch_promote",
        "count": len(results),
        "effects": results,
    }


def effect_promote(
    *,
    spec_path: str = "",
    batch_id: str = "",
    phase: str = "full",
    force: bool = False,
    write_witness: bool = True,
) -> dict[str, Any]:
    """Shared MCP/CLI entry — pack, promote, or full on one spec or reference batch."""
    phase_norm = phase.strip().lower()
    if phase_norm not in ("pack", "promote", "full"):
        raise ValueError("phase must be pack, promote, or full")

    body: dict[str, Any]
    if batch_id or not spec_path:
        bid = batch_id or REFERENCE_BATCH_ID
        if phase_norm == "pack":
            body = pack_reference_batch(bid, force=force)
        elif phase_norm == "promote":
            body = promote_reference_batch(bid, force=force, pack_first=False)
        else:
            pack_reference_batch(bid, force=force)
            body = promote_reference_batch(bid, force=force, pack_first=False)
    else:
        if not spec_path:
            raise ValueError("spec_path required when batch_id is omitted")
        if phase_norm == "pack":
            body = pack_effect_spec(spec_path, force=force)
        elif phase_norm == "promote":
            body = promote_effect(spec_path, force=force, pack_first=False)
        else:
            pack_effect_spec(spec_path, force=force)
            body = promote_effect(spec_path, force=force, pack_first=False)

    if write_witness:
        body["witness"] = refresh_artist_vfx_pipeline_witness()
    return body


def refresh_artist_vfx_pipeline_witness(*, repo: Path | None = None) -> dict[str, Any]:
    """Refresh debug_runs/artist_vfx_pipeline_live.json from disk (partial_ship → closer to green)."""
    root = repo or repo_root()
    witness_path = root / WITNESS_REL

    staging_effects: dict[str, Any] = {}
    promoted_effects: dict[str, Any] = {}
    for rel in REFERENCE_SPEC_RELS:
        spec_path = root / rel
        if not spec_path.is_file():
            continue
        spec = load_json_file(spec_path)
        effect_id = str(spec.get("effect_id") or spec_path.stem)
        staging_dir = root / "assets" / "staging" / effect_id
        registry_dir = root / REGISTRY_REL / effect_id
        pack_manifest = staging_dir / PACK_MANIFEST
        row: dict[str, Any] = {
            "spec": str(spec_path.relative_to(root)).replace("\\", "/"),
            "lane": spec.get("lane"),
            "validation": validate_effect_spec(spec_path, compression_level=4).status,
        }
        if pack_manifest.is_file():
            pm = json.loads(pack_manifest.read_text(encoding="utf-8"))
            row["staging_path"] = f"assets/staging/{effect_id}/"
            row["pack_hash"] = pm.get("pack_hash")
            row["shader_count"] = len(pm.get("shader_hashes") or {})
            row["preview_witness"] = preview_witness_stub(
                effect_id=effect_id,
                staging_rel=f"assets/staging/{effect_id}/",
                promoted_rel=f"{REGISTRY_REL}/{effect_id}/" if (registry_dir / PROMOTE_MANIFEST).is_file() else "",
            )
            staging_effects[effect_id] = row
        promote_manifest = registry_dir / PROMOTE_MANIFEST
        if promote_manifest.is_file():
            pm = json.loads(promote_manifest.read_text(encoding="utf-8"))
            promoted_effects[effect_id] = {
                "registry_path": f"{REGISTRY_REL}/{effect_id}/",
                "pack_hash": pm.get("pack_hash"),
                "lane": pm.get("lane"),
                "preview_witness": pm.get("preview_witness"),
            }

    packed_count = len(staging_effects)
    promoted_count = len(promoted_effects)
    pack_complete = packed_count >= 3
    promote_complete = promoted_count >= 3
    green = pack_complete and promote_complete

    body: dict[str, Any] = {
        "_agent_meta": {
            "profile": "VSS-T4-003",
            "schema": "debug_run_envelope_v1",
            "relative_path": WITNESS_REL,
            "source_system": "effect_promote",
            "track": "VSS-T4",
            "task_id": "VSS-T4-003",
            "proceed_ship": promote_complete,
            "art_quality": "smoke_tier_reference_batch",
            "docs": {
                "plan": "src/dev/plan_artist_vfx_toolchain_v1.md",
                "schema": "tools/mcp/schemas/effect_spec_v1.schema.json",
            },
        },
        "schema": "artist_vfx_pipeline_witness_v1",
        "status": "partial_ship" if not green else "shipped",
        "program_id": "VSS-001",
        "slice_id": "VSS-T4-003",
        "green": green,
        "schema_risks": [
            {
                "id": "R-SCHEMA-1",
                "title": "allOf conditional spawn_hook_config",
                "owner": "@coder-mcp",
                "status": "mitigated",
                "mitigation": "Draft202012Validator + pytest — validate_effect_spec_report",
            },
            {
                "id": "R-SCHEMA-2",
                "title": "LOD cap drift vs FIRE_LOD_CAP_* runtime",
                "owner": "@coder",
                "mitigation": "lint bridge after VSS-T1-002",
            },
            {
                "id": "R-SCHEMA-3",
                "title": "wgsl_pack paths vs assets/effects/registry promote bundle",
                "owner": "@coder-mcp",
                "status": "mitigated" if pack_complete else "open",
                "mitigation": "pack_manifest sha256 per shader_pack role — effect_promote pack step",
            },
        ],
        "shipped": {
            "effect_spec_schema": "tools/mcp/schemas/effect_spec_v1.schema.json",
            "effect_spec_validator": "tools/mcp/python/rust_engine_mcp/validators/effect_spec.py",
            "validate_effect_spec_mcp": "validate_effect_spec_report",
            "validate_effect_spec_cli": "validate-report effect_spec",
            "effect_promote_mcp": "effect_promote",
            "effect_promote_cli": "effect-promote",
            "effect_pack_cli": "effect-pack",
            "reference_batch_id": REFERENCE_BATCH_ID,
            "plan_doc": "src/dev/plan_artist_vfx_toolchain_v1.md",
        },
        "reference_specs": {
            "batch_id": REFERENCE_BATCH_ID,
            "status": "shipped" if promote_complete else ("packed" if pack_complete else "partial_ship"),
            "slice": "VSS-T4-003",
            "validator": "rust_engine_mcp.validators.effect_spec",
            "validation": {
                "effect_spec_smoke_column_v1.json": validate_effect_spec(
                    root / REFERENCE_SPEC_RELS[1], compression_level=4
                ).status,
                "effect_spec_rain_streaks_v1.json": validate_effect_spec(
                    root / REFERENCE_SPEC_RELS[2], compression_level=4
                ).status,
                "effect_spec_spark_shower_v1.json": validate_effect_spec(
                    root / REFERENCE_SPEC_RELS[0], compression_level=4
                ).status,
            },
            "lane_decisions": {
                "smoke_column": "gpu_field",
                "rain_streaks": "particle_instanced",
                "rain_lane_rationale": "ES-5 GPU precip spine; climate_precip field_sample drives density",
            },
            "staging": staging_effects,
            "promoted": promoted_effects,
            "gaps": [] if promote_complete else ["preview_witness honest_gate pending until G4 preview worker"],
        },
        "pending": {
            "preview_witness_capture": "G4 preview worker — capture_hash + honest_gate=honest",
            "effect_consumable_ecs": "VSS-T4-004",
        },
        "tribunal": {"dissent": []},
    }

    witness_path.parent.mkdir(parents=True, exist_ok=True)
    witness_path.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = WITNESS_REL
    return body
