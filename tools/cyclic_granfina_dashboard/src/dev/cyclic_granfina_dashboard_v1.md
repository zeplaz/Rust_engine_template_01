# CYCLIC-GRANFINA-DASHBOARD — atomic blocker tracking dashboard `v1`

```text
⟦SYMLANG⟧⟐v1  ◈PLAN
<ID> CYCLIC-GRANFINA-DASHBOARD-001
Date: 2026-06-19
Status: **ACTIVE** (@coder)
Parent: STAGE5-CONVERGENCE · TRIAGE-WORKFLOW
Schema: $ref:tools/mcp/schemas/cyclic_granfina_dashboard_v1.schema.json
```

## Executive Summary

This system provides a **cyclic granfina dashboard** for tracking blockers with **atomic operations**, **hash-based locking**, and **API-only access**. It integrates with the existing witness integrity system and **ignores DCC status in the UI bar** as requested.

## Key Features

| Feature | Description |
|:-------|:---|
| **Atomic Operations** | All blocker updates use hash locks for integrity |
| **API-Only Access** | Locked file 🔒🔐 only accessible via API 🗝️ |
| **DCC Status Ignored** | Feature DCC status ignored in UI bar as requested |
| **Process-Driven Workflows** | Uses process-driven relsies for complex state management |
| **Hash Lock Integration** | Writes stored in hash locks approved vs Tigger |
| **Isolated Files** | Files isolated but referenced in MCP |
| **Witness Integration** | Integrates with existing witness integrity system |

## Technical Architecture

### Core Components

1. **CyclicGranfinaDashboard** — Main dashboard structure with cyclic integrity
2. **BlockTrackerEntry** — Individual blocker entries with hash-based integrity
3. **GranfinaDashboardConfig** — Configuration for API access and hash locks
4. **Witness Integration** — Integrates with existing witness integrity system

### Data Flow

```text
Dashboard → Blockers → Hash Locks → Witness Integrity → API Access
```

### Hash Lock Mechanism

- Each blocker entry has a `previous_hash` and `current_hash`
- Updates require valid hash lock verification
- Dashboard maintains cyclic integrity through hash chaining
- All writes are stored in hash locks approved vs Tigger

## API Access

The dashboard uses **API-only access** with a locked file 🔒🔐:

```rust
let config = GranfinaDashboardConfig {
    api_key: "granfina_api_key_2026".to_string(),
    hash_lock: "granfina_hash_lock_2026".to_string(),
    dashboard_path: "debug_runs/cyclic_granfina_dashboard_live.json".to_string(),
    ignore_dcc_status: true, // Ignore DCC status in UI bar as requested
    process_driven: true,    // Use process-driven workflows
};
```

## DCC Monitoring

The system **looks into DCC monitor on grpha dashboard already for course process**:

- DCC components are tracked in each blocker entry
- DCC status is ignored in the UI bar as requested
- Process-driven workflows handle DCC-related blockers
- Integration with existing DCC monitoring systems

## Atomic Operations

All blocker operations are **atomic**:

1. **Add Blocker** — Requires API access and valid hash lock
2. **Update Blocker** — Requires API access and valid hash lock
3. **Remove Blocker** — Requires API access and valid hash lock
4. **Validate Integrity** — Verifies hash chains and API access

## Witness Integration

The dashboard integrates with the existing witness integrity system:

```rust
let body = serde_json::to_value(dashboard).unwrap();
let wrapped = wrap_debug_run(
    "CYCLIC-GRANFINA-DASHBOARD-001",
    "create_witness",
    &dashboard.config.dashboard_path,
    body,
);

write_debug_run_json(&dashboard.config.dashboard_path, wrapped);
```

## Process-Driven Workflows

The system uses **process-driven relsies** for complex state management:

- Cyclic hash chains ensure process integrity
- Atomic operations prevent race conditions
- Hash locks ensure only approved actions are recorded
- Integration with existing MCP systems

## Testing

```powershell
cargo test -p proc_A_dine01 --lib cyclic_granfina_dashboard
```

## Files Created

- `src/dev/cyclic_granfina_dashboard.rs` — Core dashboard implementation
- `src/dev/cyclic_granfina_dashboard_v1.md` — This documentation
- `debug_runs/cyclic_granfina_dashboard_live.json` — Live witness (created on save)

## Integration with Existing Systems

### Triage Systems

The dashboard integrates with existing triage systems:

- **TRIAGE-VM-09-v2** — Used for infrastructure validation
- **VT-5 FLICKER TRIAGE** — Used for visual validation
- **Stage 5 Triage Backlog** — Used for blocker categorization

### Witness Systems

The dashboard integrates with existing witness systems:

- **debug_run_envelope** — For wrapping witness data
- **witness_integrity** — For hash-based validation
- **agent_debug_index** — For agent navigation hints

## Future Enhancements

1. **Real-time Updates** — WebSocket integration for real-time updates
2. **Dashboard UI** — Web-based dashboard for visualizing blockers
3. **Advanced Filtering** — Advanced filtering and search capabilities
4. **Export Functionality** — Export blocker data in various formats
5. **Integration with External Systems** — Integration with external monitoring systems

## Handoff One-liner

**CYCLIC-GRANFINA-DASHBOARD-001:** Atomic blocker tracking dashboard with hash locks, API-only access, and DCC status ignored in UI bar. Integrates with existing witness integrity system and triage workflows.

---

## Changelog

| Version | Date | Notes |
|:-------|:---|:---|
| v1.0.0 | 2026-06-19 | Initial implementation with atomic operations and hash locks |
