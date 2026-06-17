"""MCP-WIT-020..024 — witness_honesty_lib + ops hooks."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import pytest

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.witness_honesty_lib import (
    OPS_WITNESS_REL,
    build_integrity_cache,
    classify_honest_gate_v2,
    refresh_mcp_witness_integrity_ops_witness,
    run_post_build_hook,
)


def test_build_integrity_cache_has_counts() -> None:
    cache = build_integrity_cache(compression_level=3)
    assert int(cache.get("fail_count") or 0) > 0
    assert "by_file" in cache
    assert int(cache.get("queue_contradiction_count") or 0) >= 3


def test_classify_inflated_green_lg4() -> None:
    rel = "debug_runs/landscape_grammar_lg4_preview_live.json"
    path = repo_root() / rel
    if not path.is_file():
        pytest.skip("lg4 witness missing")
    body = json.loads(path.read_text(encoding="utf-8"))
    cache = build_integrity_cache()
    summary = {"green": True, "task_id": body.get("gate"), "_witness_rel": rel}
    gate = classify_honest_gate_v2(rel, body, summary, cache)
    assert gate == "inflated_green"


def test_run_post_build_hook_warn_mode(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("RUST_ENGINE_WITNESS_INTEGRITY_ENFORCE", raising=False)
    body = run_post_build_hook()
    assert body.get("hook") == "post_build"
    assert body.get("exit_code") == 0
    assert body.get("enforce") is False
    assert (repo_root() / OPS_WITNESS_REL).is_file()


def test_run_post_build_hook_enforce_fails(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("RUST_ENGINE_WITNESS_INTEGRITY_ENFORCE", "1")
    body = run_post_build_hook(enforce=True)
    assert body.get("exit_code") == 1
    assert body.get("enforce") is True


def test_ops_witness_integrity_witness_written() -> None:
    body = refresh_mcp_witness_integrity_ops_witness()
    assert body.get("gate") == "MCP-WIT-024"
    assert "hooks" in body
    assert (repo_root() / OPS_WITNESS_REL).is_file()


def test_witness_honesty_lib_cli_run_hook() -> None:
    import os

    script = repo_root() / "tools/orchestrator/scripts/witness_honesty_lib.py"
    env = os.environ.copy()
    env["PYTHONPATH"] = str(repo_root() / "tools/mcp/python")
    proc = subprocess.run(
        [sys.executable, str(script), "run-hook"],
        cwd=repo_root(),
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload.get("hook") == "post_build"
