"""OPS telemetry — dashboard, drift, run_events rollup."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

from rust_engine_mcp.ops_telemetry import (
    OPS_DASHBOARD_REL,
    build_ops_dashboard,
    scan_drift_instances,
    scan_run_events,
    summarize_github_ci_run,
    write_ops_dashboard_witness,
)
from rust_engine_mcp.paths import repo_root


def test_scan_run_events_schema():
    body = scan_run_events(window_hours=168)
    assert body["ok"] is True
    assert body["schema"] == "ops_run_events_rollup_v1"
    assert "metrics_tier1" in body
    assert "slip_ups" in body
    tier1 = body["metrics_tier1"]
    assert tier1["status"] in ("measured", "sparse")
    assert "ftr" in tier1


def test_scan_drift_instances_schema():
    body = scan_drift_instances()
    assert body["ok"] is True
    assert body["schema"] == "ops_drift_scan_v1"
    assert isinstance(body["instances"], list)
    assert "alert_count" in body


def test_build_ops_dashboard_merges_slip_ups():
    body = build_ops_dashboard(window_hours=24)
    assert body["schema"] == "ops_dashboard_v1"
    assert body["ok"] is True
    assert "run_events" in body
    assert "processes" in body
    assert "drift" in body
    assert "metrics_tier1" in body
    assert "grafana" in body
    assert isinstance(body["slip_ups"], list)
    ci = body["ci_lib_test"]
    assert ci["schema"] == "ops_ci_lib_test_v1"
    assert ci["lib_test_outcome"] in (
        "success",
        "failure",
        "cancelled",
        "skipped",
        "timed_out",
        "unknown",
    )
    assert "ci_duration_ms" in ci


def test_write_ops_dashboard_witness():
    body = write_ops_dashboard_witness(window_hours=24)
    assert body.get("written") == OPS_DASHBOARD_REL
    path = repo_root() / OPS_DASHBOARD_REL
    assert path.is_file()
    disk = json.loads(path.read_text(encoding="utf-8"))
    assert disk["schema"] == "ops_dashboard_v1"
    assert disk.get("_agent_meta", {}).get("source_system") == "ops_telemetry"


def test_summarize_github_ci_run_reads_lib_step():
    run = {
        "id": 36049220492,
        "name": "CI",
        "head_sha": "0faa8bb3b6e1badf493c38bbed29ade0b3956e9e",
        "conclusion": "cancelled",
        "created_at": "2026-09-24T19:35:48Z",
        "updated_at": "2026-09-25T01:36:43Z",
        "html_url": "https://github.com/zeplaz/Rust_engine_template_01/actions/runs/36049220492",
    }
    jobs = [
        {
            "name": "check",
            "conclusion": "cancelled",
            "started_at": "2026-09-24T19:35:51Z",
            "completed_at": "2026-09-25T01:36:42Z",
            "steps": [
                {
                    "name": "cargo test (lib)",
                    "conclusion": "cancelled",
                    "started_at": "2026-09-24T20:03:21Z",
                    "completed_at": "2026-09-25T01:36:40Z",
                }
            ],
        }
    ]
    body = summarize_github_ci_run(run, jobs)
    assert body["schema"] == "ops_ci_lib_test_v1"
    assert body["ok"] is True
    assert body["lib_test_outcome"] == "cancelled"
    assert body["ci_duration_ms"] == 21_651_000
    assert body["lib_test_duration_ms"] == 19_999_000
    assert body["run_id"] == 36049220492


def test_summarize_github_ci_run_missing_lib_step_is_unknown():
    body = summarize_github_ci_run(
        {"id": 1, "name": "CI", "created_at": "2026-09-24T00:00:00Z", "updated_at": "2026-09-24T00:01:00Z"},
        [{"name": "check", "started_at": "2026-09-24T00:00:01Z", "completed_at": "2026-09-24T00:00:02Z", "steps": []}],
    )
    assert body["lib_test_outcome"] == "unknown"
    assert body["ok"] is False
    assert body["ci_duration_ms"] == 1_000
    assert body["lib_test_duration_ms"] is None


def test_cli_ops_dashboard_refresh():
    proc = subprocess.run(
        [sys.executable, "-m", "rust_engine_mcp.cli", "ops-dashboard-refresh", "--window-hours", "24"],
        cwd=repo_root() / "tools/mcp/python",
        capture_output=True,
        text=True,
        check=False,
    )
    assert proc.returncode == 0, proc.stderr or proc.stdout
    body = json.loads(proc.stdout)
    assert body.get("ok") is True
