"""VSS-T4 residual — deterministic staging preview frames for EffectSpec honesty.

Capture ladder (no GPU / operator theater, no invented VFX pixels):

  validate_effect_spec → pack hash verify → staging_pack_digest frames
  → capture_hash → honest_gate=honest → Assign unblocked

Frames are SHA-tilings of ``pack_hash|frame_idx|seed`` — reproducible contact
tiles for the APS preview ladder, not live particle screenshots.
"""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

from .effect_promote import (
    PROMOTE_MANIFEST,
    REFERENCE_BATCH_ID,
    REFERENCE_SPEC_RELS,
    REGISTRY_REL,
    WITNESS_REL,
    _load_pack_manifest,
    _require_rules_check,
    _require_validate,
    _resolve_spec,
    _sha256_file,
    _utc_now,
    _verify_staging_hashes,
    pack_effect_spec,
    refresh_artist_vfx_pipeline_witness,
)
from .paths import repo_root, staging_root

CAPTURE_KIND = "staging_pack_digest"
FRAMES_DIR = "preview_frames"
FRAMES_MANIFEST = "frames_manifest.json"
# Cap ladder frames — full preview.frames is sandbox theater, not G4 honesty.
FRAME_CAP = 4
FRAME_SIZE = 64


def _frame_count(spec: dict[str, Any]) -> int:
    preview = spec.get("preview") if isinstance(spec.get("preview"), dict) else {}
    requested = int(preview.get("frames") or 1)
    return max(1, min(FRAME_CAP, requested))


def _frame_digest(pack_hash: str, frame_idx: int, seed: str) -> str:
    payload = f"{pack_hash}|{frame_idx}|{seed}|{CAPTURE_KIND}".encode("utf-8")
    return hashlib.sha256(payload).hexdigest()


def _write_digest_png(path: Path, digest_hex: str, *, size: int = FRAME_SIZE) -> str:
    """Write a deterministic PNG from hex digest bytes (pack truth tile, not VFX art)."""
    try:
        from PIL import Image
    except ImportError as exc:  # pragma: no cover
        raise ImportError("Install Pillow: pip install Pillow") from exc

    raw = bytes.fromhex(digest_hex)
    if not raw:
        raise ValueError("empty digest")
    img = Image.new("RGB", (size, size))
    pixels: list[tuple[int, int, int]] = []
    for i in range(size * size):
        b = raw[i % len(raw)]
        pixels.append((b, (b * 3) % 256, (b * 7) % 256))
    img.putdata(pixels)
    path.parent.mkdir(parents=True, exist_ok=True)
    img.save(path, format="PNG")
    return _sha256_file(path)


def build_preview_witness(
    *,
    effect_id: str,
    staging_rel: str,
    frames_captured: int,
    capture_hash: str,
    promoted_rel: str = "",
    honest_gate: str = "honest",
) -> dict[str, Any]:
    body: dict[str, Any] = {
        "witness_schema": "artist_vfx_preview_witness_v1",
        "witness_path": WITNESS_REL,
        "preview_job_id": f"{effect_id}_preview_capture",
        "frames_captured": frames_captured,
        "capture_hash": capture_hash,
        "staging_path": staging_rel,
        "honest_gate": honest_gate,
        "captured_at_utc": _utc_now(),
        "toolchain": {
            "mcp_tool": "effect_preview_capture",
            "cli_parity": True,
            "blender_present": False,
        },
    }
    if promoted_rel:
        body["promoted_path"] = promoted_rel
    return body


def _write_frames(
    staging_dir: Path,
    *,
    pack_hash: str,
    seed: str,
    frame_count: int,
) -> tuple[list[dict[str, Any]], str]:
    frames_dir = staging_dir / FRAMES_DIR
    if frames_dir.is_dir():
        for old in frames_dir.glob("capture_*.png"):
            old.unlink()
    frames_dir.mkdir(parents=True, exist_ok=True)

    rows: list[dict[str, Any]] = []
    digests: list[str] = []
    for idx in range(frame_count):
        src = _frame_digest(pack_hash, idx, seed)
        name = f"capture_{idx:03d}.png"
        dest = frames_dir / name
        file_hash = _write_digest_png(dest, src)
        digests.append(file_hash)
        rows.append(
            {
                "index": idx,
                "path": f"{FRAMES_DIR}/{name}",
                "source_digest": src,
                "sha256": file_hash,
                "capture_kind": CAPTURE_KIND,
            }
        )
    capture_hash = hashlib.sha256("|".join(digests).encode("utf-8")).hexdigest()
    return rows, capture_hash


def load_staging_preview_witness(staging_dir: Path) -> dict[str, Any] | None:
    manifest_path = staging_dir / FRAMES_DIR / FRAMES_MANIFEST
    if not manifest_path.is_file():
        return None
    try:
        body = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    pw = body.get("preview_witness") if isinstance(body, dict) else None
    return dict(pw) if isinstance(pw, dict) else None


def capture_effect_preview(
    spec_path: str | Path,
    *,
    pack_first: bool = True,
    force: bool = False,
    write_registry: bool = True,
) -> dict[str, Any]:
    """Capture deterministic staging frames and set honest_gate=honest on manifests."""
    path = _resolve_spec(spec_path)
    if pack_first:
        pack_effect_spec(path, force=force)
    spec = _require_validate(path)
    _require_rules_check(spec, path=path)

    effect_id = str(spec["effect_id"])
    batch_id = str(spec.get("batch_id") or "")
    seed = str((spec.get("rules_check") or {}).get("seed") or spec.get("seed") or "0")
    staging_dir = staging_root() / effect_id
    pack_manifest = _load_pack_manifest(staging_dir)
    _verify_staging_hashes(staging_dir, pack_manifest)
    pack_hash = str(pack_manifest.get("pack_hash") or "")
    if not pack_hash:
        raise ValueError(f"pack_manifest missing pack_hash for {effect_id}")

    frame_count = _frame_count(spec)
    rows, capture_hash = _write_frames(
        staging_dir, pack_hash=pack_hash, seed=seed, frame_count=frame_count
    )
    staging_rel = f"assets/staging/{effect_id}/"
    registry_dir = repo_root() / REGISTRY_REL / effect_id
    promoted_rel = f"{REGISTRY_REL}/{effect_id}/" if (registry_dir / PROMOTE_MANIFEST).is_file() else ""

    pw = build_preview_witness(
        effect_id=effect_id,
        staging_rel=staging_rel,
        frames_captured=len(rows),
        capture_hash=capture_hash,
        promoted_rel=promoted_rel,
        honest_gate="honest",
    )

    frames_body = {
        "schema": "effect_preview_frames_manifest_v1",
        "effect_id": effect_id,
        "batch_id": batch_id,
        "capture_kind": CAPTURE_KIND,
        "pack_hash": pack_hash,
        "seed": seed,
        "captured_at_utc": _utc_now(),
        "frames": rows,
        "capture_hash": capture_hash,
        "preview_witness": pw,
        "_note": "Deterministic staging_pack_digest tiles — not GPU/operator theater pixels",
    }
    frames_path = staging_dir / FRAMES_DIR / FRAMES_MANIFEST
    frames_path.write_text(json.dumps(frames_body, indent=2) + "\n", encoding="utf-8")

    # Convenience alias for effect_registry_browse._thumb_for
    thumb_src = staging_dir / FRAMES_DIR / "capture_000.png"
    if thumb_src.is_file():
        import shutil

        shutil.copy2(thumb_src, staging_dir / "capture_000.png")

    if write_registry and (registry_dir / PROMOTE_MANIFEST).is_file():
        _sync_registry_capture(registry_dir, staging_dir, pw)

    return {
        "effect_id": effect_id,
        "batch_id": batch_id,
        "phase": "preview_capture",
        "capture_kind": CAPTURE_KIND,
        "frames_captured": len(rows),
        "capture_hash": capture_hash,
        "staging_frames": f"{staging_rel}{FRAMES_DIR}/",
        "preview_witness": pw,
        "honest_gate": "honest",
    }


def _sync_registry_capture(registry_dir: Path, staging_dir: Path, pw: dict[str, Any]) -> None:
    import shutil

    dest_frames = registry_dir / FRAMES_DIR
    if dest_frames.is_dir():
        shutil.rmtree(dest_frames)
    shutil.copytree(staging_dir / FRAMES_DIR, dest_frames)
    thumb = staging_dir / "capture_000.png"
    if thumb.is_file():
        shutil.copy2(thumb, registry_dir / "capture_000.png")

    promote_path = registry_dir / PROMOTE_MANIFEST
    body = json.loads(promote_path.read_text(encoding="utf-8"))
    body["preview_witness"] = {
        **pw,
        "promoted_path": f"{REGISTRY_REL}/{registry_dir.name}/",
    }
    promote_path.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")

    # Keep registry effect_spec free of preview_witness (schema-gated); honesty lives on manifest.


def capture_reference_batch(
    batch_id: str = REFERENCE_BATCH_ID,
    *,
    pack_first: bool = True,
    force: bool = False,
) -> dict[str, Any]:
    root = repo_root()
    specs: list[Path] = []
    for rel in REFERENCE_SPEC_RELS:
        path = root / rel
        if not path.is_file():
            continue
        from .schemas import load_json_file

        data = load_json_file(path)
        if str(data.get("batch_id") or "") == batch_id:
            specs.append(path)
    if not specs and batch_id == REFERENCE_BATCH_ID:
        specs = [_resolve_spec(rel) for rel in REFERENCE_SPEC_RELS]
    if not specs:
        raise FileNotFoundError(f"No EffectSpec examples found for batch_id={batch_id!r}")

    results = [
        capture_effect_preview(p, pack_first=pack_first, force=force, write_registry=True)
        for p in specs
    ]
    return {
        "batch_id": batch_id,
        "phase": "batch_preview_capture",
        "count": len(results),
        "effects": results,
        "all_honest": all(r.get("honest_gate") == "honest" for r in results),
    }


def effect_preview_capture(
    *,
    spec_path: str = "",
    batch_id: str = "",
    pack_first: bool = True,
    force: bool = False,
    write_witness: bool = True,
) -> dict[str, Any]:
    """Shared MCP/CLI entry — capture one spec or the reference batch."""
    if batch_id or not spec_path:
        body = capture_reference_batch(
            batch_id or REFERENCE_BATCH_ID,
            pack_first=pack_first,
            force=force,
        )
    else:
        body = capture_effect_preview(
            spec_path, pack_first=pack_first, force=force, write_registry=True
        )
    if write_witness:
        body["witness"] = refresh_artist_vfx_pipeline_witness()
    return body


def verify_capture_hash(staging_dir: Path) -> bool:
    """Recompute capture_hash from on-disk frames; True when matches frames_manifest."""
    manifest_path = staging_dir / FRAMES_DIR / FRAMES_MANIFEST
    if not manifest_path.is_file():
        return False
    body = json.loads(manifest_path.read_text(encoding="utf-8"))
    expected = str(body.get("capture_hash") or "")
    frames = body.get("frames") or []
    digests: list[str] = []
    for row in frames:
        if not isinstance(row, dict):
            return False
        rel = str(row.get("path") or "")
        path = staging_dir / rel
        if not path.is_file():
            return False
        digests.append(_sha256_file(path))
    actual = hashlib.sha256("|".join(digests).encode("utf-8")).hexdigest()
    return bool(expected) and actual == expected
