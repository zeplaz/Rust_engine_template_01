"""Atomic hash-locked witness writes — single-writer, API-approved actors only."""

from __future__ import annotations

import hashlib
import json
import os
import tempfile
import time
from pathlib import Path
from typing import Any

from .paths import repo_root

DEFAULT_HASH_LOCK = "granfina_hash_lock_2026"
APPROVED_ACTORS = frozenset(
    {
        "ops_crash_daemon",
        "ops_crash_exporter",
        "operations-intelligence",
        "mcp:ops_dashboard_snapshot",
        "mcp:ops_triage_refresh",
        "witness_honesty_lib",
    }
)

_LOCK_SUFFIX = ".hashlock"


def _sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def hash_lock_secret() -> str:
    return os.environ.get("RUST_ENGINE_OPS_HASH_LOCK", DEFAULT_HASH_LOCK)


def compute_body_hash(body: dict[str, Any], *, previous_hash: str = "", lock: str | None = None) -> str:
    lock = lock or hash_lock_secret()
    payload = {k: v for k, v in body.items() if k not in ("content_hash", "previous_hash")}
    canonical = json.dumps(payload, sort_keys=True, separators=(",", ":"))
    return _sha256_text(f"{previous_hash}|{lock}|{canonical}")


def _lock_path(target: Path) -> Path:
    return target.with_suffix(target.suffix + _LOCK_SUFFIX)


def acquire_write_lock(target: Path, *, actor: str, timeout_sec: float = 5.0) -> None:
    if actor not in APPROVED_ACTORS:
        raise PermissionError(f"actor not approved for atomic witness write: {actor}")
    lock = _lock_path(target)
    lock.parent.mkdir(parents=True, exist_ok=True)
    deadline = time.time() + timeout_sec
    while time.time() < deadline:
        try:
            fd = os.open(str(lock), os.O_CREAT | os.O_EXCL | os.O_WRONLY)
            with os.fdopen(fd, "w", encoding="utf-8") as fh:
                fh.write(json.dumps({"actor": actor, "pid": os.getpid(), "ts": time.time()}))
            return
        except FileExistsError:
            time.sleep(0.05)
    raise TimeoutError(f"hash lock busy: {lock}")


def release_write_lock(target: Path) -> None:
    lock = _lock_path(target)
    try:
        lock.unlink(missing_ok=True)
    except OSError:
        pass


def read_previous_hash(target: Path) -> str:
    if not target.is_file():
        return ""
    try:
        data = json.loads(target.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError):
        return ""
    if isinstance(data, dict):
        return str(data.get("content_hash") or data.get("current_hash") or "")
    return ""


def write_witness_atomic(
    rel_path: str,
    body: dict[str, Any],
    *,
    actor: str,
    profile: str,
    source_system: str,
    glyph: str | None = None,
) -> dict[str, Any]:
    """Write JSON witness atomically with hash chain + _agent_meta envelope."""
    target = repo_root() / rel_path
    target.parent.mkdir(parents=True, exist_ok=True)
    previous = read_previous_hash(target)
    lock = hash_lock_secret()
    content_hash = compute_body_hash(body, previous_hash=previous, lock=lock)
    out = dict(body)
    out["previous_hash"] = previous
    out["content_hash"] = content_hash
    out["hash_lock_id"] = lock[:12] + "…"
    if glyph:
        out["glyph_chain"] = glyph
    meta = {
        "schema": "debug_run_envelope_v1",
        "written_at_epoch_secs": int(time.time()),
        "profile": profile,
        "source_system": source_system,
        "relative_path": rel_path,
        "agent": actor,
        "content_hash": content_hash,
    }
    out["_agent_meta"] = meta
    acquire_write_lock(target, actor=actor)
    try:
        fd, tmp = tempfile.mkstemp(dir=str(target.parent), suffix=".tmp")
        try:
            with os.fdopen(fd, "w", encoding="utf-8") as fh:
                fh.write(json.dumps(out, indent=2))
                fh.write("\n")
            os.replace(tmp, target)
        finally:
            if os.path.exists(tmp):
                os.unlink(tmp)
    finally:
        release_write_lock(target)
    return {"ok": True, "path": rel_path, "content_hash": content_hash, "glyph": glyph}


def append_jsonl_atomic(rel_path: str, row: dict[str, Any], *, actor: str) -> None:
    """Append one JSONL row under hash lock (crash event ledger)."""
    if actor not in APPROVED_ACTORS:
        raise PermissionError(f"actor not approved: {actor}")
    target = repo_root() / rel_path
    target.parent.mkdir(parents=True, exist_ok=True)
    acquire_write_lock(target, actor=actor)
    try:
        line = json.dumps(row, separators=(",", ":")) + "\n"
        with target.open("a", encoding="utf-8") as fh:
            fh.write(line)
    finally:
        release_write_lock(target)
