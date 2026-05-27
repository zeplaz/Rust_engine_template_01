# WSS coder hybrid orders `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@orchestrator` |
| **Prereq** | **WSS-DESIGN-GATE-001** **PASS (qualified)** on [`wssr_design_signoff_v1.md`](wssr_design_signoff_v1.md) (2026-05-26) |
| **Plans** | [`wssr_index_v1.md`](wssr_index_v1.md) · WSS-PLAN-002/003/004 |
| **Skills** | bevy-simulation-grade · cleanup-completion-intelligence |

---

## Mandatory coder protocol (every WSS slice)

Before writing code, each coder submits a **Hybrid Assessment** (≤15 lines) in PR description or `src/dev/wss_hybrid_assessments/` note:

```yaml
slice_id: WSS-CHUNK-SLAB-001
incumbent_authority: path::Symbol — what exists today
wss_plan_asks: one sentence
options:
  A: adopt WSS as written
  B: hybrid (default when incumbent is superior)
  C: defer — document why
chosen: B
hybrid_shape: "ChunkSlab<T> for persist; keep ChunkWeather component until clipmap couples"
preserves: [fire_view_extract single writer, water witness greens, ...]
regression: cargo test -p proc_A_dine01 --lib ...
```

**Hard rules:**

1. **Question the plan** — if replacement is not clearly superior, choose **hybrid**.
2. **No silent deletion** — classify with cleanup-intelligence (A obsolete / B transitional / C dormant / D incomplete).
3. **≤3 files per PR** unless planner expands exec plan.
4. **Witness JSON** — extend `debug_runs/wss_substrate_live.json`; do not hand-edit greens.
5. **L3 must not write L1** — GPU fields / Hanabi never mutate gameplay without readback contract.
6. **Do not regress** — F7 per-view extract, Stage 5 spine, closed water tactical witness.

---

## Entry order (after design gate)

| Priority | ID | Owner | Plan | Parallel? |
|:---:|:---|:---|:---|:---:|
| 1 | **WSS-CHUNK-SLAB-001** | @coder A or substrate lane | WSS-PLAN-002 | — |
| 2 | **WSS-ATMOS-CLIPMAP-001** | @coder A (atmosphere) | WSS-PLAN-004 | After slab **types** exist |
| 3 | **WSS-HYDRO-RUNTIME-001** | @coder B or terrain lane | WSS-PLAN-003 | After gen→slab hydrate path designed |
| — | **Hanabi spike** | @coder or experiments/ | index § Hanabi | **Non-blocking** — no main plugin merge |

**Parallel with WSS (disjoint files):**

- **FIRE-F2-EXTRACT-001** — extraction graph (Team C) — **no slab writes**
- **M3-UNITS-DEPTH / REPLAY-RING** — gui minimap
- **Parametric construction** — `src/construction/*` only

**Pause on WSS track:** R4-MV-GHOST if it fights substrate overlay ownership.

---

## WSS-CHUNK-SLAB-001 — exec sketch (planner/coder)

**Goal:** Types + `WorldSubstrateRegistry` + witness skeleton — **no domain migration yet**.

| Task | Files (≤3) | Proof |
|:---|:---|:---|
| C1 | `ChunkKey`, `ChunkSlab<T>`, `WorldSubstrateRegistry` scaffold | unit tests: insert/dirty/resident |
| C2 | `wss_substrate_live.json` writer stub in dev or substrate module | all flags false, `green: false` |
| C3 | Hybrid note: map `ChunkWeather` / fire components → future slab slots | assessment yaml |

```powershell
cargo test -p proc_A_dine01 --lib wss_substrate  # when module exists
```

---

## WSS-ATMOS-CLIPMAP-001 — exec sketch

**Goal:** Clipmap **types + sim vs render separation** — do **not** delete `AtmosphereField` 128² until bridge exists.

| Hybrid default | |
|:---|:---|
| New `AtmosphereClipmapSim` resource alongside existing field | |
| `fire_visual_emit_smoke_stub` → bridge plan only in assessment | |
| Keep CPU precip until designer approves GPU migration | |

---

## WSS-HYDRO-RUNTIME-001 — exec sketch

**Goal:** `ChunkSlab<HydrologyState>` + scheduler sketch — **FluidDomain forbidden**.

| Hybrid default | |
|:---|:---|
| `HydrologyResult` / markers from terrain gen → slab hydrate | |
| `gpu_water_*` remains L3 — ocean_mask in hydrology slab | |
| Maintain `water_w1_green` regression | |

---

## Hanabi spike (non-blocking)

| Deliverable | Path |
|:---|:---|
| Compatibility report | `experiments/hanabi_validation/README.md` |
| Verdict | adopt L3 later / defer / reject for 0.18 pin |

**Do not** register Hanabi in main `App` until design signoff + spike PASS.

---

## Steward regression bundle (WSS touch)

```powershell
cargo test -p proc_A_dine01 --lib stage5 fire_streaming gpu_particles
# After first wss writer:
cargo test -p proc_A_dine01 --lib wss_substrate
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | Hybrid orders + design gate prereq |
