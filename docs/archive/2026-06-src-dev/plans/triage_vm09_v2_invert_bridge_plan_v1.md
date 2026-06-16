# TRIAGE-VM-09-v2 — invert bridge plan `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **TRIAGE-VM-09-v2** |
| **Layer** | VM-09 **C** (structural) — [`vm09_gate_v1.md`](vm09_gate_v1.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` → `@coder` |
| **Status** | **SIGNED** — **DONE** (V2-A/B/C landed 2026-05-25) |
| **Blocks** | **S7B-M2-001**+ sim comm authority · **not** S7B-M1 / preflight / FULL_APP spine |
| **Prereq (done)** | VM-09 slice 1+2 · **INFRA-PROJ2-001** · [`vm09_slice2_closure_signoff_v1.md`](vm09_slice2_closure_signoff_v1.md) |
| **Architecture** | [`view_runtime_architecture_v1.md`](view_runtime_architecture_v1.md) § VM-A/B/C |
| **Spec ref** | [`base_finsh_5.md`](../prompts/guides/base_finsh_5.md) **vm-09b** v2 |
| **Witness** | `debug_runs/infrastructure_view_isolation_live.json` |

**Planner deliverable only.** Coder executes in **2–3 PRs** max; no Stage 7 gameplay in same session.

---

## Why this gates S7B M2+

**S7B-M2** introduces **sim-session comm authority**: fixed-tick dispatch, stale-intel surfaces, and **Stage7IntelMap** tint tied to view pose. That requires **one authoritative pose path** per `ViewId` in the running sim — not a global `MapCameraDesired` that can drift from `ViewManager` after bridge sync.

| Phase | VM-09-v2 required? |
|:---|:---:|
| **S7B-PREFLIGHT** / **S7B-M1** (contracts, queue stub) | **No** |
| **S7B-M2** (dispatch delay + intel surfaces in sim) | **Yes** |
| **S7B-M3** (overlay readers) | **Yes** (same spine; no second extract) |

**Does not block:** Stage 5 FULL_APP · UI-OH 2/3 · construction · industrial · Wave P/C.

---

## Current state (v1 shipped)

```text
RTS / shell input
  → mut MapCameraDesired          (compatibility WRITE surface)
  → mirror_world_main_camera_from_map_desired
  → ViewProjectionAuthority.commit_pose(WorldMain)
  → SyncViewManager (sole ViewManager writer)
  → render / HUD readers prefer ViewManager + ViewId helpers
```

**Slice 2 done:** stray **readers** migrated (`gpu_particles`, `view_representation` zoom). **PROJ-2 done:** minimap/preview hit-test via `view_surface_*`.

**Remaining risk:** any code that still **writes** `MapCameraDesired` without going through authority (shell jumps, legacy sim HUD) can desync **WorldMain** vs **Stage7IntelMap** / comms overlay readers in M2.

---

## Target state (v2 invert)

```text
RTS / shell input
  → ViewProjectionAuthority.commit_pose / interaction (WRITE)
  → SyncViewManager
  → derive_map_camera_desired_from_view_authority (READ shim)
  → MapCameraDesired (compatibility READ-ONLY mirror for legacy APIs)
  → Bevy MainWorldCamera projection (unchanged schedule slot)
```

**Rule:** `MapCameraDesired` is **never** authoritative after v2. New code must not `ResMut<MapCameraDesired>` except the **single** derive system.

---

## PASS gates

| # | Criterion | Evidence |
|:---:|:---|:---|
| V2-1 | One writer to WorldMain pose | Grep: no `ResMut<MapCameraDesired>` outside derive + tests |
| V2-2 | RTS pan/zoom unchanged | Manual + `view_runtime` tests |
| V2-3 | Minimap shell does not teleport main | `vm_a.minimap_shell_wrote_map_camera_desired: false` |
| V2-4 | No dual writer | `vm_a.dual_writer_pose_violation: false` |
| V2-5 | Infra rollup green | `infrastructure_view_isolation_green: true` |
| V2-6 | Stage 5 spine | `cargo test -p proc_A_dine01 --lib stage5` |
| V2-7 | S7B gate field | `infrastructure_view_isolation_live.json` → `vm_09.triage_vm09_v2_green: true` (new) |

---

## Implementation phases (coder)

| PR | Scope | Primary files | Max files |
|:---:|:---|:---|:---:|
| **V2-A** | Invert WorldMain path: input → authority; derive `MapCameraDesired` | `map_camera.rs`, `view_authority.rs` | 3 |
| **V2-B** | Remove / redirect shell `MapCameraDesired` writes | `simulation_shell_phase2.rs`, `simulation_session.rs` | 2 |
| **V2-C** | Witness + lib tests + audit row | `live_proof` / isolation witness, `post_stage6_vm09_audit.md` | 3 |

### V2-A — core invert

| Step | Action |
|:---:|:---|
| 1 | `ApplyInput` mutates `ViewProjectionAuthority` for `ViewId::WorldMain` (or pending interaction buffer consumed before sync) |
| 2 | Replace `mirror_world_main_camera_from_map_desired` direction: **derive** desired from authority WorldMain pose |
| 3 | Keep schedule: `ApplyInput` → derive desired → `SyncViewManager` (order documented in `map_camera.rs`) |
| 4 | `view_projection_authority` docs: `MapCameraDesired` fallback **deprecated** for new call sites |

### V2-B — shell audit

| Site | Action |
|:---|:---|
| `simulation_shell_phase2.rs` | Route session camera sync through `commit_map_camera_pose_to_view_authority` only |
| `construction/rail/input.rs` | Review — input must not write global desired |
| `map_camera.rs` legacy `sim_map_screen_to_world_xy` | Document or route via `ViewId::SimulationMap` |

### V2-C — witness

Add to `infrastructure_view_isolation_live.json`:

```json
"vm_09": {
  "triage_vm09_coder_b_green": true,
  "triage_vm09_v2_green": true,
  "invert_bridge": "ViewProjectionAuthority_write_MapCameraDesired_derive"
}
```

**Lib tests:**

```powershell
cargo test -p proc_A_dine01 --lib view_runtime vm09_slice2 steward_vm09_infrastructure_witness_refresh stage5
```

---

## Forbidden

| Wrong | Correct |
|:---|:---|
| S7B dispatch + invert bridge same PR | Finish v2 first, then **S7B-M2-001** |
| New `MapCameraDesired` writers in `src/strategic/` | Queue + overlays use view authority |
| Reopen slice 2 `view_representation` zoom path | Already **CLOSED** |
| Second minimap extract for S7 | M3 reads compositor snapshots |

---

## Gate chain (S7B)

```text
VM-09 slice 2 + PROJ-2     ☑ CLOSED
        │
        ▼
TRIAGE-VM-09-v2 (this)      ☑ CLOSED
        │
        ▼
S7B-M2-001 dispatch delay    ☑ DONE
        │
        ▼
S7B-M3-001 overlay readers   ☑ DONE
```

---

## Copy-paste — @coder

```
Track: TRIAGE-VM-09-v2
Read: docs/archive/2026-06-src-dev/plans/triage_vm09_v2_invert_bridge_plan_v1.md
      docs/archive/2026-06-src-dev/plans/post_stage6_vm09_audit.md
      src/gui/view_authority.rs (module docs)
Prereq: vm09_slice2_closure_signoff CLOSED
Deliver: invert WorldMain write path + derive MapCameraDesired + vm_09.triage_vm09_v2_green
Verify: cargo test -p proc_A_dine01 --lib view_runtime vm09_slice2 steward_vm09_infrastructure_witness_refresh stage5
Do NOT: S7B-M2 dispatch, new strategic MapCameraDesired writers, reopen CODER-B zoom
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | **SIGNED** — unblocks **S7B-M2+** only |
