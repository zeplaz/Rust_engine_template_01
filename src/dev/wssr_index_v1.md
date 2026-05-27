# World Simulation Substrate Refactor — index `v1` (WSS-PLAN-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **WSS-PLAN-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** |
| **Parent vision** | User WSS memo (2026-05-26) · [`vfx_architecture_bevy_wgpu_v1.md`](../prompts/guides/vfx_architecture_bevy_wgpu_v1.md) · [`base_fire2_smoke.md`](../prompts/guides/base_fire2_smoke.md) |

**No Rust in planner deliverables.** Code snippets in child plans are **target architecture** — expand during implementation; do **not** treat them as bare-minimum final shape.

---

## North star

The engine converges on a **World Simulation Spine (WSS)** — persistent, chunked, saveable physical world-state — **not** disconnected feature silos (`OceanSystem`, `DustSystem`, `WeatherParticleSystem`).

```text
WORLD SUBSTRATE (L1 — simulation authority)
    terrain · hydrology · geology · atmosphere · contamination
    ecology · deformation · fire · smoke · weather · logistics pressure

        ↓ extraction (L2 — immutable per frame)

REPRESENTATION GRAPH
    RenderProjectionGraph · RepresentationResult · ViewManager · per-view frames

        ↓ GPU (L3 — transient visualization only)

VISUALIZATION
    volumetrics · custom compute fields · particles · Hanabi (scoped) · lighting
```

**Engine identity target:** Dwarf Fortress persistence + W&R industrial logistics + Qud ecology + wargame command overlays + planetary operational visualization — **not** traditional RTS feature lanes.

---

## Child plans (signed)

| Queue ID | Document | Scope |
|:---|:---|:---|
| **WSS-PLAN-002** | [`wssr_plan_002_chunk_authority_v1.md`](wssr_plan_002_chunk_authority_v1.md) | Chunk slabs, hybrid ECS, writers/readers/persist matrix |
| **WSS-EXEC-001** | [`plan_wss_chunk_slab_exec_001_v1.md`](plan_wss_chunk_slab_exec_001_v1.md) | **WSS-CHUNK-SLAB-001** — blocked on [`wss_design_gate_001_v1.md`](wss_design_gate_001_v1.md) |
| **WSS-PLAN-003** | [`wssr_plan_003_hydrology_runtime_v1.md`](wssr_plan_003_hydrology_runtime_v1.md) | Hydrology as terrain-coupled substrate; event-driven runtime |
| **WSS-PLAN-004** | [`wssr_plan_004_atmosphere_unification_v1.md`](wssr_plan_004_atmosphere_unification_v1.md) | Atmosphere clipmaps, contamination split, smoke/dust/weather spine |

---

## Locked architectural decisions (2026-05-26)

| Topic | Decision | Rationale |
|:---|:---|:---|
| **Chunk granularity** | **Resource slabs keyed by chunk first**; ECS `ActiveChunkRuntime` entities **later** for hot regions only | Avoid archetype churn, serialization pain, authority drift while substrate evolves |
| **Weather / atmosphere grid** | **Hierarchical simulation clipmaps** (L0–L3); **separate render clipmaps** | Fixed 128² breaks planetary + tactical + minimap scale; sim resolution ≠ render resolution |
| **Hydrology runtime** | **`ChunkSlab<HydrologyState>`** + scheduled active-chunk jobs; **NOT** one giant `FluidDomain` | Streaming boundaries, parallelism, save/load locality, terrain coupling |
| **Contamination** | **Separate `ContaminationState`** with **`AtmosphereCoupling`** bridge; **NOT** merged into `AtmosphereCell` | Contamination lives in soil, water, structures, ocean — not only airborne |
| **Hanabi** | **Audit compatibility NOW** (`experiments/hanabi_validation/`); **adopt LATER** as Layer 3 render consumer only | Early Bevy 0.18 / extraction / multiview answers without blocking substrate refactor |
| **Ocean / water VFX** | Ocean = **`HydrologyState`** subsystem; water GPU = Layer 3 only | No separate ocean renderer authority |
| **Smoke** | Layer A sim field (persistent) + Layer B GPU representation (transient) | See WSS-PLAN-004 |
| **Dust** | **`AtmosphereField` transport + `ContaminationState.soil/airborne`**; vehicle kicks = events → field, not silo VFX | Storms, deserts, logistics readability |
| **F2 hot-cell extract** | **Deferred** until projection graph + per-view extraction stable (Phase 4 gate in prior planner memo) | Avoid repeated extraction rewrites |

---

## Three-layer rule (mandatory language)

```text
X OWNS authority (L1 persistent state)
Y DERIVES from X (L2 extraction / RepresentationResult)
Z CONSUMES immutable snapshot (L3 GPU / Hanabi / composites)
```

**Forbidden:** GPU field or Hanabi particle state driving gameplay without explicit readback contract.

---

## Library matrix

| Library | Verdict | Layer |
|:---|:---|:---|
| `bevy_ecs_tilemap` | **KEEP** | Presentation / tile sync |
| `bevy_egui` | **KEEP** | Tooling / debug |
| `bevy_vector_shapes` | **ADD** (high) | L2/L3 tactical wire overlays, authority graphs |
| `bevy_mod_outline` | **ADD** (medium) | L3 selection / tactical highlight |
| `bevy_hanabi` | **CONDITIONAL** — spike now, integrate later | L3 local/event particles only |
| `bevy_vfx_bag` | **SKIP** as architecture | — |
| Custom wgpu compute | **REQUIRED** | L3 fields + volumetrics; evolve from `gpu_weather_fire_field` |

---

## Team routing (parallel infrastructure)

| Team | Owns | Does not own |
|:---|:---|:---|
| **A — Terrain simulation spine** | substrate slabs, hydrology, geology, deformation, chunk paging | GPU particles, Hanabi |
| **B — Atmosphere + fields** | clipmaps, smoke/dust/weather sim, contamination coupling, compute fields | ViewManager |
| **C — View + extraction** | RenderProjectionGraph, RepresentationResult, per-view isolation | L1 sim writers |
| **D — Gameplay + construction** | build, logistics, player interaction | atmosphere grid writes |

---

## Implementation wave order (recommended)

```text
WSS-PLAN-001 index (this doc)           ☑ SIGNED
WSS-DESIGN-GATE-001                   ☑ SIGNED — [`wssr_design_signoff_v1.md`](wssr_design_signoff_v1.md)
WSS-PLAN-002 chunk authority            ☑ SIGNED — coder: WSS-CHUNK-SLAB-001 (design gate PASS qualified)
WSS-PLAN-004 atmosphere unification     ◐ parallel with 002 after slab types
WSS-PLAN-003 hydrology runtime          ◐ after 002 slab types + hydrate design
Hanabi validation spike                 ◐ non-blocking (experiments/) — design signoff on scope
Extraction stability gate               ◐ blocks F2 hot-cell (F-T02, VM-08)
bevy_vector_shapes adoption             ◐ tactical overlay slice
```

**Design-before-code policy (2026-05-26):** [@designer](wssr_design_gate_brief_v1.md) evaluates identity/ethos and mandates **hybrid** where incumbent systems are superior · [@coder](wssr_coder_hybrid_orders_v1.md) must file Hybrid Assessment per slice.

**Policy:** Do **not** open new top-level `*OceanSystem*`, `*DustSystem*`, or `*WeatherVfxSystem*` modules — extend WSS domains.

---

## Witness target (new)

| Path | Purpose |
|:---|:---|
| `debug_runs/wss_substrate_live.json` | Cross-domain single-writer checks, slab paging, coupling invariants |

Envelope: [`debug_run_envelope.rs`](debug_run_envelope.rs) — add path when first coder slice lands.

---

## Regression guards (always on)

- **FIRE7-PLAN-001** per-view fire extract — do not regress
- **Stage 5 FULL_APP** spine — WSS work is **not** Stage 5 gate
- **`fire_ecology_live.json`** — F1 sim truth orthogonal
- **VFX-P2 / FX-WATER closure** — maintain tactical witness; hydrology refactor **replaces** ocean silo language in queues, not witness rollback

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | Index + three child plans signed |
