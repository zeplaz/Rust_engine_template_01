"""AUD-03 / plan S6 — studio QC for sound cue variant groups (stdlib wave only)."""

from __future__ import annotations

import math
import struct
import wave
from pathlib import Path

from rust_engine_mcp import schemas
from rust_engine_mcp.paths import repo_root

from .report import KnownFix, ValidationIssue, ValidationReport

PEAK_SPREAD_MAX_DB = 1.5
TASK_ID = "REF-BOOKS-S6-AUD-03-001"


def _wav_peak_dbfs(path: Path) -> float:
    with wave.open(str(path), "rb") as wf:
        n_channels = wf.getnchannels()
        sampwidth = wf.getsampwidth()
        n_frames = wf.getnframes()
        if n_frames == 0 or n_channels < 1:
            return float("-inf")
        raw = wf.readframes(n_frames)
    if sampwidth == 1:
        samples = struct.unpack(f"{len(raw)}B", raw)
        peak = max(abs(int(s) - 128) for s in samples)
        full = 128.0
    elif sampwidth == 2:
        count = len(raw) // 2
        if count == 0:
            return float("-inf")
        samples = struct.unpack(f"<{count}h", raw)
        peak = max(abs(s) for s in samples)
        full = 32768.0
    elif sampwidth == 4:
        count = len(raw) // 4
        if count == 0:
            return float("-inf")
        samples = struct.unpack(f"<{count}i", raw)
        peak = max(abs(s) for s in samples)
        full = 2147483648.0
    else:
        raise ValueError(f"unsupported WAV sample width {sampwidth} bytes")
    if peak <= 0:
        return float("-inf")
    return 20.0 * math.log10(peak / full)


def _resolve_wav(repo: Path, rel: str) -> Path:
    p = Path(rel)
    if not p.is_absolute():
        p = repo / p
    return p.resolve()


def validate_sound_cue_groups_data(
    data: dict,
    *,
    repo: Path | None = None,
) -> list[ValidationIssue]:
    """Rule checks after JSON schema — paths resolved against repo root."""
    root = repo or repo_root()
    issues: list[ValidationIssue] = []
    for cue in data.get("cues") or []:
        cue_id = str(cue.get("cue_id") or "")
        repeated = cue.get("repeated_oneshot", True)
        variants = cue.get("variants") or []
        paths_seen: list[str] = []
        peaks_db: list[tuple[str, float]] = []

        if repeated and len(variants) < 2:
            issues.append(
                ValidationIssue(
                    kind="CueVariantCount",
                    severity="error",
                    field=cue_id,
                    hint=f"repeated one-shot cue {cue_id!r} needs ≥2 variants (AUD-03)",
                    signature="sound_cue_min_variants",
                )
            )

        for entry in variants:
            tag = str(entry.get("tag") or "")
            rel = str(entry.get("path") or "")
            norm = rel.replace("\\", "/")
            if norm in paths_seen:
                issues.append(
                    ValidationIssue(
                        kind="DuplicateVariantPath",
                        severity="error",
                        file=rel,
                        field=cue_id,
                        symbol=tag,
                        hint=f"duplicate WAV path {rel!r} in cue {cue_id!r}",
                        signature="sound_cue_duplicate_path",
                    )
                )
            paths_seen.append(norm)

            wav_path = _resolve_wav(root, rel)
            if not wav_path.is_file():
                issues.append(
                    ValidationIssue(
                        kind="MissingWav",
                        severity="error",
                        file=rel,
                        field=cue_id,
                        symbol=tag,
                        hint=f"WAV not found: {wav_path}",
                        signature="sound_cue_missing_path",
                    )
                )
                continue
            try:
                db = _wav_peak_dbfs(wav_path)
            except (wave.Error, ValueError, OSError) as exc:
                issues.append(
                    ValidationIssue(
                        kind="WavReadError",
                        severity="error",
                        file=rel,
                        field=cue_id,
                        symbol=tag,
                        hint=str(exc),
                        signature="sound_cue_wav_read",
                    )
                )
                continue
            peaks_db.append((tag, db))

        finite = [db for _tag, db in peaks_db if math.isfinite(db)]
        if len(finite) >= 2:
            spread = max(finite) - min(finite)
            if spread > PEAK_SPREAD_MAX_DB + 1e-9:
                issues.append(
                    ValidationIssue(
                        kind="PeakLevelSpread",
                        severity="error",
                        field=cue_id,
                        hint=(
                            f"peak level spread {spread:.2f} dB exceeds {PEAK_SPREAD_MAX_DB} dB "
                            f"(tags: {', '.join(t for t, d in peaks_db if math.isfinite(d))})"
                        ),
                        signature="sound_cue_peak_spread",
                    )
                )
    return issues


def validate_sound_cue_path(path: Path, *, compression_level: int = 3) -> ValidationReport:
    issues: list[ValidationIssue] = []
    data: dict = {}
    try:
        data = schemas.load_json_file(path)
        schemas.validate_sound_cue_groups(data)
    except Exception as exc:  # noqa: BLE001
        issues.append(
            ValidationIssue(
                kind="SchemaInvalid",
                severity="error",
                file=str(path),
                symbol="sound_cue_groups_v1",
                hint=str(exc),
                signature="sound_cue_groups_v1",
            )
        )
    if not issues:
        issues.extend(validate_sound_cue_groups_data(data))

    errors = [i for i in issues if i.severity == "error"]
    status = "passed" if not errors else "failed"
    cue_count = len(data.get("cues") or []) if data else 0
    summary = (
        f"sound_cue_groups_v1 cues={cue_count} task={TASK_ID}"
        if status == "passed"
        else f"sound_cue_groups_v1 blocked: {errors[0].hint if errors else 'invalid'}"
    )
    known: list[KnownFix] = []
    if any(i.signature == "sound_cue_min_variants" for i in errors):
        known.append(
            KnownFix(
                signature="sound_cue_min_variants",
                fix="Add distinct variant WAVs (≥2) for repeated one-shots; set repeated_oneshot:false only for singleton cues.",
                confidence=0.95,
            )
        )
    if any(i.signature == "sound_cue_peak_spread" for i in errors):
        known.append(
            KnownFix(
                signature="sound_cue_peak_spread",
                fix=f"Normalize variant peaks so spread ≤ {PEAK_SPREAD_MAX_DB} dBFS across the cue group.",
                confidence=0.9,
            )
        )
    if any(i.signature == "sound_cue_duplicate_path" for i in errors):
        known.append(
            KnownFix(
                signature="sound_cue_duplicate_path",
                fix="Give each variant its own take file; duplicate paths defeat AUD-03 variation.",
                confidence=0.95,
            )
        )

    return ValidationReport(
        validator="mcp_schema",  # type: ignore[arg-type]
        status=status,  # type: ignore[arg-type]
        compression_level=compression_level,
        summary=summary,
        error_count=len(errors),
        warning_count=sum(1 for i in issues if i.severity == "warning"),
        errors=issues,
        known_fixes=known,
    ).compress(compression_level)