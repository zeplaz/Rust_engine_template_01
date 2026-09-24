"""VSS-T4-005 — list EffectSpecs from registry + staging with honest_gate."""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable

from rust_engine_mcp.paths import repo_root, staging_root

REGISTRY_REL = "assets/effects/registry"
REFERENCE_BATCH_ID = "vss_t4_reference_effects_v1"

LANE_CAPTION: dict[str, str] = {
    "particle_instanced": "particle",
    "gpu_field": "gpu_field",
    "overlay": "overlay",
    "embellishment": "embellish",
    "embellish": "embellish",
}

HONESTY_GLYPH: dict[str, str] = {
    "honest": "✓ honest",
    "pending": "◐ pending — no frames",
    "dishonest_gate": "⊘ dishonest — do not assign",
    "missing": "○ no preview witness",
}


@dataclass(frozen=True)
class EffectBrowseEntry:
    effect_id: str
    display_name: str
    lane: str
    lane_caption: str
    batch_id: str
    development_tier: str
    spawn_hook: str
    honest_gate: str
    frames_captured: int
    source: str  # registry | staging
    spec_path: str
    manifest_path: str | None
    preview_witness: dict[str, Any]
    emit: dict[str, Any]
    lod_tiers: dict[str, Any]
    spawn_hook_config: dict[str, Any]
    staging_thumb: str | None

    @property
    def honesty_label(self) -> str:
        return HONESTY_GLYPH.get(self.honest_gate, HONESTY_GLYPH["missing"])

    @property
    def assign_allowed(self) -> bool:
        return self.honest_gate == "honest"


def _read_json(path: Path) -> dict[str, Any] | None:
    if not path.is_file():
        return None
    try:
        body = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    return body if isinstance(body, dict) else None


def _honest_gate_from(spec: dict[str, Any], manifest: dict[str, Any] | None) -> tuple[str, int, dict[str, Any]]:
    pw: dict[str, Any] = {}
    if manifest and isinstance(manifest.get("preview_witness"), dict):
        pw = dict(manifest["preview_witness"])
    elif isinstance(spec.get("preview_witness"), dict):
        pw = dict(spec["preview_witness"])
    gate = str(pw.get("honest_gate") or "").strip()
    if not gate:
        return "missing", 0, pw
    frames = int(pw.get("frames_captured") or 0)
    return gate, frames, pw


def _frames_manifest_witness(effect_dir: Path) -> dict[str, Any] | None:
    path = effect_dir / "preview_frames" / "frames_manifest.json"
    body = _read_json(path)
    if not body:
        return None
    pw = body.get("preview_witness")
    return dict(pw) if isinstance(pw, dict) else None


def _thumb_for(effect_id: str, root: Path) -> str | None:
    staging = root / "assets" / "staging" / effect_id
    registry = root / REGISTRY_REL / effect_id
    candidates = (
        staging / "capture_000.png",
        staging / "preview_frames" / "capture_000.png",
        staging / "preview.png",
        staging / "thumb.png",
        registry / "capture_000.png",
        registry / "preview_frames" / "capture_000.png",
    )
    for p in candidates:
        if p.is_file():
            return str(p.relative_to(root).as_posix())
    return None


def _entry_from_dir(effect_dir: Path, *, source: str, root: Path) -> EffectBrowseEntry | None:
    spec_path = effect_dir / "effect_spec.json"
    spec = _read_json(spec_path)
    if not spec:
        return None
    effect_id = str(spec.get("effect_id") or effect_dir.name).strip()
    if not effect_id:
        return None
    manifest_path = effect_dir / "manifest.json"
    manifest = _read_json(manifest_path)
    gate, frames, pw = _honest_gate_from(spec, manifest)
    # Prefer verified staging capture witness when promote manifest is still pending.
    captured = _frames_manifest_witness(effect_dir)
    if captured and str(captured.get("honest_gate") or "") == "honest":
        if gate != "honest" or int(captured.get("frames_captured") or 0) > frames:
            pw = captured
            gate = str(captured.get("honest_gate") or gate)
            frames = int(captured.get("frames_captured") or frames)
    lane = str(spec.get("lane") or "")
    return EffectBrowseEntry(
        effect_id=effect_id,
        display_name=str(spec.get("display_name") or effect_id),
        lane=lane,
        lane_caption=LANE_CAPTION.get(lane, lane or "—"),
        batch_id=str(spec.get("batch_id") or ""),
        development_tier=str(spec.get("development_tier") or "smoke"),
        spawn_hook=str(spec.get("spawn_hook") or ""),
        honest_gate=gate,
        frames_captured=frames,
        source=source,
        spec_path=str(spec_path.relative_to(root).as_posix()),
        manifest_path=str(manifest_path.relative_to(root).as_posix()) if manifest_path.is_file() else None,
        preview_witness=pw,
        emit=dict(spec.get("emit") or {}) if isinstance(spec.get("emit"), dict) else {},
        lod_tiers=dict(spec.get("lod_tiers") or {}) if isinstance(spec.get("lod_tiers"), dict) else {},
        spawn_hook_config=(
            dict(spec.get("spawn_hook_config") or {})
            if isinstance(spec.get("spawn_hook_config"), dict)
            else {}
        ),
        staging_thumb=_thumb_for(effect_id, root),
    )


def list_effect_entries(
    *,
    repo: Path | None = None,
    include_staging: bool = True,
    batch_id: str | None = None,
) -> list[EffectBrowseEntry]:
    """Scan registry (+ optional staging) EffectSpecs."""
    root = repo or repo_root()
    by_id: dict[str, EffectBrowseEntry] = {}

    registry = root / REGISTRY_REL
    if registry.is_dir():
        for child in sorted(registry.iterdir()):
            if not child.is_dir():
                continue
            entry = _entry_from_dir(child, source="registry", root=root)
            if entry:
                by_id[entry.effect_id] = entry

    if include_staging:
        staging = staging_root() if repo is None else (root / "assets" / "staging")
        if staging.is_dir():
            for child in sorted(staging.iterdir()):
                if not child.is_dir():
                    continue
                if not (child / "effect_spec.json").is_file():
                    continue
                entry = _entry_from_dir(child, source="staging", root=root)
                if not entry:
                    continue
                existing = by_id.get(entry.effect_id)
                if existing is None:
                    by_id[entry.effect_id] = entry
                elif existing.honest_gate != "honest" and entry.honest_gate == "honest":
                    # Staging capture can unlock Assign before registry manifest refresh.
                    by_id[entry.effect_id] = entry

    entries = list(by_id.values())
    if batch_id:
        entries = [e for e in entries if e.batch_id == batch_id]
    entries.sort(key=lambda e: (0 if e.source == "registry" else 1, e.effect_id))
    return entries


def list_batch_ids(entries: Iterable[EffectBrowseEntry] | None = None) -> list[str]:
    rows = list(entries) if entries is not None else list_effect_entries()
    seen: list[str] = []
    for e in rows:
        if e.batch_id and e.batch_id not in seen:
            seen.append(e.batch_id)
    return seen


def honesty_label(gate: str | None) -> str:
    key = (gate or "").strip() or "missing"
    return HONESTY_GLYPH.get(key, HONESTY_GLYPH["missing"])


def get_effect_entry(effect_id: str, *, repo: Path | None = None) -> EffectBrowseEntry | None:
    for e in list_effect_entries(repo=repo):
        if e.effect_id == effect_id:
            return e
    return None
