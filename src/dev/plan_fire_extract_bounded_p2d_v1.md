# PLAN-FIRE-EXTRACT-BOUNDED-P2D — Residency-bounded fire extract `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-FIRE-EXTRACT-P2D-001** |
| **Parent** | [`plan_visual_perf_production_exec_001_v1.md`](plan_visual_perf_production_exec_001_v1.md) **P2-D** |
| **Authority** | [`07-repo-authority-map.md`](../../.cursor/skills/bevy-simulation-grade/07-repo-authority-map.md) · [`steward_fire7_preflight_gate_v1.md`](steward_fire7_preflight_gate_v1.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-06-07 |
| **Owner** | `@planner` → **`@coder`** |
| **Status** | **READY** — implementation slices below |
| **Non-goal** | Smaller test worlds — cost must scale with **resident + hot** chunks, not map tile count |

**Do not re-open:** FIRE7 per-view authority (`FireVisualFramesByView`), single ECS fire read contract, `RenderProjectionGraph` ownership.

---

## 1. Executive summary

Fire **gameplay** is sparse (dozens of hot cells on a 320×320 map). Fire **render extract** is not: `extract_fire_simulation_snapshot` iterates `Query<Chunk, …>` over **all loaded chunks**, rebuilds `FireSimulationSnapshot` + `FireChunkRuntime`, then ~10 downstream systems consume that snapshot (overlay, LOD, lights, particles, projection, witnesses).

**Target:** Extract cost **∝ |scan_set|** where `scan_set = residency ∪ active ∪ warm-rim ∪ dirty-queue`, not **∝ world_chunks**. Full-world reconcile remains available on a **slow cadence** for correctness drift recovery.

**Acceptance (operator + harness):** `FireExtractFrameReport.chunks_iterated` ≈ `|scan_set|` (not thousands); `extract_ms` p95 **< 8 ms** steady on 320×320 release sim; witnesses green (`fire_ecology_live.json`, `stage5_full_app_live.json`, `fire_streaming_live.json`).

---

## 2. Problem statement (validated)

| Symptom | Evidence | Root cause |
|:---|:---|:---|
| ~200ms+ frames / `substage_fire_sim_snapshot` stalls | User logs, PERF-INSTR-VFX-001 | Full ECS chunk scan + bundled stall wall time |
| Perf scales with map load, not fire intensity | 28 fire cells vs 102k tile entities | `for chunk in &q` visits every `Chunk` entity |
| Residency scoping ineffective when table empty | `scope_residency = table.is_some_and(!empty)` | Empty `ChunkResidencyTable` → full scan |
| Cadence helps but does not fix architecture | `FireExtractCadence` shipped (P2-C partial) | Still O(world_chunks) when scan runs |
| Downstream amplifies extract | `fire_visual_extract.rs` plugin graph | One snapshot feeds overlay, LOD, VFX, graph |

**Clarification for attribution:** `STALL substage_fire_sim_snapshot` measures wall time since the **previous** stall checkpoint (includes `view_sync` + sleep/wake + extract). Use `FireExtractDiagnostics` + `PerfScope::upd_fire_sim_snapshot` for isolated extract ms — not stall labels alone.

---

## 3. Goals and non-goals

### Goals

1. **Bounded scan** — default path touches only `scan_set` chunk coords per extract tick.
2. **Incremental runtime** — `FireChunkRuntime` retains cold chunks; update hot coords + eviction policy.
3. **Residency authoritative** — `ChunkResidencyTable` always non-empty in Simulation (S6-12); extract respects it.
4. **Overlay delta** — `SharedOverlayFieldBuffers::chunk_fire_heat` patches from changed rows, not full map rebuild every tick.
5. **Witness-visible** — `FireExtractFrameReport` exposes `scan_set_len`, `bounded_path`, `full_reconcile`.
6. **Large-map safe** — design holds for 512×512+ (more chunks, same resident window).

### Non-goals

- Smaller `WorldGenParams` for test harness.
- Second parallel ECS fire read path (violates FIRE7 preflight).
- Rewriting fire sim (`ChunkSurfaceFire` systems) — extract/render only.
- Substrate slab authoritative fire in this slice (Phase 6 hook only if cutover already green).
- Removing `FireSimulationSnapshot` — it stays the sim→render contract; we change **how** it is filled.

---

## 4. Authority map (must preserve)

| Domain | Sole writer | Readers | Rule |
|:---|:---|:---|:---|
| ECS `ChunkSurfaceFire` etc. | Fire sim systems | **`extract_fire_simulation_snapshot` only** | No new ECS readers |
| `FireSimulationSnapshot` | `extract_fire_simulation_snapshot` | overlay sync, LOD, view extract, projection | Snapshot may be **merged** incremental, not alternate writer |
| `FireChunkRuntime` | `extract_fire_simulation_snapshot` + `apply_fire_streaming_sleep_wake_system` | `ActiveFireChunkSet`, streaming witness | Sleep/wake runs **after** extract; bounded extract must not fight sleep semantics |
| `SharedOverlayFieldBuffers` | `sync_shared_overlay_from_simulation` | minimap compositor, world tint | Patch from snapshot deltas |
| `FireVisualFramesByView` | `build_fire_visual_frames_by_view` | projection graph, particles | Unchanged |
| `ChunkResidencyTable` | `sync_chunk_residency_from_scheduler` | fire extract, streaming, stage6 | Extract **reads**; fix population in Phase 1 |

```text
ChunkSurfaceFire (sim)
  → extract_fire_simulation_snapshot  [BOUNDED: scan_set + ChunkFireEntityIndex]
  → FireSimulationSnapshot + FireChunkRuntime
  → sync_shared_overlay_from_simulation (delta)
  → sync_fire_chunk_lod / sync_visible_fire_chunks / build_fire_visual_frames_by_view
  → RenderProjectionGraph → GPU
```

---

## 5. Target architecture

### 5.1 Scan set (per extract tick)

```rust
// Pseudocode — implement in fire_extract_scan.rs or fire_chunk_runtime.rs
fn build_fire_extract_scan_set(
    residency: &ChunkResidencyTable,
    runtime: &FireChunkRuntime,
    prev_snapshot: &FireSimulationSnapshot,
    dirty_queue: &FireExtractDirtyQueue,
    full_reconcile: bool,
) -> FxHashSet<ChunkCoord> {
    if full_reconcile {
        return FxHashSet::default(); // sentinel: use legacy full query path
    }
    let mut set = FxHashSet::default();
    for coord in residency.entries.keys() {
        set.insert(*coord);
    }
    for coord in runtime.chunks.keys() {
        let c = &runtime.chunks[coord];
        if c.active || c.visual_active || c.dirty {
            set.insert(*coord);
        }
    }
    for h in &prev_snapshot.chunk_heat {
        if h.heat > FIRE_VISUAL_ACTIVE_HEAT_EPS {
            set.insert(h.chunk);
        }
    }
    for coord in &dirty_queue.coords {
        set.insert(*coord);
    }
    // 1-chunk Moore rim of any coord already in set (matches neighbor_glow policy)
    expand_moore_rim_one(&mut set);
    set
}
```

**`full_reconcile` when:**

- `FireExtractClock.full_reconcile_due` (slow wall-clock, e.g. 30 s sim time, harness 60 s).
- `ChunkResidencyTable` empty (transition / bug — log warn once).
- Explicit dev flag `FIRE_EXTRACT_FULL=1` (debug only).
- World-gen / chunk count revision bump (`ChunkFireEntityIndex.revision`).

### 5.2 Chunk entity lookup (new)

**Problem:** Bounded coords are useless without `ChunkCoord → Entity` lookup. Today extract uses `Query` iteration.

**New resource:**

```rust
#[derive(Resource, Default)]
pub struct ChunkFireEntityIndex {
    pub by_coord: FxHashMap<ChunkCoord, Entity>,
    pub revision: u64,
}
```

**Writers (single maintenance lane):**

| Event | Action |
|:---|:---|
| Chunk spawned (materialize / streaming hydrate) | `index.insert(coord, entity)` |
| Chunk despawned | `index.remove(coord)` |
| World regen / `despawn_generated_world` | `index.clear(); revision++` |

**Preferred hook point:** `materialize_chunks` spawn path + streaming apply that creates `Chunk` entities (`src/systems/terrain/material_plugin.rs`, `src/io/streaming/mod.rs`). Do **not** scan all entities each frame to rebuild index.

**Extract path:**

```rust
for coord in scan_set {
    let Some(entity) = index.by_coord.get(&coord) else { continue };
    let Ok(bundle) = chunk_query.get(*entity) else { continue };
    // profile chunk from bundle components
}
```

Use a typed `Query` with `get(entity)` — still one ECS read site, no full-world iterator.

### 5.3 Incremental snapshot merge

On **bounded** ticks (not full reconcile):

1. Start from previous `FireSimulationSnapshot` / `FireChunkRuntime` (clone or in-place merge).
2. For each `coord ∈ scan_set`: recompute profile → update `runtime.chunks`, replace rows in `sim.instances` / `sim.chunk_heat` for that coord.
3. For coords that **cooled** below eps in scan_set: remove from instances/heat vectors (or mark inactive).
4. Run existing `neighbor_glow` + rim decay **only on `runtime.chunks` keys in scan_set ∪ neighbors** (not full runtime map scan — see Phase 4).

On **full reconcile:** keep current clear + full query behavior as fallback.

### 5.4 Overlay delta

`sync_shared_overlay_from_simulation` today rebuilds overlay map from full `sim.chunk_heat`.

**Change:**

- Resource `FireOverlayRevision` or diff list produced by extract: `Vec<(ChunkCoord, f32)>` changed this tick.
- `sync_shared_overlay_from_simulation` applies patches; calls `shared.bump()` only if `chunk_fire_heat_maps_differ` on changed keys.
- Preserve PLAY-06c/06d hold semantics (empty snapshot must not wipe overlay).

### 5.5 Cadence policy (ship — extends P2-C)

| Mode | `full_scan_on_sim_tick` | Min interval | Bounded default |
|:---|:---:|:---:|:---:|
| Editor / default budgets | true | overlay_hz | full query until residency populated |
| Simulation play | false | ≥ 1.5 s @ 320² | **bounded** |
| `--test visual` harness | false | ≥ 3 s @ 320² | bounded + full reconcile every 60 s |
| `UxFrameSpikeGuard` active | — | ×2.5 multiplier | bounded only |

Already partially in `FireExtractCadence::clamp_for_world` — **wire `full_reconcile` flag from cadence**, do not conflate with `cadence_due`.

---

## 6. Phased execution (coder slices)

### Phase 0 — Baseline instrumentation (0.5 day)

**Status:** **SHIPPED** (PR-1)

| Task | File | Done when |
|:---|:---|:---|
| Extend `FireExtractFrameReport` | `visual_perf_budget.rs` | Fields: `bounded_path`, `scan_set_len`, `full_reconcile`, `index_len`, `residency_len` |
| Log one-line summary at `info` when `extract_ms > 16` | `fire_visual_extract.rs` | `bounded=false scan=4000 residency=0` style |
| Document stall vs `upd_fire_sim_snapshot` | comment in `stall_watch.rs` | Prevents future mis-attribution |

**Tests:** unit test report serialization; no behavior change.

**Exit:** `--test visual` logs show `chunks_iterated` vs `scan_set_len` side by side.

---

### Phase 1 — Residency gate (S6-12) (0.5 day)

**Status:** **SHIPPED** (PR-1)

| Task | File |
|:---|:---|
| Assert non-empty residency in Simulation after enter | `sync_chunk_residency_from_scheduler` consumer witness |
| Warn once if extract sees empty residency post–frame 120 | `extract_fire_simulation_snapshot` |
| Harness proof: `residency_chunk_count > 0` in `sim_spectrum` / deep debug | `sim_spectrum_analytics.rs` |

**Exit:** `ChunkResidencyTable.entries.len() >= 9` (focus window) in steady sim witness.

**Do not touch:** scheduler priority semantics.

---

### Phase 2 — `ChunkFireEntityIndex` (1 day)

**Status:** **SHIPPED** (PR-2)

| File | Change |
|:---|:---|
| **NEW** `src/render/fire_chunk_entity_index.rs` | Resource + insert/remove/clear API |
| `src/render/mod.rs` | re-export |
| `src/systems/terrain/material_plugin.rs` | `index.insert` on chunk spawn |
| `src/io/streaming/mod.rs` | `index.insert` on hydrate spawn (if distinct path) |
| World despawn hook | `index.clear` on world regen (`world_generator_enhanced` despawn) |

**Tests:**

- Spawn 3 chunks → index len 3
- Despawn one → len 2
- Regen clears index

**Exit:** `index_len` matches chunk entity count in witness.

---

### Phase 3 — Bounded extract path (1.5 days) **P2-D core**

**Status:** **SHIPPED** (PR-3) — MVP bounded path; overlay delta (Phase 5) + post-pass bounds (Phase 4) still open

| File | Change |
|:---|:---|
| **NEW** `src/render/extraction/fire_extract_scan.rs` | `build_fire_extract_scan_set`, `expand_moore_rim_one` |
| `fire_visual_extract.rs` | Branch: `full_reconcile` → legacy `&q` loop; else index `get` loop |
| `fire_chunk_runtime.rs` | Helper: `merge_chunk_profile_into_snapshot` |
| `visual_perf_budget.rs` | `FireExtractDirtyQueue` resource (coords from sim events — see §6.4) |

**Schedule:** No change to `FireVisualFrameSet` order — still after `SyncViewManager`.

**Tests (lib):**

- `scan_set` includes residency + active runtime coord
- Bounded path with 100 coord index, scan_set 12 → `chunks_iterated == 12`
- Full reconcile flag runs legacy path (mock query count)

**Regression:** `cargo test -p proc_A_dine01 --lib fire` + `vt1_full_world_fire_extract_tests` still pass on **full reconcile** fixture.

---

### Phase 4 — Bound post-pass loops (0.5 day)

**Status:** PLANNED

Today after main loop: `neighbor_glow`, rim decay iterate **all** `runtime.chunks`.

| Change |
|:---|
| Restrict glow/decay to `scan_set ∪ moore_neighbors(scan_set)` |
| `apply_fire_streaming_sleep_wake_system`: iterate only `visual_active` chunks (already partial) — add early exit when `runtime.chunks.len()` huge but few visual_active |

**Exit:** `extract_ms` drops on idle map (no fire) even when `runtime.chunks` retains history.

---

### Phase 5 — Overlay delta sync (0.5 day)

**Status:** PLANNED

| File | Change |
|:---|:---|
| `fire_visual_extract.rs` | Emit `FireOverlayDelta` resource each bounded tick |
| `sync_shared_overlay_from_simulation` | Apply delta; preserve hold/warmup logic |

**Tests:** PLAY-06c empty snapshot hold; single-cell heat change bumps revision once.

---

### Phase 6 — Sim event dirty queue (1 day, can parallel Phase 5)

**Status:** PLANNED (optional for Phase 3 MVP — residency + runtime dirty sufficient initially)

When fire sim mutates `ChunkSurfaceFire` / overlay, push coord to `FireExtractDirtyQueue` via:

- Observer on component change (Bevy 0.18 observers), **or**
- Central fire step system after sim tick (preferred single writer)

**Exit:** New ignition off-residency still picked up within 1 bounded tick without full reconcile.

---

### Phase 7 — Substrate slab fast path (DEFER unless cutover green)

**Status:** DEFERRED

When `EcsRetireState.cutover_complete && !hybrid_fire_authoritative`:

- Read heat from `WorldSubstrateRegistry` for `scan_set` only
- Skip `ChunkSurfaceFire` component reads

Gate behind witness in `debug_runs/substrate_*` — do not ship until sim-steward signoff.

---

## 7. PR plan (≤3 files per PR)

| PR | Phases | Files (max 3 + mod.rs) |
|:---:|:---|:---|
| **PR-1** | 0 + 1 | `visual_perf_budget.rs`, `fire_visual_extract.rs`, `sim_spectrum_analytics.rs` |
| **PR-2** | 2 | `fire_chunk_entity_index.rs`, `material_plugin.rs`, `render/mod.rs` |
| **PR-3** | 3 | `fire_extract_scan.rs`, `fire_visual_extract.rs`, `fire_chunk_runtime.rs` |
| **PR-4** | 4 + 5 | `fire_visual_extract.rs`, `fire_streaming.rs`, overlay sync fn |
| **PR-5** | 6 | fire sim hook + dirty queue (planner slice after PR-3 validated) |

---

## 8. Verification matrix

### Commands

```powershell
cargo test -p proc_A_dine01 --lib visual_perf_budget
cargo test -p proc_A_dine01 --lib fire
cargo test -p proc_A_dine01 --lib stage5
.\tools\orchestrator\scripts\run_visual_test_clean.ps1 -Release
```

### Acceptance metrics (320×320 release, 60 s sim window)

| Metric | Before (observed) | Target |
|:---|:---|:---|
| `extract_ms` p95 | 50–200+ ms | **< 8 ms** bounded ticks |
| `chunks_iterated` p95 | ~world_chunks | **< 2 × residency_len + active** |
| `ran_full_scan` rate | every cadence tick | **≤ 1 / 30 s** (+ harness 60 s) |
| Frame p95 | 650–900 ms | **< 100 ms** (extract alone won't hit 33 ms — other lanes remain) |
| `fire_ecology_live.json` | green | unchanged |
| `stage5_full_app_live.json` | green | unchanged |

### Witness keys to add

| File | Keys |
|:---|:---|
| `FireExtractFrameReport` / sim-spectrum | `bounded_path`, `scan_set_len`, `full_reconcile`, `index_len` |
| `debug_runs/deep_debug/engine_deep_debug_live.json` | `fire_extract.bounded_path`, `fire_extract.scan_set_len` |

---

## 9. Risks and mitigations

| Risk | Mitigation |
|:---|:---|
| Missed hot chunk off residency | Moore rim + `dirty_queue` + periodic `full_reconcile` |
| Index stale after chunk respawn | `revision` bump on world regen; debug assert `index_len` ≈ chunk query count |
| Incremental snapshot drift | Full reconcile slow path; compare digest in witness |
| FIRE7 dual ECS read | **Forbidden** — index `get` only inside extract |
| Sleep/wake vs incremental runtime | Run sleep/wake after extract; extract respects `visual_active` flags |
| Test expects full-world rows | `vt1_full_world_fire_extract_tests` use `full_reconcile` fixture flag |

---

## 10. File ownership (@coder)

| Lane | Owns | Do not touch |
|:---|:---|:---|
| **fire extract** | `fire_visual_extract.rs`, `fire_extract_scan.rs`, `fire_chunk_entity_index.rs`, `fire_chunk_runtime.rs`, `visual_perf_budget.rs` | `build_fire_visual_frames_by_view` policy |
| **terrain spawn** | `material_plugin.rs` spawn hook only | pass pipeline rules |
| **streaming** | `io/streaming/mod.rs` hydrate insert only | scheduler budgets |
| **overlay** | `sync_shared_overlay_from_simulation` delta | compositor shaders |
| **witness** | `sim_spectrum_analytics.rs`, `FireExtractDiagnostics` | stage5 predicate semantics |

Read before edit: `.cursor/skills/bevy-simulation-grade/07-repo-authority-map.md`, `tools/orchestrator/agents/stage5_readiness_agent.md`, `steward_fire7_preflight_gate_v1.md`.

---

## 11. Status ledger

| Item | Status |
|:---|:---:|
| P2-C `FireExtractCadence` + spike multiplier | **SHIPPED** |
| P2-D bounded scan set + entity index | **SHIPPED** (Phases 0–3) |
| Overlay delta | **PLANNED** |
| Sim dirty queue | **PLANNED** (Phase 6) |
| Substrate slab extract | **DEFERRED** |
| Smaller test world | **REJECTED** |

---

## 12. Coder handoff checklist

- [x] Phase 0 instrumentation merged — baseline `chunks_iterated` captured in witness
- [x] Phase 2 index populated in real worldgen run (`index_len > 0`)
- [x] Phase 3 bounded path default in Simulation (when residency populated + not full reconcile)
- [x] `full_reconcile` exercised in tests at least once
- [ ] `cargo test -p proc_A_dine01 --lib fire` green
- [ ] `run_visual_test_clean.ps1 -Release` — `extract_ms` p95 logged < 8 ms on bounded frames
- [ ] No new ECS fire component readers (grep `ChunkSurfaceFire` outside sim + extract)
- [ ] Update this doc §Status ledger to **SHIPPED** per phase

**Planner sign-off:** Ready for `@coder` — start **PR-1** (instrumentation + residency witness), then **PR-2** (index), then **PR-3** (bounded extract core).
