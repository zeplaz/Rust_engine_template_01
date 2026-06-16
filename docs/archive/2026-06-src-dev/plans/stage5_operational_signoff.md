# Stage 5 operational sign-off

**Status:** **CLOSED** — operational readiness gate satisfied.  
**Signed:** 2026-05-23  
**Next open lane:** [`stage5_5_open.md`](stage5_5_open.md)

---

## What “closed” means

Stage 5 **operational readiness** per [`AGENTS.md`](../../AGENTS.md) and [`prompts/guides/operational_readiness_vs_infrastructure_perf_v1.md`](../../prompts/guides/operational_readiness_vs_infrastructure_perf_v1.md):

- **Spine authoritative** — `RepresentationResult`, `RenderProjectionGraph`, `FireVisualFrame`, `SharedOverlayFieldBuffers`, single fire extract path
- **FULL_APP measurable** — `stage5_readiness_passes`, empty violations in live evals
- **Lib proof** — `cargo test -p proc_A_dine01 --lib` (605+ tests)
- **Parallel stages** (construction operational, industrial, logistics) do **not** block this gate

Stage 5 is **not** “all VM-06…11 done”, “all fire streaming done”, or “shell perf &lt; 33 ms”. Those live in [`stage5_triage_backlog.md`](stage5_triage_backlog.md).

---

## Evidence (operator)

| Artifact | Signal |
|----------|--------|
| `debug_runs/stage5_full_app_live.json` | **Refreshed 2026-05-23:** `readiness.passes: true`, `stage5_closure.passes: true`, 13+8 boards Done, `_agent_meta` envelope |
| **`--test visual` (agent)** | `wrote stage5 FULL_APP live proof` FINISH-UX-06 streak=120; exit 0 (~90s) |
| `assets/shaders/fire/fire_particle_draw.wgsl` | VR-07: single `alpha` binding (Naga) |
| `src/render/gpu_fire_particle_raster.rs` | VR-08: globals uniform `VERTEX_FRAGMENT` visibility |
| `src/render/stage5_full_app_harness.rs` | VR-09: proof commit when `instanced_dispatch_ok` (zero fire rows OK) |
| `assets/shaders/debug/tile_debug_instanced.wgsl` | VR-01: `tile_row` (no `inst` scope panic) |
| `src/render/gpu_surface_teardown.rs` | Graceful `--test visual` exit after proof |

**Formal B checklist:** [`stage5_close_checklist.md`](stage5_close_checklist.md) — **§B complete** (B1–B6) 2026-05-23.

---

## Moved to triage (not verified for this gate)

| Topic | Backlog ID |
|-------|-----------|
| VM-06…11 implementation (beyond isolation witness) | TRIAGE-VM-06 … TRIAGE-VM-11 |
| Full fire streaming / sleep / neighbor wake | TRIAGE-FIRE-STREAM, TRIAGE-FIRE-LOD-TIERS |
| MapCameraDesired invert-only authority | TRIAGE-VM-09 |
| Shell perf p95 &lt; 33 ms sustained | T5 in triage + perf doc |
| Stage 6 virtualization exit criteria | Stage 5.5 open doc |
| Wave S / P / C product serialization | Stage 5.5 open doc |
| Logistics `log_rows` in normal play (scenario-dependent) | logistics lane |
| Compile warning hygiene (non-blocking) | TRIAGE-COMPILE-HYGIENE |

---

## Commands (regression)

```powershell
cargo test -p proc_A_dine01 --lib stage5
cargo run -p proc_A_dine01 --release -- --test visual
```

Expect: visual run → `wrote stage5 FULL_APP live proof` → `visual test proof committed — requesting graceful AppExit`.

---

## Handoff to next stage

Do **not** reopen Stage 5 for new features. Attach new work to:

1. [`stage5_5_open.md`](stage5_5_open.md) — infrastructure + product waves  
2. [`stage5_triage_backlog.md`](stage5_triage_backlog.md) — deferred depth  
3. Construction / industrial / logistics boards — parallel product lanes
