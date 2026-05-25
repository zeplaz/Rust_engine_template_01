# Stage 5 convergence directive (revised) `v1`

**Status:** Primary governance for visual / world representation work.  
**Companion:** [`base_visual_world_representation_v1.md`](base_visual_world_representation_v1.md), [`base_visual_dev01_plan_status.md`](base_visual_dev01_plan_status.md).

Stage 5 is the **primary convergence lane** — not the only lane. The goal is **convergent growth**: stop unowned duplication and architectural drift without freezing healthy forward development.

---

## 1. Primary vs secondary objectives

**Primary:** Converge all visual / world representation authority into one measurable pipeline:

`camera / player intent` → `representation resolver` → `representation frames` → `overlays / fields` → `GPU / upload / render / UI`

**Secondary:** Allow new systems only through that pipeline — future-proofing and emerging features are encouraged when they **strengthen** the spine, not bypass it.

**The real enemy:** unowned duplication (parallel extraction, duplicate LOD, second minimap truth paths).  
**Not the enemy:** experimentation, scaffolds with exit plans, or UX / campaign / strategic work that **consumes** authoritative frames.

---

## 2. Architectural rule

**Do not use:** “No new systems until Stage 5 closes.”

**Use:** **New systems must attach to authoritative contracts.**

Any new feature must:

1. Declare its **authoritative owner**.
2. **Consume** existing representation contracts where applicable.
3. **Avoid** parallel extraction or policy paths.
4. **Register validation hooks** if it affects visual authority (readiness, VT, spine audit).

---

## 3. Authority classes

| Class | Meaning | Rule |
|:---|:---|:---|
| **AUTHORITATIVE** | Real owner for a concept | Single path; others consume only |
| **TRANSITIONAL** | Approved temporary surface | Must declare migration target + removal trigger |
| **LEGACY** | Superseded | No expansion; trend toward removal |

**Authoritative anchors (non-exhaustive):** `WorldRepresentationResolver` / `RepresentationResult`, `FireVisualFrame`, `SharedOverlayFieldBuffers`, `RenderProjectionGraph`, `CommittedVisualSnapshotFence`, `GPUBufferRegistry` uploads from projection.

**Good attachment examples:** minimap mode reads `RepresentationResult`; AI overlay reads `SharedOverlayFieldBuffers`; transmission widget reads domain projection; new GPU work respects `RepresentationResult` policy.

**Forbidden:** minimap scans ECS separately; preview extracts fire independently; particles carry their own LOD policy.

---

## 4. Scaffold contract

A transitional scaffold is acceptable only with an explicit contract (see `ScaffoldContract` in `src/gui/representation_governance.rs`):

- **owner** — module / team responsible
- **intended_replacement** — authoritative target
- **exit_condition** — when migration is done
- **removal_trigger** — what forces delete if slip occurs

---

## 5. Execution tiers

### Tier 1 — mandatory convergence (always advancing)

Trend toward green each cycle:

- Resolver authority (no parallel LOD in fire / preview / upload / camera / particles)
- FULL_APP VT / readiness
- GPU preview authority
- Phase F strict instanced draw + LOD proof
- BQ-101 consumers on tile / render paths
- Docs / status truth vs code (`plan_status`, runbooks, open backlog constants)

### Tier 2 — allowed emerging systems

Allowed **if** attached to contracts: minimap evolution, TV / transmission widgets, overlay shell, command UI, streaming visualization, campaign scripting, strategic overlays.

**Construction UX is not a Stage 5 gate.** Track on the parallel **Construction stage** board ([`src/dev/construction_live_todos.rs`](../../src/dev/construction_live_todos.rs), witness [`ConstructionStageWitness`](../../src/gui/build/construction_stage_witness.rs)). Do not add construction rows to `STAGE5_TODOS` or block FULL_APP on build toolbox / roads.

### Tier 3 — future-proofing

Allowed **if** built on primary contracts without duplicate ownership — e.g. paged-atlas API inside `GPUBufferRegistry`.  
**Not allowed:** alternate minimap field uploaders, shadow policy graphs, second fire extract.

---

## 6. Allowed work matrix

| Category | Allowed? | Rule |
|:---|:---:|:---|
| UX systems | Yes | Consume authoritative frames |
| Future-proofing | Yes | Primary spine only |
| Experimental rendering | Yes | Isolated contracts + validation |
| Parallel extraction | No | Forbidden |
| Duplicate LOD logic | No | Forbidden |
| Temporary scaffolds | Yes | `ScaffoldContract` required |
| New overlays | Yes | `SharedOverlayFieldBuffers` / domain projection |
| New GPU systems | Yes | `RepresentationResult` + registry upload path |

---

## 7. Agent cycle (every pass)

1. Advance **one** Tier 1 convergence item.
2. Optionally advance **one** Tier 2 / 3 item that **attaches** correctly.
3. Verify **authority duplication did not increase** (grep / readiness / spine audit).
4. Update ownership docs and status truth.

Serialization / streaming waves (**S → P → C**) and Stage 6 virtualization continue on the spine; unresolved product choices surface as **BQ-###** per [`rulebook_backlog_designer_brief_v1.md`](rulebook_backlog_designer_brief_v1.md) §4 — do not invent schema or thresholds in engine code.

---

## 8. Prompt fragment

> Stage 5 is the **primary convergence lane**. New work must attach to authoritative representation contracts; parallel extraction and duplicate LOD are prohibited. Transitional scaffolds require `ScaffoldContract`. Each cycle: advance convergence, optionally ship attached emerging work, verify no new duplication, update docs. **Exit gate:** `Stage5ReadinessProfile::FULL_APP` green in the **running app** — see §9–§16. See [`stage5_convergence_directive_v1.md`](stage5_convergence_directive_v1.md).

---

## 9. Stage 5 DONE (exit gate)

Stage 5 is **DONE** when the **running app** proves:

- one authoritative representation policy
- one fire extraction spine
- one preview authority path
- measurable LOD affecting GPU cost
- VT-4 / VT-5 agreement under `Stage5ReadinessProfile::FULL_APP`
- Phase F instanced draw active
- no hidden parallel visual pipelines

**Code hooks:** `AppStage5ReadinessReport`, `stage5_readiness_passes`, `Stage5ReadinessProfile::FULL_APP` (`src/render/stage5_readiness.rs`); MAP REP / F3 diagnostics in the live session.

Unit tests and isolated fixtures support closure but **do not** substitute for FULL_APP runtime proof.

**Live proof artifact (2026-05-14):** `cargo run --bin proc_A_dine01 -- --test visual` emitted [`debug_runs/stage5_full_app_live.json`](../../debug_runs/stage5_full_app_live.json) with `readiness.passes=true`, `render_anomalies` all false, GPU preview render target `486×436` matching resolved viewport, explicit minimap `CpuRaster` + `shared_projection`, and Phase F live fields (`phase_f_samples=257`, `phase_f_lod_proof_ok=true`). Harness: `finalize_visual_full_app_live_probe` in `src/render/stage5_full_app_harness.rs` after `FullRenderDiagnosticSet::Capture`.

---

## 10. Not required before Stage 6

May continue in parallel when attached to shared contracts:

- perfect scalability, final art, full behavioral AI
- complete virtualization, multi-atlas infinite-world support
- Stage 6 virtualization, Wave S/P/C hardening, GUI, campaign scripting, video widgets, **construction stage** (parallel), Stage 7 planning, logistics, behavioral AI stubs

---

## 11. Hard rules

**Allowed:** temporary feature implementations, experimental gameplay, partial UI, simplified GPU paths, stub overlays (with `ScaffoldContract` when transitional).

**Not allowed:** second ECS scans for the same visual data; duplicate fire extraction; separate preview-only truth paths; isolated LOD logic per subsystem; independent overlay ownership; GPU upload paths bypassing resolver / frame.

**New-system check:** *Does this introduce a second authority path?* If yes, integrate into the spine; if no, proceed.

---

## 12. Agent workflow (every cycle)

1. **Run readiness** — `cargo test -p proc_A_dine01 --lib`, then exercise `Stage5ReadinessProfile::FULL_APP` in the running app; capture failing readiness checks.
2. **Build TODO queue from failures only** — failing gates, duplicate-path findings, missing FULL_APP proofs, unresolved VT mismatches. **Not** speculative architecture rewrites.
3. **Resolve highest-leverage item** — priority: duplicate authority → FULL_APP failures → VT mismatches → Phase F proof gaps → preview authority → resolver integration → metrics/HUD visibility.
4. **Verify in running app** — preview visible, overlays active, GPU path active, camera movement, LOD transitions; not fixtures alone.
5. **Update readiness + docs** — readiness checks, `base_visual_dev01_plan_status.md` rows, `base_visual_dev01_roadmap_next.md` rows; remove stale “open” bullets when code reality changed.

**Minimal loop:** RUN FULL_APP → collect failures → fix highest authority violation → rerun FULL_APP → update readiness/docs → repeat until FULL_APP green.

---

## 13. Mandatory closures (A–F)

| ID | Closure | DONE when |
|:---|:---|:---|
| **A** | Resolver authority | All visual systems derive from `WorldRepresentationResolver` → `FireVisualFrame` → GPU/upload/preview/overlay; audit removes local LOD branches, preview-specific extraction, GPU-private visibility logic |
| **B** | VT-4 / VT-5 FULL_APP | Minimap, preview, GPU fire field agree within tolerance at runtime; mismatch logs include `SimStepStamp` |
| **C** | Phase D preview authority | GPU render target is primary preview authority; CPU fallback/debug raster only — not silent main renderer while GPU active |
| **D** | Phase F draw | Instanced draw from resolver → `FireVisualFrame` → registry upload → GPU instances → draw; LOD measurably reduces instance count, upload bytes, dispatch cost |
| **E** | Overlay ownership | Overlays consume only `SharedOverlayFieldBuffers`; no secondary ECS scans |
| **F** | FULL_APP metrics | HUD exposes upload bytes, instance counts, LOD band, preview authority, fallback flags, partial/full dispatch, VT mismatch counters |

---

## 14. Final success state (after closure)

Authoritative world representation; stable extract/upload contracts; GPU-backed preview authority; validated LOD scaling; unified overlays; instanced render spine; runtime agreement verification.

---

## 15. Prompt fragment (exit gate)

> Stage 5 closes when `Stage5ReadinessProfile::FULL_APP` is green in the **running app** with preview, overlays, GPU path, camera movement, and LOD transitions exercised. Fix order: duplicate authority, FULL_APP failures, VT, Phase F, preview authority, resolver, HUD. Do not block Tier 2 work that consumes shared contracts. See [`stage5_convergence_directive_v1.md`](stage5_convergence_directive_v1.md) §9–§14.
