# Visual dev plan & status (split from `base_visual_dev01.md`)

**Parent doc:** [`base_visual_dev01.md`](base_visual_dev01.md) — full rationale, sketches, and code examples stay there.

**Companion:** [`base_visual_dev01_roadmap_next.md`](base_visual_dev01_roadmap_next.md) — what to do after this checklist, north-star sequencing.

**Representation / GPU spine (strict order + DONE language):** [`base_visual_world_representation_v1.md`](base_visual_world_representation_v1.md) — dependency-locked sequence **theme-3 → phase-e (E1) → VT-4 → phase-d → phase-f**; “no fake GPU” rules; target `WorldRepresentationResolver` model.

**How to use:** Keep items **rough**; flip status when behavior matches the intent (not when every sub-bullet exists). Update dates in the status column when you touch an item.

**2026-05-13 — spine slices landed; convergence target:** Gates **1–5** (policy spine, burst discipline, snapshot fence, VT-4/VT-5 harness, GPU metrics HUD) and post-gate scaffolds (zones in `apply_zone_policy`, Phase D contract + offscreen camera, Phase F registry upload + **LOD proof**, **P2-H hybrid reconcile** with authoritative GPU partial texture uploads + partial compute dispatch, **stage5-08** readiness/HUD/CI, logistics/ecology snapshots) are **in-tree** with `cargo test --lib` green. **Strict EXIT** per [`base_visual_world_representation_v1.md`](base_visual_world_representation_v1.md) and **Stage 5** still require **full-app VT surfaces**, **Phase D pixel parity** (CPU layers vs GPU quads), and **Phase F instanced draw** — not more side paths.

---

## Representation domain resolution (unifying layer)

**Engine problem (one sentence):** given current gameplay + camera + importance + zones + budgets, **what form should every world datum exist in right now?**

`WorldLodBand`, `WorldRepresentationResolver`, `WorldLodMap`, `OverlayFieldFrame`, `FireVisualFrame`, and `RenderProjectionGraph` are **pieces of one layer** — **representation domain resolution** — not separate “fire LOD”, “preview LOD”, or “GPU LOD” products.

```text
                    ┌──────────────────────┐
                    │ CameraVisualState    │
                    └──────────┬───────────┘
                               │
                    ┌──────────▼───────────┐
                    │ LodZoneRegistry      │
                    └──────────┬───────────┘
                               │
                    ┌──────────▼───────────┐
                    │ Importance sources   │
                    │ AI / combat / events │
                    └──────────┬───────────┘
                               │
                    ┌──────────▼───────────┐
                    │ RepresentationPolicy │
                    └──────────┬───────────┘
                               │
                    ┌──────────▼───────────┐
                    │ WorldRepresentation  │
                    │ Resolver (authority) │
                    └──────────┬───────────┘
                               │
             ┌─────────────────┼─────────────────┐
             │                 │                 │
     ┌───────▼───────┐ ┌──────▼───────┐ ┌──────▼───────┐
     │ Render graph  │ │ Compute graph│ │ Overlay graph│
     │ (projection)  │ │ (dispatch)   │ │ (channels)   │
     └───────────────┘ └──────────────┘ └──────────────┘
```

**Current danger (anti-pattern):** fire, preview, GPU upload, camera, and particle paths each **decide LOD or fidelity** independently. That does not scale once AI compute, logistics, projectile zones, strategic abstraction, jump points, atmospheric cells, fog-of-war, and transport lanes need **independent scaling** under one budget.

**Invariant:** **one policy evaluation per frame** → full-fidelity CPU snapshots where required → **render / compute / overlay graphs read policy output only** → **shared** `GPUBufferRegistry` (not render-owned special cases per domain).

**Target inputs / outputs (contracts to converge on):**

| Direction | Types (evolving names) |
|-----------|-------------------------|
| **In** | `RepresentationInputs` — `CameraVisualState`, `LodZoneRegistry`, importance sources, `VisualBudgetSettings` / `VisualCadence`, `SimStepStamp` |
| **Out** | `RepresentationResult` — per-chunk bands (`ChunkRepresentationMap` / today `WorldLodMap`), overlay channel mask, `GpuBudgetResult` / capacity classes |

**Recommended renames over time (internal):** `FireVisualLod` → `RepresentationBand`; `FireVisualExtractPlan` → `WorldRepresentationExtractPlan` — fire is the first consumer, not the owner of the policy.

**Zone model (target, not only camera):** hierarchical **influence domains** — `LodZoneClass` (Tactical, Operational, Strategic, Projectile, Cinematic, Debug) with `LodZone { id, class, center, inner/outer radius, importance, min/max band, falloff }`. Sources: camera, combat, alerts, AI focus, sensors, missions, cinematics, projectiles, editor, debug — **not** RTS camera tunnel vision alone.

**Unified GPU work graph (not render-centric):** `SIM SNAPSHOTS` → **representation resolution** → **GPU projection graph** → **`GPUBufferRegistry`** → render + compute passes. **Do not** ossify `FireGpuStorage` / `SmokeGpuStorage` / `AiGpuStorage` as permanent parallel allocators; systems request **views/slices** on registered buffers (`BufferId`, generation, LOD-aware `current` / `reserved` capacity).

**Theme 3 card name in tracker:** treat as **world representation policy** (was “fire LOD”).

---

## Stage staging (convergence to Stage 5)

| Stage | Focus | Status (2026-05-13) |
|-------|--------|------------------------|
| **0** | P0 latency + single fire snapshot path | ✓ enough for current milestone |
| **1** | Policy scaffolding (`WorldLodMap`, zones, expanded `LodInputs`) | ✓ `world_representation.rs`, `lod_zone_authoring.rs`, `representation_policy.rs` |
| **2** | **Authoritative resolver** — one `RepresentationResult` / frame policy; consumers read policy not raw zoom | ~ **Gates 1–2** in code; grep audit + doc in `representation_spine_audit.rs` |
| **3** | **E1 + VT-4 strict** — `SimStepStamp`, snapshot fence, triple-surface agreement + VT-5 spatial | ~ fence + unit VT; **app-level** scene matrix open |
| **4** | **Projection + registry** — graphs + `GPUBufferRegistry` sole upload | ~ fire + heat + particle rows; **draw** + domain buffers open |
| **5** | **Scalable orchestration** — multi-domain snapshots, unified work graph | ~ logistics/ecology snapshot publish; projection nodes open |

**Next real milestone:** app-level **VT-4/VT-5** + **Phase D world pixels** + **Phase F draw** + **P2-H reconcile** + **Stage 5 domain projection** (see IDE todos `next-09` … `next-06`).

**Gates 1–5 (2026-05-13, in-tree):** `RepresentationInputs` → `WorldRepresentationResolver` / `build_representation_result` → `RepresentationResult` → `RenderProjectionGraph` / `ComputeDispatchGraph` → `GPUBufferRegistry`; `CommittedVisualSnapshotFence`; `FxParticleBurstRequest` projection-only; `GpuRepresentationMetrics` + HUD `REP` line.

**Still before declaring Stage 5:** instanced particle **draw**; GPU preview **world** pixels (not clear-color scaffold); atmosphere **hybrid reconcile**; logistics/ecology **projection nodes**; CI VT integration.

---

## Major engineering ladder (avoid permanent side architecture)

**Main risk:** shipping a **temporary** side path (quick coupling, duplicate truth, UI-as-renderer) that later **ossifies** into engine debt.

**Wrong order:** feature → implementation → discover coupling → debt.

**Required order for every major phase / theme / meta item** (check off explicitly in PRs):

| Step | Deliver |
|------|---------|
| **1. PHASE** | Goal boundary: what “done” means; what is explicitly *out* of scope. |
| **2. CONTRACTS** | Types + written invariants **before** behavior code (snapshots, cadence, ownership). |
| **3. DATA OWNERSHIP** | One authoritative producer per datum; consumers read agreed buffers only. |
| **4. SCHEDULE** | `SystemSet` edges, `run_if`, multirate cadence — no magic 12/20/60 Hz literals in hot paths once cadence resources exist. |
| **5. TEST SURFACE** | Unit + integration + edge cases **before** “making it fast”. |
| **6. DEBUG VISUALIZATION** | Counters, HUD/debug lines, buffer IDs, swap counts — **before** trusting full-frame pixels. |
| **7. KNOWN FAILURE MODES** | Document symptom → likely cause (spawn collapse, chunk mismatch, stale buffers, …) — speeds future debugging. |
| **8. OPTIMIZATION** | Only after 1–7 are stable (correctness → ownership → scheduling → tests → debug → failure catalog → then perf). |

Major roadmap work is **not** “build feature X”. It is: **phase → contracts → ownership → schedule → tests → debug viz → known failure modes → optimization** in that order.

---

### Why one PHASE todo — not `D-1`, `E1-2`, `T3-C` as separate *top-level* todos

Roadmap work here is **architecture-program** level (coherent contracts, ownership, cadence), not isolated bugfixes or one-off implementations.

**If** each sub-id (`D-1`, `E1-2`, …) becomes its **own** top-level tracker item, you tend to get: fragmented ownership, duplicated notes, lost invariants, **partial drift**, and **side-architecture regressions** (a “small” path ships and ossifies).

**Correct tracker shape:** **one top-level item per phase / theme / meta** (below). Put **`[D-1]` …** inside the **SUBTASKS** field of that single card — ordered; respect dependencies (e.g. E1-2 after E1-1 contracts; E2-3 after E2-2 ownership). Reviews, commits, and test runs stay **grouped under that phase**.

**Recommended top-level IDE / task list (order is suggestive; respect BLOCKED BY):**

1. `meta-no-temp-side-paths`
2. `meta-schedule-contracts`
3. `arch-generic-snapshot-layer`
4. **`theme-3-lod-overlay-channel-matrix`** *(world **representation policy** — authoritative resolver + graphs consume policy only)*
5. `theme-1-camera-intent-productization` *(feeds `RepresentationInputs`, does not own bands)*
6. `theme-2-worldfx-atmospherefx-input-buffers`
7. **`visual-test-matrix-upgrade`** *(VT-4 strict + spatial distribution; blocks declaring resolver done)*
8. `phase-e-cadence-scale` *(E1 before interpolated render authority)*
9. `phase-d-preview-render-target` *(convergence after resolver + VT-4; not parallel feature rush)*
10. `phase-f-gpu-particles` *(buffer slice ~; **draw / Hanabi after** resolver + registry strict gates)*

**Dependency-locked spine (overrides 4/7/8/9/10 relative order for GPU + representation work):** see [`base_visual_world_representation_v1.md`](base_visual_world_representation_v1.md) §1 — **`theme-3` → `phase-e` (E1 only) → VT-4 → `phase-d` (finish/converge) → `phase-f`**. Meta items **1–3** still apply in parallel when they do not add duplicate producers. **Stage 2–4** (§ *Stage staging*) are the practical convergence reading of that spine after the seven implementation slices.

---

## How to process a top-level todo (IDE / human / agent)

Use this **every time** you pick up one card above (not ad-hoc feature coding).

1. **Select** the next item using **recommended order (1→10)** and each card’s **BLOCKED BY**. If blocked, work the blocker card first or document why the block is lifted.
2. **Read the card** in this file: **GOAL**, **SUBTASKS** (`[D-3]` style — do these *inside* the same top-level todo), **EXIT CRITERIA**, **KNOWN FAILURE MODES**.
3. **Run the engineering ladder** (§ *Major engineering ladder*) **inside that scope only**:
   - **PHASE** — restate one-sentence boundary for this session (what is out of scope).
   - **CONTRACTS** — types / invariants / resource names; update module docs at the touch site if the contract changed.
   - **DATA OWNERSHIP** — grep for a **second producer** of the same buffer (see § *meta-no-temp-side-paths* audit); do not add a parallel path “just for now”.
   - **SCHEDULE** — `SystemSet` / `.before`/`.after` / `run_if` / `VisualCadence`; avoid new bare Hz literals where `VisualCadence` exists.
   - **TEST SURFACE** — `cargo test --lib` + add/adjust a test that would fail if the regression returns (VT-style asserts where applicable).
   - **DEBUG VISUALIZATION** — bump counters / HUD lines / `PreviewPresentationDebug` when behavior is timing- or swap-sensitive.
   - **KNOWN FAILURE MODES** — add or tick a line if you discovered a new symptom → cause pairing.
   - **OPTIMIZATION** — only if 1–7 are satisfied for this slice.
4. **Implement in one vertical slice**: touch the **fewest files** that satisfy the subtask; prefer extending existing plugins/systems (`view_representation.rs`, `fire_visual_extract.rs`, `world_preview/*`, `plan_status` for status only).
5. **Verify**: `cargo test --lib` (and `cargo build --bin world_generator` if GUI / `VisualCadence` / preview paths changed).
6. **Close the loop**: update this file’s **Repo status** / master table row for that item; update the **IDE todo** description (check off SUBTASKS text, not by spawning new top-level todos for `[D-1]`-style ids).

**Anti-patterns:** new minimap/preview/GPU upload path that bypasses `FireVisualFrame` / `SharedOverlayFieldBuffers`; new top-level Cursor todo per sub-id; merging without updating **EXIT CRITERIA** or tests.

---

### Universal phase card template (copy into every major item + PR body)

Use this **same heading set** for each top-level phase/theme above (fill or write “n/a”):

```text
GOAL
<one sentence outcome boundary>

SUBTASKS
[D-1] …   (ordered checklist; not separate top-level todos)

CONTRACTS
- …

DATA OWNERSHIP
- …

SCHEDULE
- …

TEST SURFACE
- …

DEBUG VISUALIZATION
- …

KNOWN FAILURE MODES
- …   (symptoms → likely causes; speeds debugging)

OPTIMIZATION
- only after CONTRACTS through KNOWN FAILURE MODES are stable

BLOCKED BY
- <ids> | none

EXIT CRITERIA
- measurable “done” checks
```

---

### `phase-d-preview-render-target`

**Repo status:** `✓` — **D-1 / D-2 / D-3 / D-4 / D-5** implemented: preview contract + `SwapImageBuffers` front/back; offscreen `Camera2d` → `RenderTarget::Image` on **back**; GPU present swaps to **front** for egui; CPU raster gated when GPU authoritative; startup promotes GPU when `WorldPreviewGpuRuntime::offscreen_renderer_ready`. **Strict** layer parity (height/moisture/ecology overlays on GPU) remains future optimization — biome + fire overlay quads are the authoritative GPU world pixels today.

**Spine note:** [`base_visual_world_representation_v1.md`](base_visual_world_representation_v1.md) §1 step **4** — treat **further phase-d primary expansion** (real world pixels, CPU no longer idle-default) as **after** **theme-3**, **E1**, and **VT-4** strict gates; existing D work is convergence debt toward that EXIT, not parallel “feature rush.”

**GOAL** — Replace CPU preview raster **ownership** with **Bevy `RenderTarget<Image>` → `SwapImageBuffers` → egui texture display only** (no egui-owned render loop).

**SUBTASKS** — `[D-1]` Preview ownership contract (`PreviewCameraState`, `PreviewRenderMode`, ≠ gameplay camera) · `[D-2]` `PreviewRenderTarget` resource (render **before** egui) · `[D-3]` `SwapImageBuffers` CPU double-buffer + `PreviewPresentationDebug::swap_count` ✓ · `[D-4]` egui display from `PreviewRenderTarget` + GPU clear-color offscreen camera (draw world into target still future) ✓ · `[D-5]` `PreviewRenderBudget` / cadence: `PreviewRenderBudget::max_hz` prefers [`VisualCadence`](../../src/gui/view_representation.rs) then [`VisualBudgetSettings`](../../src/gui/view_representation.rs).

**CONTRACTS** — egui never owns rendering; gameplay camera ≠ preview camera; preview must not rebuild textures every frame by default.

**DATA OWNERSHIP** — `PreviewCameraState` → preview transform; `PreviewRenderTarget` → GPU image handles; `SwapImageBuffers` → presentation swap.

**SCHEDULE** — correctness → swap → resize → cadence → dirty rects (order in ladder § above).

**TEST SURFACE** — resize stress; swap stability; stale texture / handle detection.

**DEBUG VISUALIZATION** — front/back image ids; swap count; update cadence; redraw reason flags.

**KNOWN FAILURE MODES** — tearing / partial frame upload; `TextureId` churn; preview coupled to `MapCameraDesired`; full CPU path still running in parallel “temporarily”.

**OPTIMIZATION** — dirty rects, async preview, GPU compositing — **after** exit criteria for D-1–D-4.

**BLOCKED BY** — none for **maintenance**; **strict “phase-d DONE”** sequencing: [`base_visual_world_representation_v1.md`](base_visual_world_representation_v1.md) §1 (after **theme-3**, **E1**, **VT-4**). Coordinate with `theme-2` for generic `SwapImageBuffers`.

**EXIT CRITERIA** — no full CPU preview rebuild on idle pan (where GPU path active); egui only displays stable texture handle; no per-frame texture registration churn; **strict phase-d DONE** = [`base_visual_world_representation_v1.md`](base_visual_world_representation_v1.md) §4 (Phase D) (real camera world render, swap presentation, egui display-only).

---

### `phase-e-cadence-scale` *(two tracks, one top-level todo)*

**GOAL** — Deterministic sim cadence + scalable atmosphere updates without schedule chaos or dual truth. **Strict E1 first slice** (struct stamp + `FireVisualFrame` + minimal `FixedUpdate`): [`base_visual_world_representation_v1.md`](base_visual_world_representation_v1.md) §4 (Phase E).

**SUBTASKS** — **TRACK E1 — FixedUpdate + interpolation:** `[E1-1]` `SimStepStamp` **struct** (`tick`, `sim_time`) + `RenderInterpolation` (today [`SimStepStamp`](../../src/systems/sim_control.rs) may still alias [`SimTick`](../../src/systems/sim_control.rs) — **E1 DONE replaces alias**) · `[E1-2]` minimal `FixedUpdate` vs `Update` split + ordering doc · `[E1-3]` snapshots on `FireVisualFrame` (render reads snapshots only) · `[E1-4]` interpolation pass after snapshots stable. **TRACK E2 — Incremental atmosphere:** `[E2-1]` diagnostics first · `[E2-2]` dirty regions · `[E2-3]` hybrid local + full reconcile · `[E2-4]` edge diffusion / merge / stale-region tests.

**CONTRACTS** — sim authoritative; render consumes **snapshots** only; interpolation explicit and bounded.

**DATA OWNERSHIP** — `SimStepStamp` (or companion clock), `RenderInterpolation`, snapshot-bearing `FireVisualFrame` / extract buffers.

**SCHEDULE** — `FixedUpdate` sim slice; `Update` camera/UI; render prepare/upload after extract contract documented.

**TEST SURFACE** — interpolation stability; cadence mismatch; E2 edge bleed / region merge / stale region.

**DEBUG VISUALIZATION** — tick / sim_time / alpha HUD; atmosphere dirty cell counts + full refresh counts.

**KNOWN FAILURE MODES** — interpolation jitter; sim/render tick desync; stale snapshot read; atmosphere partial update wrong at chunk boundaries.

**OPTIMIZATION** — wider incremental atmosphere only after E2-4 tests green.

**BLOCKED BY** — **E1** before declaring render “authoritative” on interpolated views: **`meta-schedule-contracts`** cadence vocabulary (`VisualCadence` ✓); **strict E1** per [`base_visual_world_representation_v1.md`](base_visual_world_representation_v1.md) §4 — `SimStepStamp` **struct** on `FireVisualFrame`, minimal `FixedUpdate` slice; **E2** can proceed in parallel **only** if dirty model does not assume `FixedUpdate` layout prematurely (document coupling).

**EXIT CRITERIA** — written extract + interpolation contract; chosen sim systems on `FixedUpdate`; E2 tests pass before skipping full-grid fill.

---

### `phase-f-gpu-particles`

**GOAL** — GPU-backed particle **presentation** at scale without a second upload architecture. **Strict DONE** language: [`base_visual_world_representation_v1.md`](base_visual_world_representation_v1.md) §4 (Phase F) — real instancing + LOD cost reduction + `WorldFireFx` class; no toy buffers.

**Repo status:** `~` — `[F-1]`–`[F-4]` **buffer / policy slice** (`gpu_particles.rs`, post-LOD `WorldFireParticleFrame`, registry upload). **`phase_f_lod_proof.rs`** records runtime upload ordering per band (Full > Strategic > OverlayOnly). **`[F-3]` draw** and WGSL instancing **not** landed. **Do not** treat as Stage 5 or strict Phase F DONE until § *Stage staging* gates 1–5 pass.

**SUBTASKS** — `[F-1]` `GpuParticleInstance` row ✓ · `[F-2]` `ParticleClass` / policy vs `WorldFireFx` / `AtmosphereFx` ✓ · `[F-3]` instanced quads only ○ · `[F-4]` reuse `FireVisualFrame` + storage + prepare patterns ✓ · `[F-5]` Hanabi only if art pipeline requires ○ · **retire / fold:** `FxParticleBurstRequest` must not become a second upload truth (`meta-no-temp-side-paths` audit).

**BLOCKED BY** — **authoritative `WorldRepresentationResolver`** (§ *Representation domain resolution*); **`theme-3`** strict GPU byte proof; **`phase-e`** snapshot contract if particles sample sim-stepped data; **`visual-test-matrix-upgrade`** VT-4 + **spatial distribution** VT asserts.

**EXIT CRITERIA** — instanced path proven under budget; no duplicate buffer story; Hanabi deferred unless F-5 triggered; **no** primary expansion until resolver + VT-4 strict gates.

---

### `theme-1-camera-intent-productization`

**GOAL** — Camera **subsystem**: intent → ownership → pipeline sets → derived `CameraVisualState` only downstream reads.

**SUBTASKS** — `[T1-1]` `CameraIntent` · `[T1-2]` owner / `ActiveCameraOwner` product model · `[T1-3]` `CameraPipelineSet` (`GatherIntent` → `ResolveVisuals` → `Apply`) · `[T1-4]` `CameraVisualState` as single policy read surface.

**BLOCKED BY** — none critical; coordinate **`meta-schedule-contracts`** for cadence `run_if` alignment.

**KNOWN FAILURE MODES** — scattered `if zoom` / `if tactical` after refactor; HUD and sim disagreeing on owner.

**EXIT CRITERIA** — downstream systems take weights from `CameraVisualState` only (audit grep).

---

### `theme-2-worldfx-atmospherefx-input-buffers`

**GOAL** — Three-way split: **swap utility**, **FX class ownership**, **input snapshot** — no preview-only one-off.

**SUBTASKS** — `[T2-A]` generic `SwapImageBuffers` + debug · `[T2-B]` `WorldFireFx` / `AtmosphereFx` policy + attachment · `[T2-C]` `InputFrame` as sole hot-path read surface.

**BLOCKED BY** — optional: **`phase-d`** for preview swap wiring destination.

**KNOWN FAILURE MODES** — camera-space vs world-space garnish; zoom-scaling wrong class; duplicate cursor readers.

---

### `theme-3-lod-overlay-channel-matrix` *(world representation policy)*

**GOAL** — **Representation domain resolution:** one policy surface decides scaled representation **before** domain snapshots and **before** GPU upload; overlay **matrix** not N independent pipelines. Canonical strict criteria: [`base_visual_world_representation_v1.md`](base_visual_world_representation_v1.md) §2–4 (Theme 3). **Not** “fire LOD” as a subsystem-owned concern.

**SUBTASKS** — `[T3-A]` `FireVisualExtractPlan` + `resolve_fire_visual_extract_plan` before extract ✓ · `[T3-B]` tiered extract: **`OverlayOnly` skips instances** ✓; **`Clustered` caps instances** to [`CLUSTERED_FIRE_INSTANCE_CAP`](../../src/render/extraction/fire_visual_extract.rs) (heat-sorted) while **`chunk_heat` stays complete** ✓ · `[T3-C]` `OverlayFieldFrame.fire_heat_overlay_revision` mirrors [`SharedOverlayFieldBuffers::revision`](../../src/render/overlay_field_buffers.rs) ✓ (`HashMap` image handles for GPU-backed channels still open) · `[T3-D]` per-chunk `WorldLodMap` sampling in render + compute graphs ✓ · `[T3-E]` expanded `LodInputs` + zone authoring into `LodZoneRegistry` ✓ · **strict DONE (open):** **`WorldRepresentationResolver` authoritative** — `RepresentationInputs` → `RepresentationResult`; render / compute / overlay graphs **only** read policy; **measurable** GPU byte / row counts follow bands; grep **no** parallel LOD in fire / preview / upload / camera / particles · **rename track (optional):** `FireVisualLod` → `RepresentationBand`, `FireVisualExtractPlan` → `WorldRepresentationExtractPlan` · **zone evolution (target):** `LodZoneClass` + `LodZone` influence domains (not camera-only).

**BLOCKED BY** — **`theme-1`** stable `CameraVisualState` for LOD inputs; **`visual-test-matrix-upgrade`** VT-2/VT-4 before tuning tiers.

**KNOWN FAILURE MODES** — “particles only in a square” / collapsed spawn bounds; **local-space spawn collapse**; **wrong chunk origin**; **lod extraction truncation**; **buffer instance count mismatch**; **cluster aggregation bounds wrong**; **compute dispatch dimensions wrong**; minimap vs preview heat mismatch; **per-subsystem LOD** (fire vs preview vs GPU upload vs camera vs particles).

**EXIT CRITERIA** — one LOD policy path; overlay channels registered in one frame object; **plus** strict Theme 3 clause in [`base_visual_world_representation_v1.md`](base_visual_world_representation_v1.md) (GPU bytes + single extract); **resolver authoritative** per § *Representation domain resolution*.

---

### `meta-schedule-contracts`

**GOAL** — Central cadence policy + scheduler wrappers **before** large `FixedUpdate` migration.

**SUBTASKS** — `[M-S-1]` `VisualCadence` + sync from `VisualBudgetSettings` ✓ · `[M-S-2]` `on_visual_cadence_*` helpers ✓; **wired:** GPU weather-fire uniform sync throttles on `atmosphere_hz` when `VisualCadence` exists; sim minimap CPU raster throttles on `minimap_hz` when `VisualCadence` exists · `[M-S-3]` cadence tests (partial: defaults + preview contract prefers cadence) · **E1 interim:** [`SimStepStamp`](../../src/systems/sim_control.rs) **struct** on frames ✓; **`FixedUpdate` slice + interpolation** still open per **`phase-e-cadence-scale`** card below.

**BLOCKED BY** — none to start; **informs** `phase-e` and `phase-d` budgets.

**KNOWN FAILURE MODES** — double tick; starving camera; hardcoded Hz scattered after policy exists.

---

### `meta-no-temp-side-paths`

**GOAL** — Kill duplicate truth and “temporary” bypass extractors before they ossify.

**SUBTASKS** — maintain audit table (SYSTEM / SOURCE OF TRUTH / DUPLICATE? / OWNER / TARGET); periodic grep for second minimap path, second fire extract, direct GPU writes from sim.

**Living audit (2026-05-10, grep-backed)** — *“duplicate?” means a second **visual** producer for the same agreed buffer, not sim reads of `ChunkSurfaceFire`.*

| Path / system | Source of truth | Duplicate? | Owner | Target / note |
|----------------|-----------------|------------|-------|-----------------|
| `FireVisualFrame` + `extract_fire_visual_frame` | ECS sim + one extract query | **no** (canonical) | `render/extraction/fire_visual_extract` | Only place that should pack `FireVisualGpuInstance` / `chunk_heat` for render snapshot |
| `SharedOverlayFieldBuffers::chunk_fire_heat` | `FireVisualFrame::chunk_heat` via `sync_shared_overlay_from_frame` | **no** | same plugin chain | Minimap + world preview must sample this map only for fire tint |
| `gpu_field_bridge` / `prepare_fire_visual_gpu_storage` | `FireVisualFrame` (extract resource) | **no** | `systems/atmosphere`, `render/gpu_weather_fire_field` | GPU must not re-scan ECS for fire rows |
| Map editor minimap (`map_editor_raster_minimap`) | Terrain / roads / editor pick grid | **n/a** (different product: terrain pick, not sim fire overlay) | `gui/editor/map_editor` | Keep explicitly separate from P1-G sim fire tint; document in UI |
| Sim minimap + `tile_world_fallback` fire tint | `SharedOverlayFieldBuffers` after fire extract | **no** when raster orders after `FireVisualFrameSet::BuildProfiles` | `render/tile_world_fallback` | Watch for regressions: raster before extract |
| World preview fire overlay | Same shared buffers + revision | **no** | `gui/editor/world_preview` | `world_preview/*` must not add a second `ChunkSurfaceFire` scan |
| Atmosphere `emitter_sync` | Reads `ChunkSurfaceFire` for **sim** emitters | **not** a duplicate visual path | `systems/atmosphere/emitter_sync` | Sim coupling; still must not replace `FireVisualFrame` for GPU rows |
| `FireEmitter` / `fx_burst_request` | Legacy / bridge hints | **watch** | `render/fx_burst_request` | Ensure burst path does not become a second upload truth |

**BLOCKED BY** — none.

**KNOWN FAILURE MODES** — second producer for same overlay; tests pass but minimap disagrees with preview.

---

### `arch-generic-snapshot-layer`

**GOAL** — Long-term stabilizer: registered snapshot producers + one scheduling graph.

**SUBTASKS** — `[A-1]` `ExtractFrameSnapshot` trait ✓ (minimal, no registry) · `[A-2]` `FrameSnapshotRegistry` · `[A-3]` snapshot scheduling graph in docs + `SystemSet` edges.

**BLOCKED BY** — **`phase-e`** E1-3 for “snapshots only” semantics alignment.

---

### `visual-test-matrix-upgrade` *(top-level; not buried inside D/F)*

**GOAL** — Dedicated **scenes + automated / semi-automated checks** so regressions like “particles only in a square” or overlay misalignment are **caught by VT**, not accidental playtests.

**SUBTASKS** — `[VT-1]` full-world distribution + overlay-only LOD + clustered instance cap ✓ · `[VT-2]` zoom invariant: pure scale sweep ✓ + full policy [`apply_camera_visual_from_map_snapshot`](../../src/gui/view_representation.rs) sweep ✓ · scripted camera / HUD path still optional · `[VT-3]` camera mode sweep ○ · `[VT-4]` triple agreement + **detectable mismatch** (minimap + world preview + GPU fire field) — strict criteria: [`base_visual_world_representation_v1.md`](base_visual_world_representation_v1.md) §4 (VT-4) ~ (hash/stamp harness; single runnable scenario open) · **`[VT-5]` spatial distribution invariants** on extract / particle paths — e.g. `occupied_chunk_count > MIN_EXPECTED`, `max_distance_from_origin > threshold` (guards “particles only in a square” and chunk-origin collapse).

**KNOWN FAILURE MODES** — **local-space spawn collapse**; **chunk origin mismatch**; **stale overlay buffers**; **LOD extraction truncation**; **swap desync** (preview); **interpolation jitter** (post–E1); **buffer instance count mismatch**; **cluster bounds wrong**; **compute dispatch dimensions wrong**.

**BLOCKED BY** — none to **author** maps early; **VT-4 full strictness** may wait until `FireVisualFrame` + GPU path stable (already largely true — tighten asserts iteratively).

**EXIT CRITERIA** — VT-1…VT-5 runnable in test harness or dev command line; documented how to run; at least one assertion per VT catching the failure modes above; **VT-4 strict** required before declaring resolver / Stage 2 done.

---

## Rendering & extraction invariants

These are **regression guards** for the View Representation Layer (camera intent → snapshots → GPU/UI). Flip code or docs deliberately when changing one.

1. **ECS simulation** is the sole gameplay authority.
2. **`FireVisualFrame`** is the sole CPU render snapshot for fire visuals (instances + chunk heat path agreed in P1-E).
3. **GPU resources** never query ECS directly; prepare/upload consumes extracted buffers only.
4. **Overlay / minimap / UI fire heat** derives only from agreed frame data (today: `FireVisualFrame::chunk_heat` → `SharedOverlayFieldBuffers`).
5. **Camera / input** does not block on: minimap raster, world preview rebuild, atmosphere recompute, or GPU uploads (ordering + multirate budgets).
6. **Atmospheric / garnish FX** are non-authoritative presentation unless explicitly gated as sim experiments.
7. **Extraction** is deterministic and side-effect free from the sim’s point of view (no hidden writes back to gameplay components during extract).
8. **Prepare** systems only upload already-extracted data.

---

## View Representation Layer (unifies the three roadmap themes)

**Principle:** *Camera intent drives visual representation selection.* Themes 1–3 in the tables below are **one pipeline** in the long arc:

`CameraIntent` / ownership → **`CameraVisualState`** (+ budgets) → LOD selection, overlay channel weights, atmosphere vs world FX class, preview representation, extraction scope.

**Code sketch:** `src/gui/view_representation.rs` (resources + markers + `OverlayFieldFrame` / `FireVisualLod`); `MapCameraSystemSet` in `src/gui/map_camera.rs` slots work between input and smoothing.

---

## Three major themes (map to the parent doc)

| Part | Parent doc focus | Engineering center |
|------|------------------|----------------------|
| **1st** | Strategic world layer vs atmospheric “outside world” layer; camera UX, modes, recenter/edge | **Camera intent + ownership** → drives macro vs tactical read (`CameraVisualState`, future HUD `CAM:` line) |
| **2nd** | Multi-rate pipelines, input vs UI vs extraction, dirty flags, minimap/preview, world vs screen particles | **Latency + invalidation** + **FX class** (`WorldFireFx` vs `AtmosphereFx`), `InputFrame`, `SwapImageBuffers`, preview double-buffer |
| **3rd** | Single fire visual snapshot, shared overlays, P2 refactors, GPU path | **One extract → many consumers** + **LOD / overlay channel matrix** (`FireVisualLod`, `OverlayChannel` → `OverlayFieldFrame`) |

---

## Master priority order (from parent tail + “HIGHEST PRIORITY FIX ORDER”)

Status: `○` not started · `~` in progress · `✓` done enough for current milestone · `!` blocked / needs design

| ID | Item | Status | Note (rough) |
|----|------|--------|--------------|
| P0-A | **Dirty raster** (`tile_world_fallback` + world preview invalidation), not `(w,h,tile_count)` only | ✓ | `TileWorldFallbackRasterDirty` + ECS `Added`/`Changed` + `TerrainRegistriesHandles::is_changed`; revision skip in raster |
| P0-B | **Stable egui texture handles** (preview/minimap); avoid per-frame `add_image` churn | ✓ | `Local<Option<(Handle<Image>, TextureId)>>` in minimap + world preview window |
| P0-C | **Preview `run_if` / throttle** — no full CPU raster at 60 Hz when nothing changed | ✓ | Partial ~12 Hz; **overlay-only** full passes throttled same rate (`render_raster.rs`); keys = epoch / dirty / layers / tex / overlay rev (not egui zoom) |
| P0-D | **HUD dirty flags** — cached strings; no continuous `format!` for static labels | ✓ | Ops strip fingerprints + text write only on change; build line fp; narrative `is_changed` + line cache |
| P1-E | **Single CPU fire snapshot + GPU upload** (proxy row + frame + overlay derive) | ✓ | **Two CPU concepts:** [`FireVisualFrame`](../../src/render/extraction/fire_visual_extract.rs) (`FireVisualProxy` / [`FireVisualGpuInstance`](../../src/render/sim_visual_extract.rs) rows + [`ChunkFireHeat`](../../src/render/sim_visual_extract.rs)); [`SharedOverlayFieldBuffers`](../../src/render/overlay_field_buffers.rs) **only** from `FireVisualFrame::chunk_heat`. GPU: [`FireVisualGpuInstanceStorage`](../../src/render/gpu_weather_fire_field.rs). ECS `ChunkSurfaceFire` **only** in `extract_fire_visual_frame`. |
| P1-F | **Camera smoothing + `run_if`** (`CameraTarget` vs current `Transform`) | ✓ | `MapCameraDesired` + chained lerp; edge toggle / recenter / Z reset / B frame-world; ScrollLock edge; `focus_main_camera` syncs desired |
| P1-G | **Shared minimap + preview overlay** (derived view model) | ✓ | `SharedOverlayFieldBuffers::chunk_fire_heat` **from `FireVisualFrame` only** (no second ECS fire scan); world preview + sim tile fallback / egui minimap via `apply_shared_fire_heat_to_rgba`; dirty on overlay `revision`; raster after `FireVisualFrameSet::BuildProfiles`. |
| P2-H | **Incremental atmosphere** (dirty rects + periodic full refresh) | ✓ | CPU dirty-region scheduling + mirrored GPU subresource uploads + partial compute dispatch; full-field fallback only on reconcile cadence |
| stage5-08 | **App readiness + MAP REP + CI** | ✓ | `stage5_readiness.rs` + `Stage5ReadinessProfile` (`HEADLESS` default; `FULL_APP` with preview); MAP REP HUD; F3 diagnostics; `vt_ci_matrix` fixture + CI `cargo test --lib` |
| P2-I | **FixedUpdate sim** (fire/ecology/logistics) vs Update input/camera/UI | ○ | Large ordering refactor |
| P2-J | **GPU particles** (instanced quads → Hanabi/compute) | ~ | Post-LOD buffer upload ✓; **draw / strict F blocked** until authoritative resolver + VT-4 strict (§ *Stage staging*) |
| **VT** | **`visual-test-matrix-upgrade`** (VT-1…VT-5 scenes + asserts) | ~ | VT-1 ✓; VT-2 ✓; VT-3–4 open; **VT-5 spatial distribution** open |

---

## Consolidated backlog — convergence order (post–seven slices)

Rough dependency order; flip rows in the master table above when done. **Primary expansion** follows § *Stage staging* (resolver authority before particles / preview pixels / atmosphere incremental).

**Representation policy (Stage 2 — unlock)**

1. **`WorldRepresentationResolver` authoritative** — `RepresentationInputs` / `RepresentationResult`; single evaluation per frame; render + compute + overlay graphs read **only** policy output (`world_representation.rs`, `render_projection_graph.rs`, `compute_dispatch_graph.rs`).
2. **Grep gate** — no parallel LOD in fire extract, preview contract, GPU prepare, camera, particles (§ *meta-no-temp-side-paths* audit row for `fx_burst_request`).
3. **Zone evolution** — document / implement `LodZoneClass` + influence domains beyond camera (missions, combat, projectiles, sensors) feeding registry.

**E1 + VT (Stage 3)**

4. **E1** — `FixedUpdate` minimal sim slice; render reads committed `SimStepStamp` snapshots only; interpolation contract.
5. **VT-3** — camera mode sweep.
6. **VT-4 strict** — one runnable triple-surface scenario (minimap + preview + GPU fire field).
7. **VT-5** — spatial distribution invariants on extract / particle paths.

**Projection + registry (Stage 4)**

8. **Measurable GPU bytes / rows vs band** — prove shrink-on-band on fire + particle + heat buffers.
9. **`arch-generic-snapshot-layer`** — `FrameSnapshotRegistry` + scheduling graph doc.

**Convergence features (after gates — not primary until Stages 2–4 strict)**

10. **Phase D** — ✓ GPU `RenderTarget::Image` + `SwapImageBuffers` + egui display-only; CPU raster fallback only when GPU unavailable or toolbar forces CPU.
11. **Phase F draw** — instanced quads from registry; fold or retire burst side path.
12. **P2-H** — incremental atmosphere (E2 track).
13. **P2-I** — broader `FixedUpdate` + logistics/ecology ordering.

**P1 / themes (parallel when not adding duplicate producers)**

14. **Theme 1st** — `CameraIntent` / `CameraOwner`; camera feeds inputs, does not own bands.
15. **Theme 2nd** — `SwapImageBuffers`, `WorldFireFx` / `AtmosphereFx` attachment policy, `InputFrame`.
16. **Theme 3rd** — rename track + overlay channel GPU handles in `OverlayFieldFrame`.

**Injection queue (misc)**

17. PresentMode / mailbox pass; egui scissor; visible-chunk preview cache; transport event cadence notes.

---

## 1st theme — layers & cameras (planned todos not fully in table above)

| Todo | Status | Note |
|------|--------|------|
| **Resource:** `CameraControlSettings` or `CameraControlState` — edge scroll toggle, default zoom, mode | ~ | Edge + zoom live on `MapCameraSettings` / `MapCameraDesired`; consolidated `CameraControlState` not added |
| **Actions:** Recenter (Home / Space×2 / double middle), reset zoom (Z / Shift+Home), frame world (`fit_world_bounds`) | ~ | Home / Z / B + double middle ✓; Space×2 not wired |
| **Enum:** `CameraMode` — Strategic / Tactical / Cinematic / FollowEntity / FreePan | ~ | `MapCameraMode` + `CameraIntent` scaffold in `view_representation.rs`; Follow / FreePan still ○ |
| **HUD strip:** tiny camera state (edge on/off, zoom, mode, follow target) | ~ | Bevy ops strip MAP line (mode, edge, zoom, yaw); follow target N/A |
| **Clarify “two cameras” mentally:** (A) Bevy `MainWorldCamera` gameplay view (B) egui preview = texture, not a second game camera yet | ○ | Long-term: render target inside egui (parent §9) |
| **Outside-frame fun:** macro layer stays **camera-reactive**, **not sim-authoritative**; document in code near atmosphere drivers | ○ | Zoom out → plumes/haze; zoom in → tactical smoke |

---

## 2nd theme — pipelines, particles, UI perf

| Todo | Status | Note |
|------|--------|------|
| **Stages:** input collect → camera → UI intent → visual invalidation flags → low-rate extract → render | ○ | Align with `SystemSet` names in parent; incremental adoption |
| **Minimap:** cached texture; dirty on camera chunk / fire / overlay change | ~ | Sim minimap: stable egui texture + `TileWorldFallbackRasterDirty` + **fire tint** from `SharedOverlayFieldBuffers`; camera-chunk dirty not granular yet |
| **Preview:** double-buffer RGBA or swap handles to avoid upload tear | ○ | Parent “bars” hypothesis: timing + empty tiles |
| **Split particles:** `WorldFireFx` vs `AtmosphereFx` (world-scaled vs screen/macro) | ~ | Marker components in `view_representation.rs`; attach + policy TBD |
| **Event coalescing:** latest mouse + scroll per frame (`InputFrame`) | ~ | Pointer + scroll + drag flag + frame counter in `PreUpdate`; camera reads snapshot |
| **Camera never waits on:** worldgen, minimap raster, smoke extract, heavy egui | ○ | Ordering audit |

---

## 3rd theme — extraction, overlays, GPU path *(consumers of representation policy)*

| Todo | Status | Note |
|------|--------|------|
| **Formalize** `FireVisualProxy` / single consumer buffer from `ChunkSurfaceFire` once per tick | ~ | Extract + `SharedOverlayFieldBuffers` + preview/minimap/light path; aggregate `FireAtmosphereAggregate` still separate row |
| **Representation bands** by policy (not per-system zoom): strategic blobs vs mid vs close flame sprites | ~ | `FireVisualLod` / `RepresentationBand` rename track; **`WorldLodMap`** + projection caps ✓; **resolver authority** ○ |
| **Overlay channel list** height/temp/moisture/ecology/smoke/fire/wind/mobility/pressure — shared sampler | ~ | `OverlayChannel` + `OverlayFieldFrame` resource stub; GPU-backed channel handles ○ |
| **GPU weather-fire field** — keep as visual-only; no sim readback without gate | ○ | Upload via **`GPUBufferRegistry`** + projection graph — extend pattern, do not fork allocators per domain |

---

## Bottom injection queue (misc from parent not mapped above)

Add here when you pull new bullets from `base_visual_dev01.md`:

- [ ] PresentMode / mailbox review vs “feels like input lag”
- [ ] Viewport scissoring for large egui regions
- [ ] `InputFrameState`-style fast path (optional resource)
- [ ] Transport/logistics: event-driven cadence called out in parent timing table

---

*Last template edit: 2026-05-10 — § *How to process a top-level todo* (execution loop); prior: optimized order, `VisualCadence`, `ExtractFrameSnapshot`, VT-1, meta-no-temp audit.*
