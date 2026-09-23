# RPC-2 — pose authority completion (TODO-04) `v1`

**Program:** PLAN-RENDER-PROD-CLEANUP-v1 · Phase 2  
**Stage5 row:** TODO-04 — *ViewManager sole authority for WorldMain camera pose after bridge; MapCameraDesired only mirrored*  
**Queue:** `tools/orchestrator/queues/render_production_cleanup_queue.json`  
**Parent:** [`plan_render_production_cleanup_v1.md`](plan_render_production_cleanup_v1.md)  
**Prior invert (SIGNED DONE, residual clobber remains):** [`triage_vm09_v2_invert_bridge_plan_v1.md`](triage_vm09_v2_invert_bridge_plan_v1.md) · skill [`01-view-authority-viewmanager.md`](../../.cursor/skills/bevy-simulation-grade/01-view-authority-viewmanager.md) · spine [`07-repo-authority-map.md`](../../.cursor/skills/bevy-simulation-grade/07-repo-authority-map.md)  
**Planner packet:** RPC-2-001 ★ **DONE 2026-08-12** — this freeze. Coder implements RPC-2-002…004.

---

## Summary

Finish TODO-04 by making **`ViewProjectionAuthority` the sole WorldMain / SimulationMap pose commit surface**. `MapCameraDesired` (component + `MapCameraDesiredRes`) becomes a **compatibility mirror only**, filled by **`derive_map_camera_desired_from_view_authority`**. `ViewManager` stays a **frame-rebuilt read model** via **`sync_view_manager_bridge`** (VM-06) — it is **not** a pose writer.

This kills the startup / frame-1 clobber class (`ZOOM_REVERT 1.0→0.02`, alpha-state default) caused by **desired→authority** sync in `ApplyInput` overwriting an authority commit from the same frame.

---

## Current Problems

| # | Problem | Evidence |
|:---|:---|:---|
| P1 | **Hot path still desired-first** | `map_camera.rs` module doc + schedule: ApplyInput mutates `MapCameraDesired` component → `mirror_map_camera_component_to_resource` → `sync_map_camera_pose_to_view_authority` (**desired → authority**) |
| P2 | **Derive fights ApplyInput** | `derive_map_camera_desired_from_view_authority` (authority → desired) runs in `DeriveDesired` *after* ApplyInput — correct direction, but only after a same-frame reverse write |
| P3 | **Startup dual-write hack** | `focus_main_camera_on_world_params` must write component **and** `commit_map_camera_pose_to_view_authority` because ApplyInput sync would otherwise republish spawn-default pose (comment at `tile_world_fallback.rs` ~643–647) |
| P4 | **Doc / skill drift** | Skill 01 + TRIAGE-VM-09-v2 declare invert DONE; live ApplyInput still implements the pre-invert chain; TODO-04 / TODO-05 remain Open |
| P5 | **Misleading labels** | `MapCameraDesired` comment still says “ECS authority”; `ViewManager` is often called “sole authority” in TODO-04 wording — precise truth: **authority = `ViewProjectionAuthority`**, **read spine = `ViewManager`**, **mirror = `MapCameraDesired`** |

**Already correct (do not regress):**

- `apply_minimap_camera_intent` → `commit_pose(Minimap, …, MinimapShell)` only — **never** WorldMain / `MapCameraDesired`
- Minimap click-to-focus main → `commit_map_camera_pose_to_view_authority` (read-modify-commit)
- `simulation_session` focus helper → authority commit (Transform only for camera entity)
- `ViewProjectionAuthority` → `extracted_camera_metrics` single-writer/multi-reader (parent plan lock F10)
- Locks: do not touch ViewProjectionAuthority *contract* shape or `stage6_virtualization`

---

## Target Architecture

```text
◎authority
⊚ViewProjectionAuthority ═▶ ⊨WorldMain∧SimulationMap pose
⊚ViewManager              ⊰ ⊚ViewProjectionAuthority  (via sync_view_manager_bridge after ViewportResolve)
⊚MapCameraDesired[+Res]   ⊰ ⊚ViewProjectionAuthority  (via derive_map_camera_desired_from_view_authority)
MainWorldCamera Transform/Ortho  ◂⊳[snapshot] ⊚MapCameraDesired   (Smooth / ApplyCameraProjection — presentation)

◎write-paths (WorldMain)
  RTS pan/zoom/wheel     → commit_pose(WorldMain[+SimulationMap], MapCameraInput)
  Startup / world focus  → commit_pose(…, MapCameraInput)  [optional Transform seed same frame]
  Minimap jump-to-main   → commit_pose(WorldMain[+SimulationMap], MapCameraInput)
  Minimap shell chrome   → commit_pose(Minimap, MinimapShell)   ⛔ ¬WorldMain
  FollowCamera           → commit_pose(Minimap, MinimapFollow)  ⛔ ¬WorldMain

◎forbidden
  ResMut<MapCameraDesired[Res]> outside derive (+ tests)
  sync_map_camera_pose_to_view_authority (desired→authority) in production schedule
  UI / render / extraction writing pose
```

### Single commit point

| Surface | Sole writer API | Writer tag |
|:---|:---|:---|
| WorldMain + SimulationMap pose | `ViewProjectionAuthority::commit_pose` / `commit_pose_traced` via `commit_map_camera_pose_to_view_authority` (+ `_simple` only if still needed without trace) | `ViewAuthorityWriter::MapCameraInput` |
| Minimap pose | same `commit_pose` | `MinimapShell` / `MinimapFollow` |
| `MapCameraDesired` + Res | **only** `derive_map_camera_desired_from_view_authority` | n/a (mirror) |
| `ViewManager` | **only** `sync_view_manager_bridge` | VM-06 |

### Mirror direction (frozen)

```text
ViewProjectionAuthority  ──derive──▶  MapCameraDesired component + MapCameraDesiredRes
ViewProjectionAuthority  ──bridge──▶  ViewManager.views[WorldMain].camera
MapCameraDesired  ──Smooth/Ortho──▶  Bevy Transform / OrthographicProjection   (presentation)
⛔ MapCameraDesired ──sync──▶ ViewProjectionAuthority   (retire)
```

### Schedule (must hold)

```text
Input / shell commits
  ═▶ MapCameraSystemSet::ApplyInput     (authority commits only; ¬desired mutate)
  ═▶ MapCameraSystemSet::DeriveDesired  (sole MapCameraDesired writer)
  ═▶ MapCameraSystemSet::Smooth         (presentation toward desired)
  ═▶ ViewportPipelineSet::Resolve
  ═▶ apply_minimap_camera_intent        (Minimap surface only; before SyncViewManager)
  ═▶ ViewAuthoritySystemSet::SyncViewManager
  ═▶ build_view_representation_snapshot / FireVisualFrameSet / extraction
```

---

## Writer enumeration (live → target)

| Site | Today | Target |
|:---|:---|:---|
| `map_camera_apply_input` | mut `MapCameraDesired` + Transform | Compute delta → `commit_pose(WorldMain[+SimMap], MapCameraInput)`; Transform optional presentation seed |
| `map_camera_wheel_zoom_system` | mut `MapCameraDesired` | Same — authority commit only |
| `mirror_map_camera_component_to_resource` | Comp → Res | **Delete** or fold into derive |
| `sync_map_camera_pose_to_view_authority` | Comp → authority | **Retire from schedule** |
| `derive_map_camera_desired_from_view_authority` | authority → Comp+Res | **Keep** — sole mirror writer |
| `focus_main_camera_on_world_params` | Comp + Transform + authority | Authority (+ Transform seed); **¬** Comp write — derive fills Comp/Res |
| `simulation_session` focus helper | authority + Transform | Keep |
| `minimap_bevy_interaction` jump | authority RMW | Keep |
| `apply_minimap_camera_intent` | Minimap authority | Keep — assert ¬`MapCameraDesired` |
| `vfx_fire_test_highlight` / harness | authority commit | Keep (test/debug writers tagged) |
| Construction / HUD / pick | `Res<MapCameraDesiredRes>` readers | Prefer `tactical_camera_world_pose(authority, …)` over time; **no write** in this phase |

---

## Implementation Phases (coder DAG)

### ★RPC-2-002 — Invert ApplyInput hot path (DONE 2026-08-12)
**Goal:** Pan/keys/wheel commit `ViewProjectionAuthority` first; remove desired→authority sync from `ApplyInput` chain.  
**Files:** `src/gui/tactical/map_camera.rs` (± thin helpers in `view_authority.rs`)  
**⊚authority-owner:** `ViewProjectionAuthority` / `MapCameraInput`  
**Risks:** One-frame feel change if Transform not seeded; param-count limits (prefer `_simple` commit without trace).  
**Diagnostics:** `RUST_LOG=map_camera_desired::write=debug` — ApplyInput must **not** emit `MAP_CAMERA_DESIRED_WRITE`; only derive may.  
**Migration-compat:** Keep `MapCameraDesired` type as value object for commit payload construction.  
**Rollback-trigger:** RTS pan/zoom broken in Simulation/Editor → revert slice.  
**Acceptance:**
- Schedule: ApplyInput has **no** `sync_map_camera_pose_to_view_authority`
- Grep: production ApplyInput systems do not `Query<&mut MapCameraDesired>`
- `cargo test -p proc_A_dine01 --lib view_runtime`
- ★RPC-2-002-done

### ★RPC-2-003 — Startup / focus clobber kill (DONE 2026-08-12)
**Goal:** `focus_main_camera_on_world_params` (and any sibling OnEnter focus) authority-only; remove dual-write hack comments.  
**Files:** `src/render/tile_world_fallback.rs`, `src/gui/hud/simulation_session.rs` (audit only if still dual)  
**⊚authority-owner:** same  
**Risks:** Frame-0 zoom if DeriveDesired not run same schedule tick after OnEnter — order OnEnter commit **before** first Update Derive, or call derive helper inline once after commit.  
**Diagnostics:** no `ZOOM_REVERT` / post_mirror drift spike on sim enter; TODO-04 bridge drift ≤ `MAP_BRIDGE_DRIFT_OK`.  
**Acceptance:**
- Focus paths: **zero** `*desired =` / `Query<&mut MapCameraDesired>` writes
- FULL_APP or lib stage5 map-camera bridge witness: `last_post_mirror_drift` OK
- ★RPC-2-003-done
- **Shipped:** focus commits authority + Transform seed only; `sync_minimap_follow_camera_on_sim_enter` reads authority (not desired Res); shared `apply_derived_map_camera_desired` extracted for DeriveDesired.

### ★RPC-2-004 — Mirror hygiene + TODO-04 exit (DONE 2026-08-12)
**Goal:** Docs/module comments match invert; retire dead aliases; close Stage5 TODO-04 predicate + witness fields.  
**Files:** `map_camera.rs` docs, `stage5_live_todos` predicate consumers / isolation witness, `view_runtime` tests  
**⊚authority-owner:** `ViewProjectionAuthority` / `MapCameraInput`  
**Risks:** Closing TODO-04 without TODO-05 (hidden second writer) — keep TODO-05 open until BridgeCompat residual cleared.  
**Acceptance:**
- Grep gate: sole production `ResMut<MapCameraDesiredRes>` / `Query<&mut MapCameraDesired>` writer = `derive_map_camera_desired_from_view_authority`
- `vm_a.minimap_shell_wrote_map_camera_desired: false`
- `vm_a.dual_writer_pose_violation: false`
- `cargo test -p proc_A_dine01 --lib stage5` + `view_runtime`
- Witness note path under `debug_runs/` (isolation or stage5_full_app live) citing invert complete
- ★RPC-2-004-done → Stage5 TODO-04 may flip Done via live predicate (not queue alone)
- **Shipped:** retired `mirror_world_main_camera_from_map_desired` alias; MapCameraDesired “ECS authority” wording → compatibility mirror; extracted_camera_metrics doc no longer cites desired→authority sync.

---

## ECS Schedule Plan

```text
ApplyInput        ═▶ authority commits (MapCameraInput)
DeriveDesired     ═▶ MapCameraDesired mirror
Smooth            ═▶ presentation toward mirror
ViewportResolve   ═▶ ResolvedViewports
MinimapIntent     ═▶ Minimap commit_pose (before SyncViewManager)
SyncViewManager   ═▶ ViewManager rebuild
CameraApply / PostUpdate SimulationViewportSyncSet::ApplyCameraProjection
VisibilityExtract / RenderPrepare  ⊰ ViewManager + authority snapshots
```

---

## Diagnostics Required

| Channel | Use |
|:---|:---|
| `map_camera_desired::write` | Prove only derive mutates desired |
| Stage5 `STAGE5_MAP_CAMERA_HOOK` / bridge drift | post_mirror ≤ threshold |
| `infrastructure_view_isolation_live.json` `vm_a.*` | dual_writer + minimap_shell flags |
| `ViewRuntimeTrace` / `last_pose_writer` | WorldMain last writer = MapCameraInput (or expected shell tag) — never Unset after first commit |
| Lib: `view_runtime::isolation_tests` | bridge upsert / writer tag |

---

## Edge Cases

- **Chunk / world resize:** focus OnEnter must re-commit authority; derive fills mirror next tick.
- **Minimap FollowCamera:** may write Minimap surface only; must not touch WorldMain desired.
- **ActiveMapViewInput::Minimap:** ApplyInput already early-returns grip/scroll — preserve.
- **Test harness / VFX proof zoom:** still go through commit API with tagged writer if needed.
- **Readers still on `MapCameraDesiredRes`:** allowed; prefer `tactical_camera_world_pose` for new code (out of scope for force-migrate).
- **Do not** reopen `tile_world_fallback` god-split (RPC-3-001) in this phase — touch focus fn only.

---

## Risks

| Risk | Mitigation |
|:---|:---|
| Same-frame ortho uses stale desired before Derive | Seed Transform from committed pose in ApplyInput **or** run derive immediately after commit in same system chain |
| Accidental second WorldMain writer via Minimap | Keep writer-tag dual-writer assert; VM-A tests |
| Scope creep into construction projection | Readers only; no construction ResMut |
| Confusing “ViewManager sole authority” language | Packet language: ViewManager = read spine; Authority = pose truth |

---

## Coder exit predicates (rollup)

```text
★RPC-2-pose-done ⇔
  (1) no production desired→authority sync scheduled
  (2) sole MapCameraDesired writer = derive_…
  (3) focus/startup ¬mut MapCameraDesired
  (4) minimap shell ¬MapCameraDesired
  (5) view_runtime + stage5 lib green
  (6) witness: dual_writer_pose_violation=false ∧ post_mirror_drift OK
```

---

## Open Questions

| ⌁? | Note |
|:---|:---|
| Inline derive after OnEnter commit vs rely on Update order? | Prefer **one** shared `apply_derived_map_camera_desired(authority, …)` helper callable from focus **and** DeriveDesired system — avoids frame-0 hole without dual-write |
| Retire `MapCameraDesired` **component** later? | Out of scope — Res+component mirror OK through RPC-2-004; component removal = future DR |
| TODO-05 same PR? | **No** — RPC-2-004 may *feed* TODO-05 evidence; keep TODO-05 separate if any BridgeCompat path remains |

---

## Anti-goals

- Rewriting `ViewProjectionAuthority` schema / ViewSurfaceId set  
- Dual-writing Comp+authority “just to be safe”  
- Folding Phase 3 `tile_world_fallback` split into pose work  
- Claiming TODO-04 Done on queue row alone without live predicate / witness  

---

## Slice ladder

| id | Owner | Goal | Exit |
|:---|:---|:---|:---|
| **RPC-2-001** | planner | This freeze | plan path + queue ready for coder · **DONE 2026-08-12** |
| **RPC-2-002** | coder | Invert ApplyInput + wheel | desired→authority sync gone; view_runtime green · **DONE 2026-08-12** |
| **RPC-2-003** | coder | Startup/focus authority-only | no Comp write; bridge drift OK · **DONE 2026-08-12** |
| **RPC-2-004** | coder | Grep gate + docs + TODO-04 witness | dual_writer false; stage5 + view_runtime green · **DONE 2026-08-12** |

**Next pick after this packet:** `@coder` **RPC-2-002** (parallel OK with operator **RPC-1-006**; serialize vs any other `map_camera.rs` editor).
