# WSS substrate diagnostics copy `v1` (WSS-DESIGN-GATE-001 · G1)

| Field | Value |
|:---|:---|
| **Gate** | **WSS-DESIGN-GATE-001** — slab preflight **G1** |
| **Parent** | [`wssr_design_signoff_v1.md`](wssr_design_signoff_v1.md) · [`wss_design_gate_001_v1.md`](wss_design_gate_001_v1.md) |
| **Exec plan** | [`plan_wss_chunk_slab_exec_001_v1.md`](plan_wss_chunk_slab_exec_001_v1.md) (**WSS-CHUNK-SLAB-001**) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Status** | **SIGNED** |
| **Coder wire** | `src/gui/diagnostics_ui.rs` (collapsed section) — optional until substrate plugin ships |
| **No Rust** | Copy + placement only |

---

## Purpose

Operators and agents must see that **PR-1 slab is a mirror**, not sim authority. Gameplay weather and fire still read **ECS chunk components** until PR-2 dual-write / cutover.

---

## F3 panel — section title

| Element | Canonical string |
|:---|:---|
| **CollapsingHeader** | `World Substrate (PR-1)` |
| **Placement** | After **Fire Phase 7 — chunk streaming (F7-B)**, before **GPU weather / fire field (compute)** |
| **default_open** | `false` in Simulation; `true` in editor dev profile optional |

---

## F3 body copy (wire order)

Use **muted_label** rows — one line each. Template variables match witness JSON ([`plan_wss_chunk_slab_exec_001_v1.md`](plan_wss_chunk_slab_exec_001_v1.md) § Witness).

| # | Display line (template) | Source |
|:---:|:---|:---|
| 1 | `WSS gate={gate} green={green}` | `wss_substrate_live.json` `/gate`, `/green` |
| 2 | `WSS slab_chunks={chunk_count} resident={resident_count} dirty={dirty_count}` | `/chunk_count`, `/resident_count`, `/dirty_count` |
| 3 | `WSS plugin={substrate_plugin_enabled}` | `/substrate_plugin_enabled` |
| 4 | `WSS hydrate={hydrate_wired} paging={paging_wired}` | `/hydrate_wired`, `/paging_wired` |
| 5 | `WSS sim authority: ChunkWeather + ChunkSurfaceFire (ECS)` | static (PR-1) |
| 6 | `WSS slab mirror only until PR-2 cutover` | static (PR-1) |

**Example (skeleton witness):**

```text
WSS gate=WSS-CHUNK-SLAB-001 green=false
WSS slab_chunks=0 resident=0 dirty=0
WSS plugin=true
WSS hydrate=false paging=false
WSS sim authority: ChunkWeather + ChunkSurfaceFire (ECS)
WSS slab mirror only until PR-2 cutover
```

---

## Hybrid migration banner (required)

**PR-1 default** (when PR-4/PR-5 retire witnesses not green):

```text
Hybrid (PR-1): ECS components are authoritative for weather and fire. WorldSubstrateRegistry is hydrate + witness only.
```

**PR-4+ retire banners** — use [`wss_pr4_retire_cutover_ux_v1.md`](wss_pr4_retire_cutover_ux_v1.md) (**DESIGN-PR4-RETIRE-UX-001**) when `substrate_persist_roundtrip_ok` / `ecs_retire_fixture_green` drive phase.

**Do not** use the word “deprecated” in player-facing copy — reserve for dev logs.

---

## Tooltip — hybrid authority (hover on line 5–6)

| Term | Meaning |
|:---|:---|
| **ChunkWeather** | ECS component — sim weather truth until slab cutover |
| **ChunkSurfaceFire** | ECS component — fire ecology / surface state on chunk entities |
| **Slab mirror** | `ChunkSlab` cells copied from terrain + markers; not written by render or egui |
| **PR-2** | Dual-write shim + drift witness (`dual_write_drift_max`) — future milestone |

---

## Relation to other diagnostics

| Section | Relationship |
|:---|:---|
| **Fire Phase 7 (F7-B)** | Fire **chunk** sleep/wake — orthogonal to substrate slab |
| **Stage 6 residency** | World tile streaming — orthogonal to `ChunkSlab` registry |
| **GPU weather / fire field** | GPU ping-pong — not substrate L1 |
| **Construction parametric** | Build ghosts — no substrate writer |

---

## Acceptance (G1)

1. Copy states **ECS authoritative until PR-2** in plain language.
2. Slab described as **mirror / hydrate** only — not “new sim truth”.
3. Telemetry keys align with `wss_substrate_live.json` field names.
4. Section title exact: `World Substrate (PR-1)`.

---

## Coder wiring sketch

```
Lane: WSS-CHUNK-SLAB-001 (optional P2 UI)
Read: wss_substrate_diagnostics_copy_v1.md
Wire: diagnostics_ui.rs — CollapsingHeader + 6–7 muted_label rows
Resource: Res<WssSubstrateWitness> or read proof JSON in dev profile
Do NOT: claim slab drives weather sim in PR-1
Verify: lines match wss_substrate_live.json after substrate proof writer runs
```
