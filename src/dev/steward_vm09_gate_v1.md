# STEWARD-VM-09-001 — infra slice 2+ `v1.1`

| Field | Value |
|:---|:---|
| **Lane ID** | `STEWARD-VM-09-001` |
| **Scope** | **Slice 2+ only** (`view_representation` — **TRIAGE-VM-09-CODER-B**) |
| **Date** | 2026-05-25 |
| **Audit** | [`post_stage6_vm09_audit.md`](post_stage6_vm09_audit.md) |
| **Witness** | [`debug_runs/infrastructure_view_isolation_live.json`](../../debug_runs/infrastructure_view_isolation_live.json) |

---

## Verdict: **CLOSED (slice 2)**

**TRIAGE-VM-09-CODER-B** is **landed and steward-signed**. Steward does **not** reopen slice 1 (`gpu_particles`) in this pass.

| Gate | Verdict |
|:---|:---:|
| `resolve_world_main_camera_scale` | ✅ ViewManager `WorldMain` before `MapCameraDesired` |
| `sync_camera_visual_state_from_map_camera` | ✅ uses resolver |
| `vm09_slice2_resolve_world_main_scale_prefers_view_manager` | ✅ **1/1** |
| `vm_09.triage_vm09_coder_b_green` | ✅ |
| `dual_writer_pose_violation` | ✅ **false** |
| `infrastructure_view_isolation_green` | ✅ (witness refreshed) |

---

## Shift A — Observe (audit § slice 2)

From [`post_stage6_vm09_audit.md`](post_stage6_vm09_audit.md):

| Reader | File | Status |
|:---|:---|:---:|
| Camera visual / FX band | `view_representation.rs` | **Migrated** — CODER-B |
| Fire spark zoom (slice 1) | `gpu_particles.rs` | **Out of scope** this pass |
| Minimap click (PROJ-2) | `tile_world_fallback.rs` | ☑ **INFRA-PROJ2-001** |
| World Preview hover (PROJ-2) | `editor/world_preview/interaction.rs` | ☑ **INFRA-PROJ2-001** |

**Witness block:**

```json
"vm_09": {
  "triage_vm09_coder_b_green": true,
  "view_representation_world_main_zoom": "resolve_world_main_camera_scale"
}
```

---

## Shift B — Handoff **TRIAGE-VM-09-CODER-B** → `@coder`

**CODER-B status: DONE** — no further edits on `resolve_world_main_camera_scale` unless regression.

**Next coder lanes (slice 3+, not CODER-B):**

| ID | Goal | Files (≤3) | Verify |
|:---|:---|:---|:---|
| **INFRA-PROJ2-001** / **INFRA-PROJ2-CODER-B** | PROJ-2 plan | [`infra_proj2_sole_writer_plan_v1.md`](infra_proj2_sole_writer_plan_v1.md) | hit-test + `ViewManager` sole writer — **done** |
| **INFRA-VM10-001** | Lockstep hardening | `view_runtime` diagnostics | operator drift only |

**Do NOT (CODER-B guardrails):**

- Reintroduce parallel `desired.scale.x` as primary zoom in `CameraVisualState` / FX weighting
- Add a second WorldMain pose writer outside `view_runtime` bridge + `map_camera` input path

---

## Shift C — Act

```powershell
cargo test -p proc_A_dine01 --lib vm09_slice2
cargo test -p proc_A_dine01 --lib steward_vm09_infrastructure_witness_refresh
cargo test -p proc_A_dine01 --lib view_runtime
```

| Action | Result |
|:---|:---|
| Witness refresh | ✅ `steward_vm09_infrastructure_witness_refresh` |
| Audit row #2 | ✅ marked done in `post_stage6_vm09_audit.md` |

---

## Copy-paste — `@coder` (post CODER-B)

```
Track: INFRA-55 — post TRIAGE-VM-09-CODER-B
Read: src/dev/post_stage6_vm09_audit.md (PROJ-2 inventory)
      tools/orchestrator/agents/viewport_cleanup_agent.md
First: tile_world_fallback.rs minimap click → view_surface_screen_to_world(ViewId::Minimap, …)
Do NOT: touch resolve_world_main_camera_scale (slice 2 CLOSED)
Verify: cargo test -p proc_A_dine01 --lib view_runtime
Witness: debug_runs/infrastructure_view_isolation_live.json vm_09 + vm_a green
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.1.0 | 2026-05-25 | Slice 2+ only; **TRIAGE-VM-09-CODER-B** handoff to INFRA-PROJ2-001 |
| v1.0.0 | 2026-05-24 | Full steward pass (slice 1+2) |
