# Power grid HUD copy registry `v1`

| Field | Value |
|:---|:---|
| **Program** | PLAN-POWER-GRID-CONSTRUCTION-UX-001 |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Specs** | tool sheet · routing · voltage · overlay · targeting · repair |
| **Verdict** | **PASS** |

Locked strings for power line UX — coder: `power_grid_copy.rs` or extend `sim_hud_copy_registry_v1.md` §power.

---

## Tool sheet & strip

| Key | String |
|:---|:---|
| `power.strip.drawing` | `POWER · {voltage} · {mode} · LMB add · RMB undo · Shift commit` |
| `power.strip.blocked` | `POWER · blocked: {reason}` |
| `power.strip.queued` | `POWER · line queued · {n} segments` |
| `power.strip.island` | `POWER · island — {n} offline` |
| `power.sheet.build` | `Build line` |
| `power.sheet.cancel` | `Cancel` |
| `power.mode.curved` | `Curved` |
| `power.mode.orthogonal` | `90°` |

## Voltage (see voltage picker spec)

## Targeting (see targeting spec)

## Repair (see repair panel spec)

## Island

| Key | String |
|:---|:---|
| `power.island.toast` | `Power island — {n} buildings offline` |
| `power.island.ops` | `⚠ Island · {n} offline` |

## Node hover (DES-POWER-NODE-HOVER-001)

| Key | String |
|:---|:---|
| `power.hover.transformer.title` | `Distribution transformer` |
| `power.hover.substation.title` | `Grid substation` |
| `power.hover.status.online` | `● Online` |
| `power.hover.status.offline` | `○ Offline` |
| `power.hover.status.damaged` | `◆ Damaged` |
| `power.hover.status.destroyed` | `× Destroyed` |
| `power.hover.status.overload` | `⟳ Overload` |
| `power.hover.load` | `Load` |
| `power.hover.capacity` | `Capacity` |
| `power.hover.feeds` | `Feeds` |
| `power.hover.links` | `Links` |
| `power.hover.links.fmt` | `{lines} lines · {upstream} upstream` |
| `power.hover.feeds.fmt` | `{n} consumers` |
| `power.hover.capacity.fmt` | `{used} / {max} MVA` |
| `power.hover.voltage.mixed` | `Mixed voltage` |
