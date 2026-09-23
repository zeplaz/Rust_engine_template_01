# PLAN-VFX-SPECTATOR-SCALE-001 — Real game test · scale contract · artist VFX · reflection fleet `v1`

```text
⟦SYMLANG⟧⟐v1  ◈PLAN  ◈PROGRAM
⟨ID⟩ PLAN-VFX-SPECTATOR-SCALE-001  ·  VSS-001
Q↑↑  Ct▮▮▮  Cx▮▮▮▮▮▮▮▮  EV/Cx=HIGH  Au:🏛🟨🟨🟨🟨
Parent: $ref:src/dev/development_plan_index.md
Effects: $ref:src/dev/plan_effects_system_architecture_v1.md
Sim spine: $ref:src/dev/guide_sim_effect_spine_v1.md
Trip registry: $ref:tools/orchestrator/queues/vss_trip_artifact_registry.json
Tribunal: $ref:tools/orchestrator/queues/vss_tribunal_touch_registry.json
Build encoding: $ref:prompts/guides/symbolic_build_output_encoding_v1.md
Status: **ACTIVE** — 2026-08-28
Owner: @orchestrator → tiered subagents
```

**Rule:** Witness JSON wins over queue rows. **Trip artifacts** in `vss_trip_artifact_registry.json` MUST be read before editing listed paths. **Dissent is allowed** — tribunal packets keep minority rows.

**Kickoff packet (ops):** [`debug_runs/agent_ops/vss_001_kickoff_packet.yaml`](../../debug_runs/agent_ops/vss_001_kickoff_packet.yaml) — Q/C/E APPROVE · **T2-002 done** · pick **T3-002** / **T1-003** / **T4-004**

**Child plans:** [`plan_world_scale_contract_v1.md`](plan_world_scale_contract_v1.md) (T1) · [`plan_artist_vfx_toolchain_v1.md`](plan_artist_vfx_toolchain_v1.md) (T4)

---

## Executive verdict

Operators report **no visible fire/sparks/smoke** at any zoom. Telemetry can show green while the product path is wrong because:

1. **Scale contract missing** — 32×32 chunk slabs read as city-sized territories; buildings and fire do not align.
2. **Test harness ≠ game** — `--test vfx` seeds overlay heat; campaign/save/scenario paths are not the proof spine.
3. **No artist-owned effects lane** — sparks are Rust/WGSL-only; no Blender → consumable → preview pipeline.
4. **Build output not symbolic** — agents re-read raw cargo walls; token waste and repeated mistakes.

**Program goal:** Five parallel tracks with **~1/7 fleet** on reflection/tooling; every track emits **trip artifacts** and **witnesses**; build/compile output flows through **symbolic encoding** before agents reason.

---

## Fleet allocation (7-agent session model)

| Slot | Track | Agent | Never owns |
|:---:|:---|:---|:---|
| 1 | **T5 Reflection / encoding** | `@operations-intelligence` + `@coder-mcp` | Production ECS |
| 2 | T1 World scale | `@planner` → `@coder` | VFX shaders |
| 2 | T1 World scale (cont.) | `@designer` (readability audit) | Sim tick |
| 3 | T2 Spectator / test parity | `@orchestrator` → `@coder` | MCP art bake |
| 4 | T3 SimEffect + fire consumer | `@coder` + `@sim-steward` | Harness-only seeds as sole proof |
| 5 | T4 Artist VFX toolchain | `@orchestrator-mcp` → `@designer-mcp` → `@coder-mcp` | Inline procedural art |
| 6 | T4 Bevy registry hook | `@coder` | bpy execution |
| 7 | Cross-lane continuity | `@main-thread-orchestrator` | Parallel authority edits |

**Token policy:** L0 cheap digests → L2 exec only with filled packet (`agent_flow_route`). Never paste raw `cargo check` stderr — use `symbolic_build_digest_tool` / `validate_cargo_report`.

---

## Track 1 — World scale & grid contract (VSS-T1)

**Problem:** Tactical map cells feel like huge territories; multi-building blocks should fit in one viewport frame.

| Slice | ID | Owner | Deliverable |
|:---|:---|:---|:---|
| S1 | VSS-T1-001 | @planner | `plan_world_scale_contract_v1.md` — tile unit, chunk slab, building footprint |
| S2 | VSS-T1-002 | @coder | **SHIPPED** — `world_scale_contract.rs`; operational zoom on sim enter |
| S3 | VSS-T1-003 | @designer | **SHIPPED** — bpf 10.15 PASS w/ dissent; primary 78.8 px |
| S4 | VSS-T1-004 | @coder | Split strategic sim chunk vs tactical display tile if required |

**Witness:** `debug_runs/world_scale_contract_live.json`

**Trip:** Editing `WorldGenParams`, chunk slab size, `map_camera` limits → read tribunal registry + file `src/dev/TRIP_VSS_SCALE_CONTRACT.md`

---

## Track 2 — Real-game test & spectator pipeline (VSS-T2)

**Problem:** `--test vfx` proves harness green, not product fire from save/campaign/script.

```text
Entry: save │ campaign │ scenario script │ --test (parity only)
  → SimEffectQueue / scenario hooks
  → ChunkEnvironment fire tick (same as interactive)
  → extract → GPU (same spine)
  → witness (same schema all entry paths)
```

| Slice | ID | Owner | Deliverable |
|:---|:---|:---|:---|
| P0 | VSS-T2-001 | @coder | **SHIPPED** — harness fire `debug_fallback_only`; witness stub |
| P1 | VSS-T2-002 | @coder | **SHIPPED** — scenario `IgniteAt` / `TriggerEffect` |
| P1b | VSS-T2-003 | @coder | **SHIPPED** — all product entry paths green |
| P2 | VSS-T2-003 | @coder + @designer | Spectator: pause, step, inject event, RTT capture |
| P3 | VSS-T2-004 | @sim-steward | Parity gate: harness vs scenario vs interactive witness |

**Witness:** `debug_runs/spectator_parity_live.json`

**Trip:** `src/engine/test_harness.rs`, `play_scenario.rs`, `ActiveTestScene` — see `TRIP_VSS_TEST_HARNESS.md`

---

## Track 3 — SimEffect spine + fire as ECS consumer (VSS-T3)

**Authority:** [`guide_sim_effect_spine_v1.md`](guide_sim_effect_spine_v1.md) — fire is consumer, not spine.

| Slice | ID | Owner | Deliverable |
|:---|:---|:---|:---|
| P0 | VSS-T3-001 | @coder | Single ignition funnel via `EmberSpotIgnitionEvent` |
| P1 | VSS-T3-002 | @sim-steward | **SHIPPED** — sole overlay writer `sync_shared_overlay_from_simulation` |
| P2 | VSS-T3-003 | @coder | Save round-trip fire state hash |
| P3 | VSS-T3-004 | @cleanup-intelligence | DEBT registry from effects plan — delete stubs |

**Witness:** `debug_runs/sim_effect_fire_consumer_live.json`

---

## Track 4 — Independent artist VFX toolchain (VSS-T4)

**Separate lane** from sim code (MCP art pipeline pattern).

```text
EffectSpec → validate → bpy/WGSL pack → staging → promote
  → assets/effects/registry/
  → EffectConsumable (Bevy)
  → preview tool (G4) + in-engine Effect Sandbox view
```

| Slice | ID | Owner | Deliverable |
|:---|:---|:---|:---|
| P0 | VSS-T4-001 | @planner-mcp | `tools/mcp/schemas/effect_spec_v1.schema.json` |
| P1 | VSS-T4-002 | @designer-mcp | 3 reference effects (spark, smoke, rain) |
| P2 | VSS-T4-003 | @coder-mcp | **SHIPPED** — validate + promote + registry on disk |
| P3 | VSS-T4-004 | @coder | **SHIPPED** — `EffectConsumableRegistry` + spawn_hook resolver |
| P4 | VSS-T4-005 | @designer | APS browse/preview/assign panel |

**Witness:** `debug_runs/artist_vfx_pipeline_live.json`

**Trip:** `assets/shaders/fire/*`, `src/render/fire_vfx/*` — must not add parallel draw path; extend consumable registry.

---

## Track 5 — Reflection, tribunals & symbolic build encoding (VSS-T5) — ~1/7 fleet

**Problem:** Green builds without critique; repeated token spend on raw logs.

### Tribunal model

```text
artifact_touch(path, program_id, agent)
  → tribunal_packet.yaml (debug_runs/tribunals/)
      verdict: ship | revise | dissent
      majority: { ... }
      dissent[]: { agent, reason, proposed_change }   # ALWAYS ALLOWED
      complexity_budget: { Q, C, E }
      routing: @orchestrator | @planner | @sim-steward
```

**Policy:** Majority cannot delete dissent rows. Orchestrator resolves in HANDOFF.

### Symbolic build encoding

All **mandatory** builds (`cargo check`, `cargo test`, `validate-report`) SHOULD produce:

1. `ValidationReport` JSON (compression 3–4) — existing
2. `SymbolicBuildDigest` — SYMLANG-coded issue list + trip hits — **new**

Tool: MCP `symbolic_build_digest_tool` · CLI via `validate-report` extension · Guide: [`symbolic_build_output_encoding_v1.md`](../../prompts/guides/symbolic_build_output_encoding_v1.md)

| Slice | ID | Owner | Deliverable |
|:---|:---|:---|:---|
| P0 | VSS-T5-001 | @coder-mcp | `symbolic_build_encoder.py` + MCP tool |
| P1 | VSS-T5-002 | @operations-intelligence | Postgres witness index spec (dev-only) |
| P2 | VSS-T5-003 | @coder-mcp | `tribunal_on_touch` MCP stub |
| P3 | VSS-T5-004 | @operations-intelligence | Grafana export from `ops_dashboard_snapshot` |

**Witness:** `debug_runs/symbolic_build_encoding_live.json`

---

## Sub-tribunaries (trip on touch)

When an agent edits a path in `vss_tribunal_touch_registry.json`:

1. Run `symbolic_build_digest_tool` if build touched
2. Read trip markdown stub linked from registry
3. File `tribunal_packet` if verdict ≠ ship OR dissent
4. Update witness before Q✓

---

## Phase pick order (orchestrator)

```text
Wave 0 (parallel OK):
  VSS-T5-001 symbolic encoder (coder-mcp)
  VSS-T1-001 scale contract doc (planner)
  VSS-T4-001 EffectSpec schema (planner-mcp)

Wave 1 (serial authority):
  VSS-T2-001 harness demotion (coder) — after T5 encoder lands
  VSS-T2-002 scenario ignite (coder)
  VSS-T3-001 ignition funnel (coder) — same session as T2-002 ONLY if sim-steward clears dual-write map

Wave 2:
  T4 reference effects + T2 spectator + T1 code constants
```

**Conflict matrix:** T2 + T3 + fire raster = **SERIAL** (same overlay authority). T4 MCP = **PARALLEL OK** with T1.

---

## Queue & HANDOFF

Machine queue: [`vss_program_queue.json`](../../tools/orchestrator/queues/vss_program_queue.json)

HANDOFF lease: § **PLAN-VFX-SPECTATOR-SCALE-001**

---

## Exit predicates (program close)

| Gate | Witness |
|:---|:---|
| Scale contract published + live | `world_scale_contract_live.json` |
| Scenario ignite works without harness | `spectator_parity_live.json` |
| EffectSpec promote + in-world spawn | `artist_vfx_pipeline_live.json` |
| Symbolic digest on all agent builds | `symbolic_build_encoding_live.json` |
| Tribunal registry honored (ops scan) | `tribunal_hygiene_live.json` |

---

## @orchestrator-status

`@orchestrator-status: ACTIVE`  
`@orchestrator-owner: @orchestrator`  
`@orchestrator-do-not-cleanup: trip registries, tribunal stubs, symbolic encoder until VSS exit gates green`
