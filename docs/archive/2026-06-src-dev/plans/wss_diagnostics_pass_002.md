# DESIGN-WSS-DIAGNOSTICS-PASS-002 — Substrate diagnostics naming pass `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-WSS-DIAGNOSTICS-PASS-002** |
| **Scope** | Substrate diagnostics copy + overlay key naming consistency |
| **References** | [`wss_substrate_diagnostics_copy_v1.md`](wss_substrate_diagnostics_copy_v1.md), [`wss_substrate_debug_overlay_names_v1.md`](wss_substrate_debug_overlay_names_v1.md) |
| **Witness** | `debug_runs/wss_substrate_live.json` |
| **Version** | `1.0.0` |
| **Date** | 2026-05-27 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |
| **Unblocks** | WSS diagnostics polish on A-W1/A-W2 and operator handoff consistency |
| **No Rust** | Naming + copy validation only |

---

## Validation against current witness

Current substrate witness is operational (`green: true`) and exposes canonical fields used by diagnostics:

- `chunk_count`
- `resident_count`
- `dirty_count`
- `substrate_plugin_enabled`
- `hybrid_ecs_weather_authoritative`
- `hybrid_ecs_fire_authoritative`

Nested block present for atmosphere lane (`wss_atmos_clipmap_001`) with consistent snake_case keys.

---

## Canonical UI naming (PASS)

| UI key | Witness pointer |
|:---|:---|
| `substrate_chunk_count` | `/chunk_count` |
| `substrate_resident` | `/resident_count` |
| `substrate_dirty` | `/dirty_count` |
| `substrate_plugin` | `/substrate_plugin_enabled` |

Status lines in `wss_substrate_diagnostics_copy_v1.md` remain valid against the live witness shape.

---

## Do not break

- `wss_substrate_diagnostics_copy_v1.md` F3 line templates
- `wss_substrate_debug_overlay_names_v1.md` canonical key table
- `debug_runs/wss_substrate_live.json` snake_case contract

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-05-27 |
