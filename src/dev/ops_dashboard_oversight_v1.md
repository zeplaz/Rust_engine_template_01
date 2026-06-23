# OPS Dashboard Oversight `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **OPS-DASHBOARD-001** |
| **Status** | **SHIPPED** — MCP + CLI + witness + Grafana/HTML viewers |
| **Owner** | `@operations-intelligence` |

## Problem

Agents lacked continuous oversight for:

- Running processes (Blender bakes, pytest, cargo, bevy)
- Drift / dishonest witness instances
- Run-event telemetry (`agent_run_append`) and repeat-slice slip-ups
- A single dashboard surface to catch regressions before they spread

## Solution

`tools/mcp/python/rust_engine_mcp/ops_telemetry.py` rolls up:

| Scan | Output |
|:---|:---|
| `scan_run_events` | FTR/RTR proxies, repeat-slice slip-ups from `run_events.jsonl` |
| `scan_processes` | OS tasklist/ps for engine-related PIDs |
| `scan_drift_instances` | viewport drift, dishonest gates, queue contradictions, ΔWF |
| `build_ops_dashboard` | Unified bundle → `debug_runs/agent_ops/ops_dashboard_live.json` |

## How to refresh

```powershell
tools/orchestrator/scripts/ops_intelligence_scan.ps1   # full ops scan + dashboard
tools/orchestrator/scripts/ops_dashboard_refresh.ps1   # dashboard only
python -m rust_engine_mcp.cli ops-dashboard-refresh
```

## MCP tools

| Tool | Role |
|:---|:---|
| `ops_dashboard_snapshot` | Live bundle; `write_witness=true` persists JSON |
| `ops_process_scan_tool` | Process scan only |
| `ops_drift_scan_tool` | Drift instances only |
| `ops_run_events_rollup_tool` | Run events rollup only |

## Viewers

- **Local HTML:** `tools/orchestrator/dashboard/ops_dashboard.html` (fetch or file picker for `ops_dashboard_live.json`)
- **Grafana:** import `tools/orchestrator/dashboard/grafana_ops_overview.json` with Infinity datasource pointed at `ops_dashboard_live.json`

## Agent ritual

1. End every slice: `agent_run_append` via HANDOFF `-OpsEvent` or MCP `BLANG:RUN`
2. Lane close: `ops_intelligence_scan.ps1`
3. Review `slip_ups[]` in dashboard — route alerts to `@sim-steward` / `@orchestrator`

## Tier-1 KPIs

`metrics_tier1` in `ops_project_brief_v1` now merges run_events rollup when the ledger is populated. Sparse until HANDOFF telemetry is used consistently.
