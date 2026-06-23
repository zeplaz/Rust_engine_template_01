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


def test_write_ops_dashboard_witness():
    body = write_ops_dashboard_witness(window_hours=24)
    assert body.get("written") == OPS_DASHBOARD_REL
    path = repo_root() / OPS_DASHBOARD_REL
    assert path.is_file()
    disk = json.loads(path.read_text(encoding="utf-8"))
    assert disk["schema"] == "ops_dashboard_v1"
    assert disk.get("_agent_meta", {}).get("source_system") == "ops_telemetry"


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
