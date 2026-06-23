"""OPS crash exporter + atomic witness tests."""

from __future__ import annotations

import json
import subprocess
import sys

from rust_engine_mcp.ops_atomic_witness import compute_body_hash, write_witness_atomic
from rust_engine_mcp.ops_crash_exporter import (
    PROMETHEUS_REL,
    TRIAGE_LIVE_REL,
    build_triage_live,
    run_crash_scan,
    write_prometheus_metrics,
    write_triage_witness,
)
from rust_engine_mcp.paths import repo_root


def test_run_crash_scan_schema():
    body = run_crash_scan(record_events=False)
    assert body["ok"] is True
    assert body["schema"] == "ops_crash_scan_v1"
    assert "slip_ups" in body
    assert "process_count" in body


def test_prometheus_export():
    scan = run_crash_scan(record_events=False)
    rel = write_prometheus_metrics(scan)
    path = repo_root() / rel
    assert path.is_file()
    text = path.read_text(encoding="utf-8")
    assert "rust_engine_ops_crash_alerts_total" in text


def test_atomic_witness_hash_chain():
    rel = "debug_runs/agent_ops/test_atomic_witness.json"
    first = write_witness_atomic(
        rel,
        {"schema": "test_v1", "n": 1},
        actor="ops_crash_exporter",
        profile="TEST",
        source_system="test",
        glyph="✅",
    )
    second = write_witness_atomic(
        rel,
        {"schema": "test_v1", "n": 2},
        actor="ops_crash_exporter",
        profile="TEST",
        source_system="test",
    )
    assert first["content_hash"] != second["content_hash"]
    disk = json.loads((repo_root() / rel).read_text(encoding="utf-8"))
    assert disk["previous_hash"] == first["content_hash"]
    assert compute_body_hash({"schema": "test_v1", "n": 2}, previous_hash=first["content_hash"]) == disk["content_hash"]


def test_build_triage_live():
    body = build_triage_live(window_hours=24)
    assert body["schema"] == "ops_triage_live_v1"
    assert body["ignore_dcc_status_bar"] is True
    assert "blockers" in body
    assert "metrics" in body


def test_write_triage_witness():
    body = write_triage_witness(window_hours=24)
    assert body.get("ok") is True
    path = repo_root() / TRIAGE_LIVE_REL
    assert path.is_file()
    disk = json.loads(path.read_text(encoding="utf-8"))
    assert disk["schema"] == "ops_triage_live_v1"
    prom = repo_root() / PROMETHEUS_REL
    assert prom.is_file()


def test_cli_ops_crash_scan():
    proc = subprocess.run(
        [sys.executable, "-m", "rust_engine_mcp.cli", "ops-crash-scan"],
        cwd=repo_root() / "tools/mcp/python",
        capture_output=True,
        text=True,
        check=False,
    )
    assert proc.returncode == 0, proc.stderr or proc.stdout
    body = json.loads(proc.stdout)
    assert body.get("ok") is True
