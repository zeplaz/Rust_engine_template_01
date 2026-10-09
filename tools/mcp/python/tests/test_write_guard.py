"""Pytest write guard — child-process writes under the repo root are rolled back."""

from __future__ import annotations

import os
import subprocess
import sys
import textwrap
from pathlib import Path

from rust_engine_mcp import test_write_guard as guard

_PY_ROOT = Path(__file__).resolve().parents[1]

_CHILD = textwrap.dedent(
    """
    import os, shutil, sys
    from pathlib import Path
    import rust_engine_mcp  # installs the guard from env
    repo = Path(sys.argv[1])
    outside = Path(sys.argv[2])
    (repo / "keep.json").write_text("mutated", encoding="utf-8")
    with open(repo / "log.jsonl", "a", encoding="utf-8") as fh:
        fh.write("appended\\n")
    (repo / "new_dir" / "deep").mkdir(parents=True)
    (repo / "new_dir" / "deep" / "fresh.json").write_text("{}", encoding="utf-8")
    tmp = repo / "atomic.json.tmp"
    tmp.write_text("atomic-new", encoding="utf-8")
    os.replace(tmp, repo / "atomic.json")
    (repo / "doomed.txt").unlink()
    shutil.rmtree(repo / "tree")
    (outside / "elsewhere.txt").write_text("outside", encoding="utf-8")
    """
)


def _seed(repo: Path) -> None:
    (repo / "keep.json").write_text("original", encoding="utf-8")
    (repo / "log.jsonl").write_text("line1\n", encoding="utf-8")
    (repo / "atomic.json").write_text("atomic-old", encoding="utf-8")
    (repo / "doomed.txt").write_text("doomed", encoding="utf-8")
    (repo / "tree").mkdir()
    (repo / "tree" / "a.txt").write_text("tree-a", encoding="utf-8")


def _run_child(repo: Path, outside: Path, ledger: Path, *, keep: bool = False) -> None:
    env = dict(os.environ)
    env[guard.ENV_LEDGER] = str(ledger)
    env["RUST_ENGINE_REPO"] = str(repo)
    env.pop(guard.ENV_KEEP, None)
    if keep:
        env[guard.ENV_KEEP] = "1"
    subprocess.run(
        [sys.executable, "-c", _CHILD, str(repo), str(outside)],
        cwd=_PY_ROOT,
        env=env,
        check=True,
        timeout=60,
    )


def test_child_writes_rolled_back(tmp_path: Path) -> None:
    repo, outside, ledger = tmp_path / "repo", tmp_path / "outside", tmp_path / "ledger"
    repo.mkdir()
    outside.mkdir()
    _seed(repo)
    _run_child(repo, outside, ledger)

    assert (repo / "keep.json").read_text(encoding="utf-8") == "mutated"
    restored = guard.restore(ledger)

    assert restored
    assert (repo / "keep.json").read_text(encoding="utf-8") == "original"
    assert (repo / "log.jsonl").read_text(encoding="utf-8") == "line1\n"
    assert (repo / "atomic.json").read_text(encoding="utf-8") == "atomic-old"
    assert not (repo / "atomic.json.tmp").exists()
    assert (repo / "doomed.txt").read_text(encoding="utf-8") == "doomed"
    assert (repo / "tree" / "a.txt").read_text(encoding="utf-8") == "tree-a"
    assert not (repo / "new_dir").exists()
    # Outside the repo root: untouched by the guard.
    assert (outside / "elsewhere.txt").read_text(encoding="utf-8") == "outside"


def test_keep_writes_opt_out(tmp_path: Path) -> None:
    repo, outside, ledger = tmp_path / "repo", tmp_path / "outside", tmp_path / "ledger"
    repo.mkdir()
    outside.mkdir()
    _seed(repo)
    _run_child(repo, outside, ledger, keep=True)

    assert guard.restore(ledger) == []
    assert (repo / "keep.json").read_text(encoding="utf-8") == "mutated"


def test_restore_missing_ledger_is_noop(tmp_path: Path) -> None:
    assert guard.restore(tmp_path / "nope") == []
