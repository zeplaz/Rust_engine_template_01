# Fire streaming debug overlay names `v1` (DESIGN-F7-B-DEBUG-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-F7-B-DEBUG-001** |
| **Alias** | **FIRE7-DESIGN-002** (Phase 7 plan) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Status** | **SIGNED** |
| **Coder lane** | **FIRE7-F7-B-001** — **CLOSED** (streaming impl) |
| **Prereq** | [`fire_lod_player_read_v1.md`](fire_lod_player_read_v1.md) · [`fire7_f7_b_streaming_impl_plan_v1.md`](fire7_f7_b_streaming_impl_plan_v1.md) |
| **Code** | [`fire_streaming.rs`](../render/fire_streaming.rs) · [`camera_focus_debug.rs`](../gui/camera_focus_debug.rs) · [`diagnostics_ui.rs`](../gui/diagnostics_ui.rs) |
| **Witness** | [`debug_runs/fire_streaming_live.json`](../debug_runs/fire_streaming_live.json) |

**No Rust in this doc.** Canonical **F3** section title, telemetry line keys, and map gizmo legend for F7-B chunk sleep/wake — aligned with **BQ-134** terse `key=value` style ([`stage6_consumer.rs`](../gui/hud/stage6_consumer.rs)).

---

## Executive summary

| Surface | Purpose |
|:---|:---|
| **F3 — collapsing header** | Engineer read of sleep/wake + active set (not player-facing) |
| **Map gizmo legend** | Focus grid + fire-active tiles when `CameraFocusDebug` enabled |
| **Log prefix** | `FOCUS:` trace line — keep; add optional `F7B:` rollup line |

**Designer rule:** Names use **product words** (sleep / wake / active chunks), not internal struct names (`visual_active` may appear once in a tooltip).

---

## F3 panel — section title (new)

| Element | Canonical string |
|:---|:---|
| **CollapsingHeader** | `Fire Phase 7 — chunk streaming (F7-B)` |
| **Placement** | After **Stage 5 readiness**, before **GPU weather / fire field (compute)** |
| **default_open** | `false` in Simulation; `true` in editor dev profile optional |

---

## F3 telemetry lines (wire order)

Each line is one `muted_label` row. Keys are **stable** for logs / mods / witness diff.

| # | Display line (template) | JSON / resource source |
|:---:|:---|:---|
| 1 | `F7B gate={gate} green={green}` | `fire_streaming_live.json` `/gate`, `/green` |
| 2 | `F7B focus_chunk=({x},{y}) sleep_r={sleep_r}` | `FireStreamingWitness.focus_chunk` · `FIRE_STREAMING_SLEEP_RADIUS` |
| 3 | `F7B sleep={sleep} wake={wake} active={active}` | `/sleep_transitions`, `/wake_transitions`, `/active_chunk_count` |
| 4 | `F7B runtime_writer={runtime_writer}` | `/runtime_writer` |
| 5 | `F7B visual_active={vis} sim_active={sim} total_chunks={tot}` | `FireChunkRuntime` counts (optional P2 coder) |

**Example (witness-backed 2026-05-26):**

```text
F7B gate=FIRE7-F7-B-001 green=true
F7B focus_chunk=(0,0) sleep_r=6
F7B sleep=1 wake=0 active=1
F7B runtime_writer=true
```

---

## F3 tooltip — field glossary

| Key | Player/engineer meaning |
|:---|:---|
| **sleep** | Chunks that lost `visual_active` this frame (beyond Chebyshev radius from camera focus) |
| **wake** | Sleeping chunks re-enabled because a **hot** neighbor is within 1 tile |
| **active** | Count in `ActiveFireChunkSet` after sync |
| **sleep_r** | Chebyshev distance threshold (`FIRE_STREAMING_SLEEP_RADIUS` = 6) |
| **visual_active** | Chunk may contribute to fire **draw** path |
| **sim_active** | Chunk has sim heat above `FIRE_SIM_CHUNK_ACTIVE_EPS` |

---

## Map gizmo legend (`CameraFocusDebug` / tile debug)

When focus overlay or instanced tile debug is on, use this legend in F3 or hover on **Debug → Focus grid**:

| Gizmo color | Label (UI) | Meaning |
|:---|:---|:---|
| Gold `#F2D926` | **Focus chunk** | Camera-derived focus tile |
| Red `#FF261E` | **Fire active** | Chunk in fire-active union (heat + instances) |
| Green `#33BF40` | **Terrain resident** | Chunk entity present |
| Dark gray `#1E1E24` | **Empty** | No terrain / no fire |

**Do not rename** log prefix `FOCUS: tile=… fire_active=…` — add parallel `F7B:` line only if coder wants single-grep filter.

---

## Relation to other F3 sections

| Existing section | Relationship |
|:---|:---|
| **GPU weather / fire field (compute)** | GPU ping-pong field — **not** F7-B chunk sleep |
| **Stage 6 residency** (side panel / BQ-134) | World **tile** streaming — orthogonal to **fire chunk** streaming |
| **Atmosphere diagnostics** | Smoke/toxic budgets — complementary |

---

## Coder wiring (optional P2)

```
Lane: F7-B-DEBUG-UI-001 (optional)
Read: fire_streaming_debug_overlay_names_v1.md
Wire: diagnostics_ui.rs — CollapsingHeader + 4–5 muted_label rows
Resources: Res<FireStreamingWitness> Res<ActiveFireChunkSet> optional Res<FireChunkRuntime>
Do NOT: second fire extract · duplicate Stage 6 residency panel
Verify: F3 open in sim — lines match fire_streaming_live.json after cargo test fire_streaming
```

---

## Sign-off

| Role | Date | Verdict |
|:---|:---|:---|
| Designer | 2026-05-26 | **SIGNED** — label contract on disk |
| Coder | — | Optional F3 wire; **FIRE7-F7-B-001** witness already green |

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | **DESIGN-F7-B-DEBUG-001** — F3 + gizmo names |
