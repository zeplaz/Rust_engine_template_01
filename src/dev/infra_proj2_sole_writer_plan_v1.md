# INFRA PROJ-2 — sole writer + per-view projection plan `v1` (PLAN-INFRA-PROJ2-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-INFRA-PROJ2-001** |
| **Coder slices** | **INFRA-PROJ2-001** (hit-test) · **INFRA-PROJ2-CODER-B** (sole writer + read surface) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` |
| **Status** | **CLOSED** — do not re-queue PROJ2 sweep |
| **Parent** | [`post_stage6_infra_slice2_plan_v1.md`](post_stage6_infra_slice2_plan_v1.md) Part A · [`vm09_slice2_closure_signoff_v1.md`](vm09_slice2_closure_signoff_v1.md) |
| **Audit** | [`post_stage6_vm09_audit.md`](post_stage6_vm09_audit.md) |
| **Witness** | [`debug_runs/infrastructure_view_isolation_live.json`](../../debug_runs/infrastructure_view_isolation_live.json) |

**No Rust.** Planner rollup for **PROJ-2**: per-view screen↔world isolation **and** **ViewManager** single-writer policy (VM-06). Distinct from **TRIAGE-VM-09-CODER-B** (WorldMain zoom in `view_representation.rs`).

---

## Naming guard (do not conflate)

| ID | Scope | Status |
|:---|:---|:---:|
| **TRIAGE-VM-09-CODER-B** | `resolve_world_main_camera_scale` — camera visual / FX band | **DONE** |
| **INFRA-PROJ2-001** | Minimap + World Preview `view_surface_screen_to_world` | **DONE** |
| **INFRA-PROJ2-CODER-B** | ViewManager **sole** `ResMut` writer + read-only projection API | **DONE** (VM-06) |

---

## Track map

| Track | ID | Goal | Status |
|:---|:---|:---|:---:|
| **PROJ2-A** | **INFRA-PROJ2-001** | Hit-test uses per-view camera, not WorldMain bleed | **CLOSED** |
| **PROJ2-B** | **INFRA-PROJ2-CODER-B** | `ViewManager` rebuilt only by bridge; reads via `view_projection_authority` | **CLOSED** |
| **VM-09 zoom** | **TRIAGE-VM-09-CODER-B** | Separate lane — see steward gate | **CLOSED** |

**Closure policy:** [`vm09_slice2_closure_signoff_v1.md`](vm09_slice2_closure_signoff_v1.md) — **no rework** on hit-test sweep or `view_representation` zoom without new drift witness.

---

## Master gate chain

```text
S-VM-09 slice 1 (gpu_particles zoom)              ☑
        │
        ▼
TRIAGE-VM-09-CODER-B (WorldMain scale resolver)   ☑
        │
        ▼
INFRA-PROJ2-CODER-B (ViewManager sole writer)     ☑ VM-06
        │
        ▼
INFRA-PROJ2-001 (Minimap + Preview hit-test)      ☑
        │
        ▼
STEWARD-VM-09-001 slice 2                         ☑ CLOSED
        │
        └─► TRIAGE-VM-09-v2 (invert bridge)       ☐ separate planner lane
```

---

## PROJ2-B — ViewManager sole writer (**INFRA-PROJ2-CODER-B**)

### Policy (VM-06)

| Resource | Sole `ResMut` writer | Read surface |
|:---|:---|:---|
| **`ViewManager`** | `sync_view_manager_bridge` in [`view_authority.rs`](../gui/view_authority.rs) | [`view_projection_authority.rs`](../gui/view_projection_authority.rs) — `&ViewManager` only |
| **`ViewProjectionAuthority`** | `commit_*` systems in view runtime | Pose bus before bridge rebuild |
| **`ResolvedViewports`** | `viewport_pipeline` (+ VM-C2 mirror in `view_runtime/commit.rs`) | Measure after authority commit |

**Code comment (authoritative):**

```7:9:src/gui/view_authority.rs
//! **VM-06:** [`sync_view_manager_bridge`] is the **sole** `ResMut<ViewManager>` writer — it rebuilds the
//! read model from authority after viewport resolve.
```

**Static scan:** [`tools/orchestrator/src/authority_scan.rs`](../../tools/orchestrator/src/authority_scan.rs) — canonical path `src/gui/view_authority.rs` for `ViewManager`; non-canonical `ResMut` sites → **LOW** alert.

### PROJ2-B deliverables (**met**)

| # | Criterion | Evidence |
|:---:|:---|:---|
| B1 | Exactly one production `ResMut<ViewManager>` | `rg ResMut<ViewManager` → `view_authority.rs` only |
| B2 | Bridge runs after viewport resolve + pose commits | `ViewAuthoritySystemSet::SyncViewManager` |
| B3 | Witness names sole writer | `vm_06.view_manager_sole_writer: "sync_view_manager_bridge"` |
| B4 | `view_projection_authority_pose_bus: true` | live proof payload |
| B5 | No `dual_writer_pose_violation` | `vm_a.dual_writer_pose_violation: false` |

### MapCameraDesired (compatibility — not PROJ2 read path)

- **Input writer:** RTS / shell (`map_camera.rs`) — allowed.
- **Render consumers:** must use `ViewManager` + `ViewId` helpers; `MapCameraDesired` fallback only when bridge not populated ([`view_projection_authority.rs`](../gui/view_projection_authority.rs) module docs).

---

## PROJ2-A — per-view hit-test (**INFRA-PROJ2-001**)

### Canonical API

```rust
view_surface_screen_to_world(manager: &ViewManager, id: ViewId, screen, image_rect, tw, th)
view_surface_world_to_screen(manager: &ViewManager, id: ViewId, ...)
```

Low-level math stays in `map_view_projection.rs`; **callers pass `ViewId`**.

### Migrated call sites (**done**)

| File | `ViewId` | Steward |
|:---|:---|:---|
| [`tile_world_fallback.rs`](../render/tile_world_fallback.rs) | `Minimap` | egui minimap click / focus |
| [`editor/world_preview/interaction.rs`](../gui/editor/world_preview/interaction.rs) | `WorldPreview` | hover tile pick |

### PROJ2-A deliverables (**met**)

| # | Criterion | Evidence |
|:---:|:---|:---|
| A1 | Minimap screen→world ≠ WorldPreview screen→world for same pixel | unit test |
| A2 | Witness `infra_proj2_001_green: true` | `infrastructure_view_isolation_live.json` |
| A3 | `triage_vm09_coder_b_green: true` | no dual-writer / minimap shell bleed |

**Lib test:**

```powershell
cargo test -p proc_A_dine01 --lib infra_proj2_view_surface_screen_to_world_isolates_minimap_and_preview
```

Defined in [`isolation_tests.rs`](../render/view_runtime/isolation_tests.rs).

---

## Witness bundle (fleet truth 2026-05-25)

| Field | Value |
|:---|:---|
| `infrastructure_view_isolation_green` | `true` |
| `vm_06.view_manager_sole_writer` | `sync_view_manager_bridge` |
| `vm_09.infra_proj2_001_green` | `true` |
| `vm_09.triage_vm09_coder_b_green` | `true` |
| `vm_a.dual_writer_pose_violation` | `false` |
| `vm_10.minimap_lockstep_suspect` | `false` |

**Regression suite:**

```powershell
cargo test -p proc_A_dine01 --lib stage5 view_runtime vm09_slice2 steward_vm09_infrastructure_witness_refresh
```

Optional operator:

```powershell
cargo run -p proc_A_dine01 --release -- --test visual
```

---

## Copy-paste — INFRA-PROJ2-CODER-B (archive — done)

```
Lane: INFRA-PROJ2-CODER-B — ViewManager sole writer (VM-06)
Read: src/dev/infra_proj2_sole_writer_plan_v1.md § PROJ2-B
      src/gui/view_authority.rs (sync_view_manager_bridge)
      src/dev/recovery_viewport.md § single-writer map
Do NOT: add ResMut<ViewManager> outside view_authority.rs
Do NOT: read MapCameraDesired for minimap/preview hit-test (use ViewId helpers)
Verify: cargo orchestrate authority scan; cargo test -p proc_A_dine01 --lib view_runtime
Witness: vm_06.view_manager_sole_writer + dual_writer_pose_violation false
```

---

## Copy-paste — INFRA-PROJ2-001 (archive — done)

```
Lane: INFRA-PROJ2-001 — minimap + world preview hit-test
Read: src/dev/infra_proj2_sole_writer_plan_v1.md § PROJ2-A
      src/dev/post_stage6_vm09_audit.md § PROJ-2 bypass inventory
First: tile_world_fallback.rs → view_surface_screen_to_world(ViewId::Minimap, …)
Second: world_preview/interaction.rs → ViewId::WorldPreview
Max files: 3 per PR
Verify: cargo test -p proc_A_dine01 --lib infra_proj2_view_surface_screen_to_world_isolates_minimap_and_preview
Witness: infra_proj2_001_green: true
```

---

## Forbidden

| Pattern | Reason |
|:---|:---|
| New `ResMut<ViewManager>` outside `view_authority.rs` | Breaks VM-06 sole writer |
| Hit-test from `MapCameraDesired` alone for Minimap/Preview | Cross-view bleed |
| Re-open PROJ2 for Stage 5 predicate edits | Infra lane only |
| Confuse **INFRA-PROJ2-CODER-B** with **TRIAGE-VM-09-CODER-B** | Different files / witnesses |
| Minimap writes `MapCameraDesired` | `minimap_shell_wrote_map_camera_desired` witness flag |

---

## Open tails (not PROJ2)

| ID | Goal | Owner |
|:---|:---|:---|
| **TRIAGE-VM-09-v2** | Invert bridge — structural | planner → coder |
| **map_camera.rs** | `sim_map_screen_to_world_xy` legacy sim HUD | review |
| **OPS-F01** | 60s perf attribution | operator |
| **WC-D04** | Wave C depth slice | coder B (Part B plan) |

---

## Sign-off

| Role | Date | Status |
|:---|:---|:---|
| Planner | 2026-05-25 | **SIGNED** — PLAN-INFRA-PROJ2-001 |
| Coder PROJ2-A/B | 2026-05-25 | **CLOSED** |
| Sim-steward | 2026-05-25 | **STEWARD-VM-09-001** slice 2 |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | PROJ2 sole writer + hit-test rollup; INFRA-PROJ2-CODER-B documented |
