# WSS coder hybrid orders `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.1.0` |
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
| 2 | **WSS-ATMOS-CLIPMAP-001** | @coder A (atmosphere) | [`plan_wss_atmos_clipmap_exec_001_v1.md`](plan_wss_atmos_clipmap_exec_001_v1.md) | After slab **types** exist |
| 3 | **WSS-HYDRO-RUNTIME-001** | @coder B or terrain lane | [`plan_wss_hydro_runtime_exec_001_v1.md`](plan_wss_hydro_runtime_exec_001_v1.md) | After `hydrate_wired` |
| — | **Hanabi spike** | @coder or experiments/ | index § Hanabi | **Non-blocking** — no main plugin merge |

**Parallel with WSS (disjoint files):**

- **FIRE-F2-EXTRACT-001** — extraction graph (Team C) — **no slab writes**
- **M3-UNITS-DEPTH / REPLAY-RING** — gui minimap
- **Parametric construction** — `src/construction/*` only

**Pause on WSS track:** R4-MV-GHOST if it fights substrate overlay ownership.

---

## WSS-CHUNK-SLAB-001 — execute plan

**Full plan:** [`plan_wss_chunk_slab_exec_001_v1.md`](plan_wss_chunk_slab_exec_001_v1.md) — CS-001…CS-006, witness `gate: WSS-CHUNK-SLAB-001`.

```powershell
cargo test -p proc_A_dine01 --lib substrate
```

---

## WSS-ATMOS-CLIPMAP-001 — execute plan

**Full plan:** [`plan_wss_atmos_clipmap_exec_001_v1.md`](plan_wss_atmos_clipmap_exec_001_v1.md) — AC-001…AC-008, witness `wss_atmos_clipmap_001`.

| Hybrid default | |
|:---|:---|
| `AtmosphereClipmapStack` L0–L3 alongside legacy `AtmosphereField` L1 alias | |
| `AtmosphereRenderClipmap` ≠ sim stack | |
| Remove `fire_visual_emit_smoke_stub` when extract node green | |

**Blocked until:** `slab_registry_present` + full `WorldChunkState` types.

---

## WSS-HYDRO-RUNTIME-001 — execute plan

**Full plan:** [`plan_wss_hydro_runtime_exec_001_v1.md`](plan_wss_hydro_runtime_exec_001_v1.md) — HY-001…HY-008, witness `wss_hydro_runtime_001`.

| Hybrid default | |
|:---|:---|
| `HydrologyResult` → slab hydrate on spawn | |
| Tier 1 background + Tier 2 deep solve scheduler | |
| `gpu_water_*` L3 consumer only — **FluidDomain forbidden** | |

**Blocked until:** `hydrate_wired` from WSS-CHUNK-SLAB-001.

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
| v1.1.0 | 2026-05-26 | Full exec plans for ATMOS + HYDRO slices |
| v1.0.0 | 2026-05-26 | Hybrid orders + design gate prereq |
