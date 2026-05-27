# IND-E03 ops strip polish `v1` (DESIGN-IND-E03-OPS-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-IND-E03-OPS-001** |
| **Coder lane** | **IND-E03-SIM-UX-001** (Coder B **#11**) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Status** | **SIGNED** |
| **Prereq** | [`s7p_grid_overload_ux_note_v1.md`](s7p_grid_overload_ux_note_v1.md) · [`s7p_grid_overload_ux_pass_record_v1.md`](s7p_grid_overload_ux_pass_record_v1.md) |
| **Code** | [`grid_overload_ux.rs`](../economy/activation/grid_overload_ux.rs) · [`simulation_shell_phase2.rs`](../gui/hud/simulation_shell_phase2.rs) |
| **Witness** | [`debug_runs/industrial_activation_live.json`](../debug_runs/industrial_activation_live.json) · [`debug_runs/ui_shell_migration_live.json`](../debug_runs/ui_shell_migration_live.json) |

**No Rust.** Polish contract for **Phase 2 ops strip** beyond witness green — idle vs overload PWR, zone prefixes, tray deferral.

---

## Executive summary

| Channel | v1 status | Polish target |
|:---|:---|:---|
| **PWR ops strip (primary)** | **Wired** — toast overrides idle % | See § PWR states |
| **ALERTS tray row** | **DEFERRED** | § Secondary — not blocking IND-E03 |
| **F3 diagnostics** | Tertiary per S7P note | One-line overload summary optional |

**Designer verdict:** **PASS (qualified)** for coder closure — witness + S7P PASS record satisfy product; § Optional polish is P2 enhancement only.

---

## Ops strip zone vocabulary (locked)

Align with [`OpsStripZone`](../../src/gui/hud/simulation_shell_phase2.rs) — **prefix + two spaces + body**:

| Zone | Idle template | Overload / event template |
|:---|:---|:---|
| **TIME** | `TIME  tick={n}  {paused\|run}  ×{speed}` | unchanged |
| **ALERTS** | `ALERTS  {n_missions}` | + tray row when shipped |
| **INTEL** | `INTEL  routes on\|off  c {proxy}` | unchanged |
| **WX** | `WX  r {r}  s {s}  f {f}` | unchanged |
| **PWR** | `PWR  {pct}%` | `PWR  ⚠ {GRID_OVERLOAD_TOAST_MESSAGE}` |
| **TRAY** | `▼ TRAY` / `◧ TRAY` / `▲ TRAY` | unchanged |

**Canonical overload body** (must match witness):

```text
Grid overload — reduce smelter load or add transformer capacity
```

**Prefix rule:** Overload uses `PWR  ⚠` + space + body (already in `apply_grid_overload_ops_strip_toast_system`).

---

## PWR state machine (designer)

```text
                    ┌─────────────────┐
     idle scarcity  │  PWR  {nn}%    │
        ───────────►│  (world_fields) │
                    └────────┬────────┘
                             │ GridOverloadEvent
                             ▼
                    ┌─────────────────┐
     ~240 ticks     │ PWR ⚠ + body  │◄── refresh on repeat event
                    └────────┬────────┘
                             │ tick > active_until_tick
                             ▼
                    ┌─────────────────┐
                    │  PWR  {nn}%    │  (restore idle line)
                    └─────────────────┘
```

| Rule | Spec |
|:---|:---|
| Toast window | `GRID_OVERLOAD_TOAST_TICKS` (240 @ 30 Hz ≈ 8s) |
| Repeat events | Refresh window; **no** stacked duplicate lines in PWR zone |
| Recovery | Revert to `PWR  {pct}%` when toast inactive |
| Editor / WorldGen | No overload copy |

---

## Secondary — alerts tray (deferred v1)

Per **S7P-DESIGN-002** — implement when **IND-E03-SIM-UX-001** extends beyond witness:

| Field | Value |
|:---|:---|
| **Title** | Grid overload |
| **Body** | Smelter demand exceeded bus capacity. Reduce load or place a distribution transformer. |
| **Severity** | Warning (amber) |
| **Throttle** | Max 1 row per bus / 30s |

**Not required** for qualified PASS on **IND-E03-SIM-UX-001** if `s7p_grid_ux_001.green` and PWR line canonical.

---

## Witness rollup (2026-05-26)

| JSON path | Expected | Observed |
|:---|:---|:---|
| `/s7p_grid_ux_001/green` | `true` | ☑ |
| `/s7p_grid_ux_001/toast_ui_wired` | `true` | ☑ |
| `/s7p_grid_ux_001/toast_message` | canonical body | ☑ |
| `/witness/ops_zones_wired` (ui shell) | `true` | ☑ |
| `/ind_e03/overload_events_total` | `> 0` | `30` |

---

## Optional polish (coder P2)

| ID | Change | Owner |
|:---|:---|:---|
| OPS-P2-1 | Brief PWR tint shift (amber) while toast active — text color only | @coder B |
| OPS-P2-2 | ALERTS badge increment on overload (count only, row later) | @coder B |
| OPS-P2-3 | Narrow layout: use short copy key `grid.overload.toast.short` | @designer + coder |

---

## Coder exit — IND-E03-SIM-UX-001

```
Lane: IND-E03-SIM-UX-001
Prereq: DESIGN-IND-E03-OPS-001 SIGNED · S7P-GRID-UX-UI-001 green
Read: ind_e03_ops_strip_polish_v1.md · grid_overload_ux.rs
Exit: s7p_grid_ux_001 + ind_e03_witness_001_green · PWR canonical in sim
Verify: cargo test -p proc_A_dine01 --lib industrial_activation
```

---

## Sign-off

| Role | Date | Verdict |
|:---|:---|:---|
| Designer | 2026-05-26 | **PASS (qualified)** |
| Coder B | — | Close on witness; tray row = follow-up |

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | **DESIGN-IND-E03-OPS-001** |
