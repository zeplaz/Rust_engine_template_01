"""Pytest write guard — tests must not leave tracked repo files mutated.

Many MCP tests drive real witness writers (``debug_runs/*_live.json``), job status files,
staging assemblies, style packs and even ``src/dev`` docs. In a shared checkout that churn
lands in every fleet's ``git status`` and stale templates silently regress real files.

Mechanism: a ``sys.addaudithook`` hook sees every write-mode ``open`` / rename / remove /
mkdir under the repo root, and on the *first* touch of a path snapshots its original bytes
into a ledger directory (outside the repo). At session end the owner process restores every
snapshot (deleting files that did not exist and empty directories it created).

Child Python processes (``python -m rust_engine_mcp.cli …``, nested pytest) inherit the
ledger through ``RUST_ENGINE_TEST_WRITE_GUARD`` and install the same hook on
``rust_engine_mcp`` import, so their writes are rolled back too. Snapshots are claimed with
exclusive-create, so the first writer across all processes records the true original.

Opt out (intentional witness refresh): ``RUST_ENGINE_TEST_KEEP_WRITES=1``.
"""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import sys
import threading
from pathlib import Path

ENV_LEDGER = "RUST_ENGINE_TEST_WRITE_GUARD"
ENV_KEEP = "RUST_ENGINE_TEST_KEEP_WRITES"

_WRITE_FLAGS = os.O_WRONLY | os.O_RDWR | os.O_APPEND | os.O_CREAT | os.O_TRUNC
_SKIP_PARTS = ("__pycache__", ".pytest_cache", f"{os.sep}.git{os.sep}")

_state: dict[str, object] = {}
_local = threading.local()


def _norm(p: object) -> str | None:
    if isinstance(p, int):
        return None
    try:
        raw = os.fsdecode(os.fspath(p))  # type: ignore[arg-type]
    except TypeError:
        return None
    return os.path.normcase(os.path.abspath(raw))


def _in_scope(path: str) -> bool:
    root = _state.get("root")
    if not isinstance(root, str):
        return False
    if path != root and not path.startswith(root + os.sep):
        return False
    return not any(part in path for part in _SKIP_PARTS)


def _key(path: str) -> str:
    return hashlib.sha1(path.encode("utf-8")).hexdigest()


def _snapshot(path: str, kind: str = "file") -> None:
    seen = _state["seen"]
    assert isinstance(seen, set)
    if path in seen:
        return
    seen.add(path)
    ledger = Path(str(_state["ledger"]))
    key = _key(path)
    meta = ledger / f"{key}.json"
    try:
        fd = os.open(meta, os.O_CREAT | os.O_EXCL | os.O_WRONLY)
    except FileExistsError:
        return  # another process (or earlier hook) already holds the original
    existed = os.path.exists(path)
    is_dir = os.path.isdir(path)
    if kind == "file" and existed and not is_dir:
        shutil.copyfile(path, ledger / f"{key}.bin")
    record = {"path": path, "kind": kind, "existed": existed, "is_dir": is_dir}
    with os.fdopen(fd, "w", encoding="utf-8") as fh:
        json.dump(record, fh)


def _snapshot_tree(path: str) -> None:
    for dirpath, _dirs, files in os.walk(path):
        for name in files:
            _snapshot(os.path.normcase(os.path.join(dirpath, name)))


def _is_write_open(mode: object, flags: object) -> bool:
    if isinstance(mode, str):
        return any(c in mode for c in "wax+")
    return isinstance(flags, int) and bool(flags & _WRITE_FLAGS)


def _hook(event: str, args: tuple) -> None:
    if getattr(_local, "busy", False) or not _state.get("active"):
        return
    targets: list[tuple[str, str]] = []
    if event == "open":
        if len(args) >= 3 and _is_write_open(args[1], args[2]):
            targets.append((args[0], "file"))
    elif event == "os.rename":
        targets += [(args[0], "file"), (args[1], "file")]
    elif event in ("os.remove", "os.truncate"):
        targets.append((args[0], "file"))
    elif event == "os.mkdir":
        targets.append((args[0], "dir"))
    elif event == "shutil.rmtree":
        targets.append((args[0], "tree"))
    else:
        return
    _local.busy = True
    try:
        for raw, kind in targets:
            path = _norm(raw)
            if path is None or not _in_scope(path):
                continue
            if kind == "tree":
                if os.path.isdir(path):
                    _snapshot_tree(path)
            elif kind == "dir":
                if not os.path.exists(path):
                    _snapshot(path, "dir")
            else:
                _snapshot(path)
    except Exception:  # noqa: BLE001 — a guard must never break the code under test
        pass
    finally:
        _local.busy = False


def install(ledger: Path | str, repo: Path | str) -> None:
    """Idempotent per process; hooks cannot be removed, so ``active`` gates them."""
    Path(ledger).mkdir(parents=True, exist_ok=True)
    _state["ledger"] = str(ledger)
    _state["root"] = os.path.normcase(os.path.abspath(str(repo)))
    _state.setdefault("seen", set())
    _state["active"] = True
    if not _state.get("hooked"):
        sys.addaudithook(_hook)
        _state["hooked"] = True


def deactivate() -> None:
    _state["active"] = False


def install_from_env() -> bool:
    ledger = os.environ.get(ENV_LEDGER, "").strip()
    if not ledger or os.environ.get(ENV_KEEP, "").strip() == "1":
        return False
    from .paths import repo_root

    install(ledger, repo_root())
    return True


def restore(ledger: Path | str) -> list[str]:
    """Restore every snapshot in ``ledger``; returns repo paths that were rolled back."""
    ledger = Path(ledger)
    if not ledger.is_dir():
        return []
    was_active = _state.get("active")
    _state["active"] = False
    restored: list[str] = []
    dirs: list[str] = []
    try:
        for meta in sorted(ledger.glob("*.json")):
            try:
                rec = json.loads(meta.read_text(encoding="utf-8"))
            except (OSError, ValueError):
                continue
            path = rec["path"]
            if rec.get("kind") == "dir":
                dirs.append(path)
                continue
            blob = meta.with_suffix(".bin")
            try:
                if rec.get("existed") and blob.is_file():
                    with open(blob, "rb") as fh:
                        original = fh.read()
                    current = None
                    if os.path.isfile(path):
                        with open(path, "rb") as fh:
                            current = fh.read()
                    if current != original:
                        os.makedirs(os.path.dirname(path), exist_ok=True)
                        with open(path, "wb") as fh:
                            fh.write(original)
                        restored.append(path)
                elif not rec.get("existed") and os.path.isfile(path):
                    os.remove(path)
                    restored.append(path)
            except OSError:
                continue
        for d in sorted(dirs, key=len, reverse=True):
            try:
                os.rmdir(d)  # only succeeds when empty
            except OSError:
                pass
    finally:
        _state["active"] = was_active
    return restored
