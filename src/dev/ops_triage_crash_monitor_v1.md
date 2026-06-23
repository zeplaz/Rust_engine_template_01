# OPS Triage & Crash Monitor `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **OPS-TRIAGE-CRASH-001** |
| **Status** | **SHIPPED** |
| **Owner** | `@operations-intelligence` |
| **Daemon** | Python loop — **not cron** |

## Architecture

```text
ops_crash_daemon (loop)
  → process poll (tasklist/ps) + PID tracking
  → preview *.status.json failures
  → witness staleness / data drops
  → crash_events.jsonl (append, hash-locked)
  → triage_live.json (atomic hash chain)
  → prometheus/rust_engine_ops.prom
Grafana/Prometheus alerts → agent heartbeat / @operations-intelligence
```

## Glyph chain (SYMLANG tier hints)

| Glyph | Meaning |
|:---|:---|
| ⛔ | DCC/process crash |
| 🔴 | Preview job non-zero exit |
| 🧊 | Data drop — witness missing |
| ⚠ | Stale witness |
| ⚡ | Process surge (many Blender) |
| ✅ | Clean scan |

Glyphs appear in `glyph_chain` on witnesses for MCP/agent context — token-efficient vs long prose.

## Atomic writes

- `ops_atomic_witness.py` — hash lock file + temp + `os.replace`
- Approved actors only (`ops_crash_exporter`, `ops_crash_daemon`, MCP tools)
- `RUST_ENGINE_OPS_HASH_LOCK` env overrides default lock secret

## Commands

```powershell
# One-shot scan + Prometheus
python -m rust_engine_mcp.cli ops-crash-scan

# Full triage witness (blockers + ops + crash)
python -m rust_engine_mcp.cli ops-triage-refresh

# Background daemon (30s default)
tools/orchestrator/scripts/ops_crash_daemon.ps1

# Full ops spine includes triage refresh
tools/orchestrator/scripts/ops_intelligence_scan.ps1
```

## Grafana + Prometheus

1. **node_exporter textfile collector** → scrape `debug_runs/agent_ops/prometheus/rust_engine_ops.prom`
2. Import **`grafana_triage_overview.json`** — DCC process table, slip-ups, open gates
3. Load **`prometheus_alert_rules.yml`** — crash alerts, slip-up surge, many DCC processes

## Cyclic granfina bridge

Rust `cyclic_granfina_dashboard` crate holds hash-lock schema for blockers. Python triage witness feeds Grafana; crash events can be promoted to granfina blockers via `@operations-intelligence` ΔWF routing.

## Agent heartbeats

When Prometheus/Grafana fires:

1. `@operations-intelligence` runs `ops-triage-refresh` + reads `slip_ups[]`
2. Route ⛔/🔴 to `@coder-mcp` (DCC) or `@sim-steward` (ECS drift)
3. Append `agent_run_append` with `glyph_chain` in note field
