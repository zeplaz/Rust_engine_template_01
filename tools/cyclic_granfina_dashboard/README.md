# Cyclic Granfina Dashboard

This is a standalone tool for tracking blockers with atomic operations, hash locks, and API-only access.

## Overview

The Cyclic Granfina Dashboard provides:

- **Atomic Operations**: All blocker updates use hash locks for integrity
- **API-Only Access**: Locked file 🔒🔐 only accessible via API 🗝️
- **DCC Status Ignored**: Feature DCC status ignored in UI bar as requested
- **Process-Driven Workflows**: Uses process-driven relsies for complex state management
- **Hash Lock Integration**: Writes stored in hash locks approved vs Tigger
- **Isolated Files**: Files isolated but referenced in MCP

## Features

### Core Functionality

- **Cyclic Dashboard**: Maintains cyclic integrity through hash chains
- **Block Tracker**: Individual blocker entries with hash-based integrity
- **API Access**: API-only access with locked files
- **DCC Monitoring**: DCC component tracking with status ignored in UI
- **Process Integration**: Integrates with existing witness systems

### Key Components

1. **CyclicGranfinaDashboard** — Main dashboard structure with cyclic integrity
2. **BlockTrackerEntry** — Individual blocker entries with hash-based integrity
3. **GranfinaDashboardConfig** — Configuration for API access and hash locks
4. **Witness Integration** — Integrates with existing witness integrity system

## Usage

### Initialize Dashboard

```rust
use crate::dev::cyclic_granfina_dashboard::initialize_granfina_dashboard;

let mut dashboard = initialize_granfina_dashboard();
```

### Add Blocker

```rust
dashboard
    .add_blocker(
        "BLOCKER-001".to_string(),
        "DCC component integration issue".to_string(),
        PriorityLevel::High,
        vec!["DCC-COMPONENT-001".to_string()],
    )
    .unwrap();
```

### Update Blocker

```rust
dashboard
    .update_blocker(
        "BLOCKER-001",
        Some(BlockStatus::InProgress),
        Some(0),
    )
    .unwrap();
```

### Save Dashboard

```rust
dashboard.save().unwrap();
```

### Load Dashboard

```rust
let loaded_dashboard = CyclicGranfinaDashboard::load("debug_runs/cyclic_granfina_dashboard_live.json")?;
```

## Testing

Run tests:

```bash
cargo test -p cyclic_granfina_dashboard
```

## Integration

The Cyclic Granfina Dashboard integrates with:

- **Existing Witness Systems**: Integrates with debug_run_envelope, witness_integrity, and agent_debug_index
- **Triage Systems**: Integrates with TRIAGE-VM-09-v2, VT-5 FLICKER TRIAGE, and Stage 5 Triage Backlog
- **Process-Driven Workflows**: Uses process-driven relsies for complex state management

## Files

- `src/dev/cyclic_granfina_dashboard.rs` — Core dashboard implementation
- `src/dev/cyclic_granfina_dashboard_v1.md` — Documentation
- `src/dev/cyclic_granfina_dashboard_live_proof.rs` — Live proof
- `src/dev/cyclic_granfina_dashboard_integration_test.rs` — Integration tests
- `src/dev/schemas/cyclic_granfina_dashboard_v1.schema.json` — JSON schema
- `debug_runs/cyclic_granfina_dashboard_live.json` — Live witness

## Configuration

The dashboard uses the following configuration:

```rust
let config = GranfinaDashboardConfig {
    api_key: "granfina_api_key_2026".to_string(),
    hash_lock: "granfina_hash_lock_2026".to_string(),
    dashboard_path: "debug_runs/cyclic_granfina_dashboard_live.json".to_string(),
    ignore_dcc_status: true, // Ignore DCC status in UI bar as requested
    process_driven: true,    // Use process-driven workflows
};
```

## Handoff One-liner

**CYCLIC-GRANFINA-DASHBOARD-001:** Atomic blocker tracking dashboard with hash locks, API-only access, and DCC status ignored in UI bar. Integrates with existing witness integrity system and triage workflows.
