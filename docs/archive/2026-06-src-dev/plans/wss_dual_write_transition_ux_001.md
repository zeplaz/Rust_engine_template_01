# DESIGN-DUAL-WRITE-UX-001 / DESIGN-DUAL-WRITE-FULL-PASS-001 — PR-2 dual-write transition UX `v1.1`

| Field | Value |
|:---|:---|
| **Queue IDs** | **DESIGN-DUAL-WRITE-UX-001** · **DESIGN-DUAL-WRITE-FULL-PASS-001** |
| **Target lane** | WSS slab PR-2 dual-write transition |
| **Parent** | [`wssr_design_signoff_v1.md`](wssr_design_signoff_v1.md), [`plan_wss_chunk_slab_exec_001_v1.md`](plan_wss_chunk_slab_exec_001_v1.md) |
| **Version** | `1.1.0` |
| **Date** | 2026-05-27 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |
| **Witness JSON** | [`debug_runs/wss_substrate_live.json`](../debug_runs/wss_substrate_live.json) |
| **Unblocks** | [`wss_pr4_retire_cutover_ux_v1.md`](wss_pr4_retire_cutover_ux_v1.md) (**DESIGN-PR4-RETIRE-UX-001**) · `PLAN-WSS-SLAB-PR-2` · `WSS-SLAB-PR-2` |
| **No Rust** | UX/copy contract only |

---

## Witness (2026-05-27)

| Field | Value |
|:---|:---|
| `dual_write_shim_enabled` | `true` |
| `dual_write_drift_max` | `0.0` |
| `hybrid_ecs_weather_authoritative` | `true` |
| `hybrid_ecs_fire_authoritative` | `true` |
| `green` | `true` |

**Policy:** Full **PASS** — PR-2 keys live; copy contract validated against witness.

---

## Purpose

UX language for ECS + slab coexistence during PR-2. Prevent false “slab authoritative” claims while drift proof is green.

---

## Transition states

| State | Condition | Dev/operator banner | Player-facing (optional footnote) |
|:---|:---|:---|:---|
| **PR-1 mirror** | `dual_write_shim_enabled=false` | `Hybrid: ECS authoritative, slab mirror only` | *(none — default play)* |
| **PR-2 dual-write active** | `dual_write_shim_enabled=true` | `Hybrid: ECS + slab dual-write active (monitor drift)` | `Environmental data syncing` |
| **Cutover-ready** | `dual_write_drift_max < ε` (witness: `0.0`) | `Dual-write stable; cutover candidate` | `Environmental data synced` |

**ε:** `1e-6` for proof JSON; operator may tighten in live sim.

---

## F3 / diagnostics label contract

| Row | Template | When shown |
|:---|:---|:---|
| **Drift** | `WSS dual_write_shim={dual_write_shim_enabled} drift_max={dual_write_drift_max}` | Always in WSS substrate section |
| **Authority** | `WSS hybrid_weather_auth={hybrid_ecs_weather_authoritative} hybrid_fire_auth={hybrid_ecs_fire_authoritative}` | Always |
| **Stable** | `WSS dual-write stable` | `dual_write_shim_enabled=true` AND `dual_write_drift_max==0` |
| **Warning** | `Drift elevated — keep ECS authority` | shim true AND drift above ε |

**Key names:** match [`wss_substrate_debug_overlay_names_v1.md`](wss_substrate_debug_overlay_names_v1.md) snake_case.

---

## Player-facing guardrails

1. Never show “slab authoritative” or “ECS retired” while any `hybrid_ecs_*_authoritative` flag is `true`.
2. Player copy is **neutral status only** — no drift numbers, no “dual-write” jargon in sim chrome.
3. Use **Environmental data synced** only when drift is at zero; otherwise omit player footnote (dev banner carries detail).
4. Gameplay claims unchanged — weather/fire sim authority remains ECS per witness.

---

## UX guardrails (dev/operator)

1. Never show "slab authoritative" while any `hybrid_ecs_*_authoritative` flag is true.
2. Dual-write warnings are developer/operator only unless cutover-ready footnote applies.
3. Use same snake_case key names as overlay names doc.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** — DESIGN-DUAL-WRITE-FULL-PASS-001 | 2026-05-27 |
| Witness | `wss_substrate_live.json` dual_write fields green | 2026-05-27 |
