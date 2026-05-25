# Operational readiness vs infrastructure hardening — plus perf playbook

This guide fixes a common confusion: **FULL_APP green / spine convergence** is **not** the same milestone as **ViewManager isolation / multiview hardening**. They overlap in tooling but answer different questions.

---

## 1. Two milestones — explicitly different

### A. Operational readiness (Stage 5 “exit” lane — convergence)

**Means:**

| Pillar | What “good” looks like |
|--------|-------------------------|
| **FULL_APP green** | `Stage5ReadinessProfile::FULL_APP` passes live gates (`evaluate_app_stage5_readiness`, violations empty when applicable). |
| **Spine valid** | Single authoritative paths: `RepresentationResult`, `RenderProjectionGraph`, fence, shared overlays, agreed GPU uploads — no duplicate extractors or shadow LOD. |
| **Runtime stable** | App runs without coherence regressions that violate contracts (camera ↔ minimap ↔ preview policies stay explainable). |
| **Contracts pass** | VT matrices/parity probes, Phase D/E/F hooks where gated — proofs attach to harness (`stage5_full_app_live.json`, lib tests). |

**In plain language this milestone is:**

- **Playable** — operator can run the session without systematic representation lies.
- **Coherent** — one spine truth per domain order (fire + logistics + ecology projection ordering).
- **Testable** — readiness + spine predicates + fixtures give repeatable signals.
- **Converged** — new work attaches to existing contracts; no parallel representation stacks.

**References:** `AGENTS.md`, `prompts/guides/stage5_convergence_directive_v1.md`, `src/render/stage5_readiness.rs`, `src/dev/stage5_live_todos.rs`.

---

### B. Infrastructure hardening (post‑convergence / quality tier)

**Means:**

| Track | Examples (non‑exhaustive) |
|-------|---------------------------|
| **VM-06 … VM-11** | View authority completion — sole writers per view, bridge drift bounded (`stage5_live_todos` witnesses). |
| **Per-view isolation** | Projection, filters, overlays keyed by `ViewId`; no accidental globals poisoning unrelated surfaces. |
| **Editor parity** | Preview/world‑gen/editing flows match simulation semantics where promised — no silent divergence. |
| **Multiview correctness** | Minimap vs main vs GPU preview independently correct rects, bindings, and revision coupling. |
| **Deterministic replay** | Scenario/session reproducibility for CI and tooling — separate hardening program. |

**In plain language this milestone is:**

- **Scalability** — more views, more tooling surfaces without entangling state.
- **Tooling** — editors, diagnostics, and replay hooks remain trustworthy under stress.
- **Futureproofing** — migrations don’t require rewriting representation spine again.
- **Robustness** — boundary violations fail loudly or degrade gracefully — no silent cross‑view bleed.

**References:** `prompts/guides/base_finsh_5.md` (VM backlog narrative — treat as **hardening**, not interchangeable with “readiness green”).

---

### Relationship — why confusion hurts planning

- You can have **operational readiness green** while **infrastructure hardening** rows remain open (per‑view audit debt, aggressive parity targets).
- Conversely, hardening work **must not** introduce parallel representation stacks — it extends authority discipline already justified by readiness contracts.

**Rule:** Do **not** slide VM backlog closure into the Stage‑5 operational gate unless **your team explicitly expands** that gate. Default AGENTS gate stays **FULL_APP live coherence**, not “all VM‑NN checklist rows Done.”

---

## 2. Performance analysis — current signature is informative

Observed pattern (representative):

```text
wall ≈ 220 ms
readiness ≈ 0.03 ms … few ms
world_repr / graph / merge / atmosphere spine buckets tiny
gpu_gap ≈ 0
```

### Interpretation

**The spine itself is no longer the primary bottleneck.** That’s meaningful convergence progress — readiness + projection merge slices aren’t eating the frame.

The dominant bucket reads like **engine shell overhead**: scheduling, uncaptured Update work, host/OS/logging/UI paths — **not** GPU saturation or fundamental renderer architecture.

---

### Bottleneck class (ordered by probability)

#### (1) Logging — highest suspicion on Windows

`info!(…)` inside hot loops can produce **50–200 ms** stalls via:

- `stdout` lock contention  
- console flush  
- tracing subscriber formatting (ANSI, etc.)

**Immediate mitigation:**

- Default runtime subscriber noise should bias **`WARN`**, not blanket **`INFO`**.
- Enable selectively: **`perf=info`**, **`stall=info`** (and subsystem‑scoped filters like `stage5_readiness::live=info` only when debugging).

**Guarding expensive formatting:**

`tracing` evaluates arguments eagerly unless guarded. Prefer:

```rust
use tracing::Level;

if tracing::enabled!(Level::INFO) {
    tracing::info!(heavy = %expensive_format(), "...");
}
```

(or subsystem‑scoped `enabled!(target: "crate::foo", Level::INFO)`).

Avoid unconditional interpolation when the handler might drop the event.

---

#### (2) Update schedule attribution gap — biggest observability problem

Today **`cpu_pre_egui` / `Update`** is largely a **black box**: instrumentation proves spine isn’t the cost center but doesn’t yet explain ~220 ms.

**Recommendation:** **hierarchical scopes first**, not per‑system spam:

```
Update
 ├── WorldGen
 ├── Streaming
 ├── ChunkApply
 ├── OverlayCompose
 ├── FireExtract
 ├── PresentationValidation
 ├── PreviewSync
 ├── VT residency
 ├── HUD commit
 └── Egui bridge
```

Instrument at **plugin / subsystem / phase / pipeline boundary**.

**Suggested helper pattern** (threshold‑logged scopes):

```rust
pub struct PerfScope {
    label: &'static str,
    start: std::time::Instant,
}

impl PerfScope {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            start: std::time::Instant::now(),
        }
    }
}

impl Drop for PerfScope {
    fn drop(&mut self) {
        let dt_ms = self.start.elapsed().as_secs_f64() * 1000.0;
        if dt_ms > 0.25 {
            tracing::info!(target: "perf_scope", "{} {:.2}ms", self.label, dt_ms);
        }
    }
}
```

Usage: `{ let _p = PerfScope::new("chunk_apply"); … }`

**Ideal log shape:**

```text
update=220ms
 ├── chunk_apply=130ms
 ├── preview_sync=40ms
 ├── validation=20ms
 ├── hud_commit=10ms
 ├── fire_extract=8ms
 └── misc=12ms
```

After attribution, optimization becomes trivial.

**Highest‑ROI first targets** given architecture:

1. World preview  
2. Streaming  
3. Chunk apply  
4. Presentation validation  
5. Overlay compose  
6. HUD shell commit  

---

#### (3) Presentation validation — easy accidental full scans

Validation/diagnostics/consistency checks that scan **full graphs / entities / overlays every frame** are common regressions.

**Prefer:**

- **Event‑driven**  
- **Stamp‑driven** (`CommittedVisualSnapshotFence`, revision counters)  
- **Dirty‑region** invalidation  

**Avoid:**

- **Polling‑style full scans** on idle frames  

---

#### (4) EGUI invalidation churn

Watch **`layout_invalidations_frame`**, **`viewport_mutations_frame`**, texture rebuild counts (`FrameBudgetDiagnostics`).

Problems spike when:

- unstable IDs cause wholesale rebuilds  
- minimap/preview textures churn  
- viewports recreated unnecessarily  

**Direction:**

Cache minimap/preview presentation behind authoritative stamps (**fence / overlay revision / raster revision**). If stamps unchanged → skip texture rebuild, viewport churn, and broad UI invalidation where safe.

---

#### (5) Representation recomputation — next architectural leverage

You already have **fence + stamps**. Fully embrace invalidation chains:

```
world mutates
 → repr invalidates
 → projection invalidates
 → view invalidates
 → GPU upload invalidates
```

**Target:** no representation‑heavy work unless **repr stamp**, **policy**, or **camera regime** changed — otherwise reuse retained snapshots.

---

#### (6) Fire — assessment

Using **`FireLightEmission::radius`** as billboard scale was **domain leakage**: light influence ≠ particle footprint.

Phase‑F remap **decouples** these domains; **heat field + emitter field + instanced atlas animation** are natural upgrades later — current trajectory is sound.

---

## 3. Final assessment narrative

You are largely past fighting:

- conceptual rendering structure  
- representation spine convergence  
- “did we wire FULL_APP correctly?”  

You are now most exposed to:

- **hidden work** inside uncaptured Update segments  
- **scheduling discipline** (ordering / contention / mega‑chains)  
- **invalidation correctness** (stamps vs polling)  
- **subsystem attribution** (who owns the 220 ms?)  

**Strongest near‑term ROI:** **frame phase attribution** + logging hygiene — before speculative micro‑opts.

---

## Revision

| Version | Date | Notes |
|---------|------|------|
| v1 | 2026‑05‑15 | Split milestones + perf playbook consolidated from operator brief. |
