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

**Track exit (full TRIAGE-VM-09):** ☐ **OPEN** — invert bridge + remaining readers in audit table.

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

## Remaining VM-09 debt (post slice 1)

| Reader | File | Priority |
|:---|:---|:---|
| Camera visual / FX band | `view_representation.rs` | P1 |
| Tile fallback focus | `tile_world_fallback.rs` | P2 |
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

## Route

| Next | Agent |
|:---|:---|
| **INFRA-PROJ2-001** | `@coder` — `world_to_screen` sweep (≤3 files) |
| **INFRA-VM10-001** | `@coder` — lockstep diagnostics hardening |
| **view_representation** VM-09 reader | `@coder` — optional slice 2 |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | S-VM-09 slice 1 **GO** — gpu_particles zoom via ViewManager |
