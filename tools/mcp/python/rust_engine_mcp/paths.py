"""Repo paths and Blender discovery — env > config.local.json > config.defaults.json."""

from __future__ import annotations

import json
import os
import shutil
from functools import lru_cache
from pathlib import Path


def _mcp_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _load_json(path: Path) -> dict:
    if not path.is_file():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


@lru_cache(maxsize=1)
def repo_root() -> Path:
    env = os.environ.get("RUST_ENGINE_REPO", "").strip()
    if env:
        return Path(env).resolve()
    defaults = _load_json(_mcp_root() / "config.defaults.json")
    local = _load_json(_mcp_root() / "config.local.json")
    # The committed default is a Windows checkout path; on another machine (CI runner, other
    # clone) it does not exist, so fall back to the repo this package lives in.
    for raw in (local.get("repo"), defaults.get("repo")):
        if raw and Path(raw).is_dir():
            return Path(raw).resolve()
    return Path(__file__).resolve().parents[4]


@lru_cache(maxsize=1)
def blender_exe() -> Path:
    env = os.environ.get("BLENDER_EXE", "").strip()
    if env:
        p = Path(env)
        if p.is_file():
            return p.resolve()
    defaults = _load_json(_mcp_root() / "config.defaults.json")
    local = _load_json(_mcp_root() / "config.local.json")
    raw = local.get("blender_exe") or defaults.get("blender_exe") or ""
    if raw:
        p = Path(raw)
        if p.is_file():
            return p.resolve()
    on_path = shutil.which("blender")
    if on_path:
        return Path(on_path).resolve()
    raise FileNotFoundError(
        "Blender not found. Set BLENDER_EXE, tools/mcp/config.local.json blender_exe, or put blender on PATH"
    )


def repo_rel(path: str | Path | None) -> str | None:
    """Repo-relative POSIX path for anything written to committed files (no machine paths).

    Paths outside the repo come back unchanged; ``None`` / empty stay as they are.
    """
    if not path:
        return path  # type: ignore[return-value]
    p = Path(path)
    if not p.is_absolute():
        return str(path).replace("\\", "/")
    try:
        return p.resolve().relative_to(repo_root().resolve()).as_posix()
    except ValueError:
        return str(path)


def resolve_under_root(root: Path, rel: str | Path, *, label: str = "path") -> Path:
    """Resolve ``rel`` under ``root``. Reject ``..`` and paths that leave ``root``."""
    raw = str(rel).replace("\\", "/").strip()
    if not raw or raw in {".", ".."}:
        raise ValueError(f"{label} escapes root: {rel!r}")
    base = Path(root).resolve()
    candidate = Path(raw)
    dest = candidate.resolve() if candidate.is_absolute() else (base / candidate).resolve()
    if dest == base or not dest.is_relative_to(base):
        raise ValueError(f"{label} escapes root: {rel!r}")
    return dest


def staging_root() -> Path:
    return repo_root() / "assets" / "staging"


def jobs_root() -> Path:
    p = repo_root() / "tools" / "mcp" / "jobs"
    p.mkdir(parents=True, exist_ok=True)
    return p


def art_pipeline_log_dir() -> Path:
    p = repo_root() / "debug_runs" / "art_pipeline"
    p.mkdir(parents=True, exist_ok=True)
    return p


def schemas_dir() -> Path:
    return repo_root() / "tools" / "mcp" / "schemas"


def blender_scripts_dir() -> Path:
    return repo_root() / "tools" / "mcp" / "blender" / "scripts"
