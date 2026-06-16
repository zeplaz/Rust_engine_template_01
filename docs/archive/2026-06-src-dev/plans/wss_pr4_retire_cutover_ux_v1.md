# DESIGN-PR4-RETIRE-UX-001 — PR-4 / PR-5 hybrid retire cutover UX `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-PR4-RETIRE-UX-001** |
| **Parent** | [`plan_wss_hybrid_retire_pr4_001_v1.md`](plan_wss_hybrid_retire_pr4_001_v1.md) · [`wss_dual_write_transition_ux_001.md`](wss_dual_write_transition_ux_001.md) |
| **Diagnostics base** | [`wss_substrate_diagnostics_copy_v1.md`](wss_substrate_diagnostics_copy_v1.md) · [`wss_substrate_debug_overlay_names_v1.md`](wss_substrate_debug_overlay_names_v1.md) |
| **PR-5 smoke follow-on** | [`plan_wss_pr5_smoke_prod_001_v1.md`](plan_wss_pr5_smoke_prod_001_v1.md) |
| **Version** | `1.1.0` |
| **Date** | 2026-06-02 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |
| **Witness** | [`debug_runs/wss_substrate_live.json`](../debug_runs/wss_substrate_live.json) |
| **Unblocks** | `WSS-SLAB-PR-5` smoke prod · F3/diagnostics wire for retire banners |
| **No Rust** | Player + diagnostics copy contract only |

---

## Witness baseline (2026-05-27)

| Field | Value | Phase |
|:---|:---|:---|
| `substrate_persist_roundtrip_ok` | `true` | **PR-4** closed |
| `dynamic_overlay_migrated` | `true` | **PR-4** closed |
| `ecs_retire_fixture_green` | `true` | **PR-5** fixture closed |
| `hybrid_ecs_weather_authoritative` | `false` | **PR-5** weather/fire cutover |
| `hybrid_ecs_fire_authoritative` | `false` | **PR-5** weather/fire cutover |
| `ecs_retire_cutover_complete` | `true` | **PR-5** fixture |
| `hybrid_ecs_smoke_authoritative` | `false` | **PR-5 smoke prod CLOSED** |
| `ecs_retire_smoke_prod_green` | `true` | smoke prod rollup |
| `ecs_retire_smoke_cutover_complete` | `true` | cutover complete |
| `dual_write_shim_enabled` | `false` | compare-only / PR-2 tail per disk |
| `dual_write_drift_max` | `0.0` | stable |
| `green` | `true` | rollup |

**2026-06-02:** Smoke prod witness green — enable **PR5-SMOKE-CLOSED** player footnote (`Atmosphere simulation updated`) per state table. Pending dev-only smoke row **removed** from default F3.

---

## Purpose

Operators and players need **plain-language status** during persist/overlay migration (PR-4) and ECS weather/fire retirement (PR-5) without implying gameplay regression or exposing hybrid jargon in simulation chrome.

**Extends** PR-2 dual-write copy — does not replace drift/authority rows.

---

## Cutover state machine (designer)

| State ID | Witness condition | Dev / diagnostics banner | Player-facing |
|:---|:---|:---|:---|
| **PR4-PERSIST-OK** | `substrate_persist_roundtrip_ok` | `WSS persist: slab round-trip OK` | *(none)* |
| **PR4-OVERLAY-OK** | `dynamic_overlay_migrated` | `WSS overlay: dynamic terrain on slab` | *(none)* |
| **PR4-CLOSED** | both PR-4 flags true | `WSS PR-4: persist + overlay migrated` | Optional once: `Archive updated` |
| **PR5-WEATHER-FIRE-RETIRED** | `hybrid_ecs_weather_authoritative==false` AND `hybrid_ecs_fire_authoritative==false` AND `ecs_retire_fixture_green` | `WSS PR-5: weather + fire sim on slab (ECS retired)` | Optional once: `Environmental simulation updated` |
| **PR5-SMOKE-CLOSED** | `hybrid_ecs_smoke_authoritative==false` AND smoke prod rollup green | `WSS PR-5: smoke on slab extract` | Optional once: `Atmosphere simulation updated` |
| ~~PR5-SMOKE-PENDING~~ | *(retired 2026-06-02)* | — | — |
| **ROLLBACK-WARNING** | any authority flips true after cutover | `WSS retire rollback — ECS authority restored (investigate)` | *(none)* |

**Priority:** show highest phase reached; never stack more than **one** player footnote per session entry.

---

## F3 — section title + migration banner

| Element | Canonical string |
|:---|:---|
| **CollapsingHeader** | `World Substrate (hybrid)` |
| **Info banner (replace PR-1 static banner when PR-4 green)** | `Hybrid: weather and fire simulation read from world substrate. ECS legacy components retired for weather/fire.` |
| **Info banner (PR-4 only, before PR-5)** | `Hybrid: persist and terrain overlay stored on substrate. Weather/fire still on ECS until retire witness green.` |
| **Smoke pending sub-row** | `WSS smoke: ECS authority until prod cutover (see hybrid_ecs_smoke_authoritative)` |

**Do not** use: deprecated, broken, migration failed, dual-write (player), ECS retired (player).

---

## F3 — telemetry rows (wire after dual-write block)

| # | Display line (template) | When shown |
|:---:|:---|:---|
| R1 | `WSS PR-4 persist_ok={substrate_persist_roundtrip_ok} overlay_migrated={dynamic_overlay_migrated}` | always in WSS section |
| R2 | `WSS PR-5 retire_fixture={ecs_retire_fixture_green} stable_ticks={ecs_retire_stable_ticks}` | `ecs_retire_fixture_green` defined |
| R3 | `WSS PR-5 weather_auth={hybrid_ecs_weather_authoritative} fire_auth={hybrid_ecs_fire_authoritative} smoke_auth={hybrid_ecs_smoke_authoritative}` | always (hybrid flags) |
| R4 | `WSS PR-5 extract_slab weather={ecs_retire_weather_extract_slab} fire={ecs_retire_fire_extract_slab}` | dev profile or `green==false` |
| R5 | `WSS PR-5 cutover_complete={ecs_retire_cutover_complete}` | witness present |

**Key names:** snake_case per [`wss_substrate_debug_overlay_names_v1.md`](wss_substrate_debug_overlay_names_v1.md). Add glossary rows:

| JSON pointer | Operator string |
|:---|:---|
| `/substrate_persist_roundtrip_ok` | `substrate_persist_roundtrip_ok` |
| `/dynamic_overlay_migrated` | `dynamic_overlay_migrated` |
| `/ecs_retire_fixture_green` | `ecs_retire_fixture_green` |
| `/ecs_retire_cutover_complete` | `ecs_retire_cutover_complete` |
| `/hybrid_ecs_smoke_authoritative` | `hybrid_ecs_smoke_authoritative` |

---

## Player-facing guardrails

1. **Never** show drift values, shim flags, or “ECS” / “slab” in simulation HUD chrome.
2. **Never** claim “upgrade complete” while `hybrid_ecs_smoke_authoritative==true` if product copy references full environmental cutover.
3. Footnotes are **neutral and singular** — max one per `OnEnter(Simulation)` after a phase flip.
4. **No alarm styling** for expected PR-5 smoke pending — diagnostics only.
5. Gameplay feel unchanged — copy must not promise new visuals.

### Approved player strings

| Moment | String | Max duration |
|:---|:---|:---|
| PR-4 closed | `Archive updated` | Toast ≤ 4s or status footnote once |
| PR-5 weather/fire | `Environmental simulation updated` | Toast ≤ 4s or footnote once |
| PR-5 smoke closed | `Atmosphere simulation updated` | Toast ≤ 4s or footnote once |

### Forbidden player strings

| String | Why |
|:---|:---|
| `ECS retired` | Internal architecture |
| `Substrate authoritative` | Jargon |
| `Dual-write active` | Dev-only |
| `Smoke migration failed` | No failure witness for pending state |
| `Weather system rebuilt` | Implies visual rewrite |

---

## Operator / dev diagnostics guardrails

1. PR-4 rows may show before PR-5 — banner must match lowest incomplete phase.
2. When `dual_write_shim_enabled==true`, keep PR-2 drift rows from dual-write doc **above** PR-4 block.
3. `ROLLBACK-WARNING` uses **warn** color in F3 only — never player toast.
4. Persist pending: `persist_pending_slots > 0` → `WSS persist: {n} slots pending flush` (dev only).

---

## Relation to PR-2 dual-write copy

| PR-2 state | PR-4/5 interaction |
|:---|:---|
| Dual-write active + drift 0 | PR-4 banner additive; do not remove drift row |
| Cutover-ready (PR-2) | PR-4 may proceed; player “Archive updated” optional |
| PR-5 fixture green | Replace PR-1 “ECS authoritative” static lines in diagnostics copy with PR-5 banner |

---

## Coder wiring sketch

```
Lane: DESIGN-PR4-RETIRE-UX-001
Read: wss_pr4_retire_cutover_ux_v1.md
Wire: diagnostics_ui.rs — World Substrate (hybrid) header + R1–R5 rows + phase banner
Resource: wss_substrate_live.json fields (no hand-edited JSON)
Player: optional toast hook on authority false transition (sim session only)
Do NOT: player copy for smoke pending
Verify: banner matches witness phase table after proof refresh
```

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-05-27 |
| `@designer` | **PASS** (smoke prod closed) | 2026-06-02 |

**Absorption:** optional tail closed — `hybrid_ecs_smoke_authoritative: false` on [`wss_substrate_live.json`](../debug_runs/wss_substrate_live.json).
