# S-VM-09 gate — VM-09 slice 1 `v1`

| Field | Value |
|:---|:---|
| **Lane ID** | `S-VM-09` |
| **Date** | 2026-05-24 |
| **Track** | [`stages/infra_55_execution_plan_v1.md`](stages/infra_55_execution_plan_v1.md) · **INFRA-VM09-001** |
| **Backlog** | [`stage5_triage_backlog.md`](stage5_triage_backlog.md) **TRIAGE-VM-09** |
| **Audit** | [`post_stage6_vm09_audit.md`](post_stage6_vm09_audit.md) |
| **Witness** | [`debug_runs/infrastructure_view_isolation_live.json`](../../debug_runs/infrastructure_view_isolation_live.json) |

**Scope:** Slice 1 only — one stray `MapCameraDesired` reader migrated + audit updated. **Not** full VM-09 invert bridge, **not** PROJ-2 sweep, **not** VM-10/11.

---

## Verdict: **GO (slice 1)**

| Gate | Required | Verdict |
|:---|:---|:---:|
| **G-VM09-01** | Audit doc lists readers + next sweep | ✅ [`post_stage6_vm09_audit.md`](post_stage6_vm09_audit.md) |
| **G-VM09-02** | ≥1 callsite migrated with ViewManager preference | ✅ `gpu_particles.rs` `sync_fire_particle_camera_scale` |
| **G-VM09-03** | `dual_writer_pose_violation: false` | ✅ witness `vm_a` |
| **G-VM09-04** | `minimap_shell_wrote_map_camera_desired: false` | ✅ witness `vm_a` |
| **G-VM09-05** | `infrastructure_view_isolation_green: true` | ✅ |
| **G-VM09-06** | Lib tests green | ✅ view_runtime **10/10**, gpu_particles **18/18** |
| **G-VM09-07** | Stage 5 not regressed | ✅ run `stage5` after merge |

**Track exit (full TRIAGE-VM-09):** ☐ **OPEN** — invert bridge + PROJ-2 sweep (see **Finish organizer** below).

---

## Finish organizer — what VM-09 is (and is not)

**VM-09 does not block Stage 5 / FULL_APP / construction / industrial play.** Those gates are **closed**. Docs that still say “GATED (VM-09)” for **S7P** or **S7-BEHAV** mean **full invert bridge + projection sweep**, not “app won’t run.”

### Three layers

| Layer | What it means | Status | Blocks |
|:---|:---|:---:|:---|
| **A — Operational** | No dual WorldMain writer; infra witness green | ✅ **DONE** | Nothing on spine |
| **B — Reader migration** | Stray `MapCameraDesired` *readers* use `ViewManager` / view authority | **2/4** slices | FX zoom drift if regressed |
| **C — Structural (v2)** | Invert bridge: input → `ViewManager` only, derive `MapCameraDesired` | ☐ **OPEN** | Backlog **TRIAGE-VM-09**; planner-sized |

### Layer B — remaining work (finish “practical VM-09”)

| Step | ID | Files | Done when | Agent |
|:---:|:---|:---|:---|:---|
| 1 | ~~Slice 1~~ | `gpu_particles.rs` | ✅ | — |
| 2 | ~~Slice 2 / CODER-B~~ | `view_representation.rs` | ✅ steward signed | — |
| 3 | ~~**PROJ-2a**~~ | `tile_world_fallback.rs` | ☑ `view_surface_screen_to_world(ViewId::Minimap, …)` | **done** |
| 4 | ~~**PROJ-2b**~~ | `editor/world_preview/interaction.rs` | ☑ `ViewId::WorldPreview` hover | **done** |
| 5 | **PROJ-2 audit** | `post_stage6_vm09_audit.md` | ≥5 sites migrated or inventoried (infra plan **I2**) | `@coder` |
| 6 | **VM-10** (optional) | `view_runtime` | Only if operator reports minimap/main lockstep | `@coder` |

**Verify after 3–5:** `cargo test -p proc_A_dine01 --lib view_runtime vm09_slice2 stage5` · refresh `infrastructure_view_isolation_live.json`.

### Layer C — defer unless you need backlog closure

| Item | Source | Size |
|:---|:---|:---:|
| Invert bridge (v2) | `base_finsh_5.md` vm-09b | Multi-PR / planner |
| `construction/rail/input.rs` review | audit table | 1 PR if needed |
| Close **TRIAGE-VM-09** row in `stage5_triage_backlog.md` | After B complete + steward sign-off | Doc |

### What incorrectly still shows “blocked by VM-09”

| Lane | Real blocker | Unblock when |
|:---|:---|:---|
| **S7P Play** | Nothing (witness green) | Already unblocked |
| **S7-BEHAV / S7B-DESIGN-001** | Worksheet + planner plan, not slice 2 | Mark gate = **PROJ-2 + v2**, or **B done** policy |
| **Construction ghosts on sim map** | DQ-POST-04 policy | VM-09 **B** + Wave P stable |
| **Wave C depth** | Explicitly deferred WC-DEPTH-003 | Not VM-09 |
| **Ledger / coder workboard** | Stale **OPEN** on CODER-B | Reconcile to ✅ (2026-05-25) |

### Recommended sequence (one coder, ~2–3 PRs)

```text
PR1  INFRA-PROJ2-001a  tile_world_fallback minimap hit-test
PR2  INFRA-PROJ2-001b  world_preview interaction
PR3  (optional) VM-10   only if witness shows lockstep_suspect
──── planner track ────
TRIAGE-VM-09-v2         invert bridge (separate milestone)
```

**Steward:** re-run **STEWARD-VM-09-001** after PR1+PR2 only (witness refresh). **Do not** wait for v2 to ungate behavioral design.

---

## Slice 1 change

**File:** `src/render/gpu_particles.rs`

`sync_fire_particle_camera_scale` now resolves zoom via `camera_zoom(&ViewManager, ViewId::WorldMain)` first, then `MapCameraDesired` fallback (matches `weather_visual.rs` pattern).

**Rationale:** Fire spark tactical cull (`zoom_alpha`) must track **view authority** pose, not a parallel global desired scale that can drift from `ViewManager` after bridge sync.

---

## Witness snapshot (2026-05-24)

`infrastructure_view_isolation_live.json`:

| Field | Value |
|:---|:---|
| `infrastructure_view_isolation_green` | `true` |
| `vm_a.dual_writer_pose_violation` | `false` |
| `vm_a.minimap_shell_wrote_map_camera_desired` | `false` |
| `vm_10.minimap_lockstep_suspect` | `false` |

---

## Slice 2 — TRIAGE-VM-09-CODER-B (2026-05-25)

**File:** `src/gui/view_representation.rs`

`sync_camera_visual_state_from_map_camera` uses [`resolve_world_main_camera_scale`] — `camera_zoom(ViewManager, WorldMain)` first, `MapCameraDesired` fallback (matches `gpu_particles.rs`).

| Gate | Verdict |
|:---|:---:|
| G-VM09-02b | ≥1 additional reader migrated | ✅ `view_representation.rs` |
| G-VM09-03..05 | Witness unchanged | ✅ re-run `view_runtime` tests |

---

## Remaining VM-09 debt (post slice 2)

| Reader | File | Priority |
|:---|:---|:---|
| ~~Tile fallback focus~~ | `tile_world_fallback.rs` | ☑ **INFRA-PROJ2-001** |
| Construction rail | `construction/rail/input.rs` | review (input path) |
| Diagnostics | `full_render_diagnostic.rs`, `visual_diagnostics.rs` | read-only OK |

**PROJ-2:** route `world_to_screen` bypasses → **INFRA-PROJ2-001** (separate slice).

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib view_runtime gpu_particles stage5
cargo run -p proc_A_dine01 --release -- --test visual
```

---

## STEWARD-VM-09-001 (steward infra slice)

**Board:** [`steward_vm09_gate_v1.md`](steward_vm09_gate_v1.md) — **GO** 2026-05-24

| Check | Result |
|:---|:---|
| Slice 1 + 2 | ✅ `gpu_particles` + `resolve_world_main_camera_scale` |
| Witness refresh | ✅ `steward_vm09_infrastructure_witness_refresh` |
| `view_runtime` tests | **11/11** |
| **INFRA-PROJ2-001** | ☑ **done** — `infra_proj2_001_green: true` |
| `view_representation.rs` reader | ✅ TRIAGE-VM-09-CODER-B |

**Verdict:** **GO** — infrastructure witness green; PROJ-2 sweep delegated.

---

## Route

| Next | Agent |
|:---|:---|
| **OPS-F01** → **WC-D04** | operator → `@coder` — per [`post_stage6_infra_slice2_plan_v1.md`](post_stage6_infra_slice2_plan_v1.md) |
| **INFRA-VM10-001** | `@coder` — lockstep diagnostics hardening |
| **TRIAGE-VM-09-v2** | `@coder` — [`triage_vm09_v2_invert_bridge_plan_v1.md`](triage_vm09_v2_invert_bridge_plan_v1.md) (layer C; blocks **S7B-M2+**) |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.1.0 | 2026-05-24 | STEWARD-VM-09-001 monitoring pass |
| v1.0.0 | 2026-05-24 | S-VM-09 slice 1 **GO** — gpu_particles zoom via ViewManager |
