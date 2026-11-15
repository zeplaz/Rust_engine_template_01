"""AUD-03 / plan S6 — sound_cue_groups_v1 validator + validate-sound-cues CLI."""

from __future__ import annotations

import json
import math
import struct
import subprocess
import sys
import wave
from pathlib import Path

import pytest

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators import run_validator
from rust_engine_mcp.validators.sound_cue import (
    PEAK_SPREAD_MAX_DB,
    validate_sound_cue_groups_data,
    validate_sound_cue_path,
)

FIXTURES = Path(__file__).resolve().parent / "fixtures" / "sound_cue"
EXAMPLE = repo_root() / "tools/mcp/schemas/examples/sound_cue_groups_engine_v1.json"


def _write_pcm16_mono(path: Path, *, peak: int, n_frames: int = 800, rate: int = 44100) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    amp = max(1, min(32767, peak))
    samples = struct.pack("<h", amp) * n_frames
    with wave.open(str(path), "wb") as wf:
        wf.setnchannels(1)
        wf.setsampwidth(2)
        wf.setframerate(rate)
        wf.writeframes(samples)


def _manifest(tmp_path: Path, cues: list[dict]) -> Path:
    doc = {"schema_version": "sound_cue_groups_v1", "cues": cues}
    path = tmp_path / "cues.json"
    path.write_text(json.dumps(doc), encoding="utf-8")
    return path


@pytest.fixture(scope="module", autouse=True)
def _ensure_fixtures() -> None:
    if (FIXTURES / "v0_peak_2000.wav").is_file():
        return
    _write_pcm16_mono(FIXTURES / "v0_peak_2000.wav", peak=2000)
    # Within 1.5 dB of v0 (spread ~1.0 dB)
    _write_pcm16_mono(FIXTURES / "v1_peak_2250.wav", peak=2250)
    # ~6 dB hotter — fails spread gate
    _write_pcm16_mono(FIXTURES / "v_hot_peak_4000.wav", peak=4000)


def test_validate_sound_cues_command_listed() -> None:
    r = subprocess.run(
        [sys.executable, "-m", "rust_engine_mcp.cli", "commands", "--grep", "validate-sound-cues"],
        cwd=str(repo_root() / "tools/mcp/python"),
        capture_output=True,
        text=True,
        check=False,
    )
    assert r.returncode == 0
    assert "validate-sound-cues" in r.stdout


def test_engine_example_passes() -> None:
    report = validate_sound_cue_path(EXAMPLE, compression_level=4)
    assert report.status == "passed"


def test_run_validator_sound_cue_parity() -> None:
    rel = EXAMPLE.relative_to(repo_root()).as_posix()
    report = run_validator("sound_cue", rel, compression_level=3)
    assert report.status == "passed"


def test_green_fixture_group(tmp_path: Path) -> None:
    root = tmp_path / "wav"
    _write_pcm16_mono(root / "a.wav", peak=3000)
    _write_pcm16_mono(root / "b.wav", peak=3300)
    manifest = _manifest(
        tmp_path,
        [
            {
                "cue_id": "test.cue",
                "repeated_oneshot": True,
                "variants": [
                    {"tag": "test.cue.v0", "path": str(root / "a.wav")},
                    {"tag": "test.cue.v1", "path": str(root / "b.wav")},
                ],
            }
        ],
    )
    issues = validate_sound_cue_groups_data(json.loads(manifest.read_text()), repo=tmp_path)
    assert not issues


def test_fails_single_variant_repeated_oneshot(tmp_path: Path) -> None:
    root = tmp_path / "wav"
    _write_pcm16_mono(root / "only.wav", peak=1000)
    data = {
        "cues": [
            {
                "cue_id": "solo",
                "repeated_oneshot": True,
                "variants": [{"tag": "solo", "path": str(root / "only.wav")}],
            }
        ]
    }
    issues = validate_sound_cue_groups_data(data, repo=tmp_path)
    assert any(i.signature == "sound_cue_min_variants" for i in issues)


def test_fails_duplicate_paths(tmp_path: Path) -> None:
    root = tmp_path / "wav"
    _write_pcm16_mono(root / "dup.wav", peak=2000)
    data = {
        "cues": [
            {
                "cue_id": "dup",
                "repeated_oneshot": True,
                "variants": [
                    {"tag": "dup.v0", "path": str(root / "dup.wav")},
                    {"tag": "dup.v1", "path": str(root / "dup.wav")},
                ],
            }
        ]
    }
    issues = validate_sound_cue_groups_data(data, repo=tmp_path)
    assert any(i.signature == "sound_cue_duplicate_path" for i in issues)


def test_fails_missing_path(tmp_path: Path) -> None:
    data = {
        "cues": [
            {
                "cue_id": "missing",
                "repeated_oneshot": True,
                "variants": [
                    {"tag": "m.v0", "path": str(tmp_path / "nope.wav")},
                    {"tag": "m.v1", "path": str(tmp_path / "also_nope.wav")},
                ],
            }
        ]
    }
    issues = validate_sound_cue_groups_data(data, repo=tmp_path)
    assert any(i.signature == "sound_cue_missing_path" for i in issues)


def test_fails_peak_spread(tmp_path: Path) -> None:
    v0 = FIXTURES / "v0_peak_2000.wav"
    v_hot = FIXTURES / "v_hot_peak_4000.wav"
    data = {
        "cues": [
            {
                "cue_id": "loud",
                "repeated_oneshot": True,
                "variants": [
                    {"tag": "loud.v0", "path": str(v0)},
                    {"tag": "loud.v1", "path": str(v_hot)},
                ],
            }
        ]
    }
    issues = validate_sound_cue_groups_data(data, repo=repo_root())
    assert any(i.signature == "sound_cue_peak_spread" for i in issues)
    spread_issue = next(i for i in issues if i.signature == "sound_cue_peak_spread")
    assert str(PEAK_SPREAD_MAX_DB) in spread_issue.hint or "dB" in spread_issue.hint


def test_committed_fixtures_within_spread() -> None:
    from rust_engine_mcp.validators.sound_cue import _wav_peak_dbfs

    v0 = _wav_peak_dbfs(FIXTURES / "v0_peak_2000.wav")
    v1 = _wav_peak_dbfs(FIXTURES / "v1_peak_2250.wav")
    assert math.isfinite(v0) and math.isfinite(v1)
    assert abs(v0 - v1) <= PEAK_SPREAD_MAX_DB + 0.05