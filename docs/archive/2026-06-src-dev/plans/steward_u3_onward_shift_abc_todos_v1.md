# Steward shift A→B→C todos — U3 onward preflight `v1`

| Field | Value |
|:---|:---|
| **Package ID** | **STEWARD-U3-ONWARD-ABC-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@sim-steward` |
| **When** | Before first **U3 Rust** step ([`u3_onward_execution_runbook`](../../.cursor/plans/u3_onward_execution_runbook_62ad3252.plan.md) → future `docs/archive/2026-06-prompts-guides/runbooks/guides/terrain_unification_runbook_v1.md`) |
| **Orchestrator** | [`orchestrator_signoff_snapshot_20260526_v1.md`](orchestrator_signoff_snapshot_20260526_v1.md) |
| **Witness index** | [`witness_status_live_v1.md`](witness_status_live_v1.md) · [`debug_runs/agent_debug_index.json`](../../debug_runs/agent_debug_index.json) |
| **Rule** | Run **A → B → C** in one session; **do not** start U3-S01 until **Shift B** is **GO** or **GO (qualified)** |

**Prereq:** Wave 3 + preflights **CLOSED** on `master` — do **not** re-run closed steward gates unless a bundle fails.

---

## Todo checklist

| ID | Shift | Action | Status | Owner |
|:---|:---|:---|:---:|:---|
| **U3-STEWARD-ABC-A** | **A** | Witness triage — JSON vs lib bundles; stale field table | ☐ | @sim-steward |
| **U3-STEWARD-ABC-B** | **B** | Authority check — viewport / extract single-writer (esp. if visual fails) | ☐ | @sim-steward |
| **U3-STEWARD-ABC-C** | **C** | Bounded fix **or** YAML route to `@coder` (≤3 files) | ☐ | @sim-steward |

---

## Shift A — Witness triage

**Goal:** Prove on-disk `debug_runs/*.json` matches lib bundle truth; flag **STALE** / **DRIFT** without reopening closed coder lanes.

### A — Read first

```
debug_runs/agent_debug_index.json
docs/archive/2026-06-src-dev/plans/witness_status_live_v1.md
docs/archive/2026-06-src-dev/plans/orchestrator_signoff_snapshot_20260526_v1.md
tools/orchestrator/queues/coder_active_queue.json
tools/orchestrator/queues/continuation_queue.json
```

### A — Lib bundles (run all; alt target if LNK1104)

```powershell
$env:CARGO_TARGET_DIR = "target\test-alt-steward"
Set-Location C:\dev\github\Rust_engine_template_01

cargo test -p proc_A_dine01 --lib stage5 -- --test-threads=1
cargo test -p proc_A_dine01 --lib steward_w3_gate_001 steward_s7b_preflight_001
cargo test -p proc_A_dine01 --lib coder_a_wave3 coder_b_wave3_bundle_001
cargo test -p proc_A_dine01 --lib steward_witness_sync_001_lib_bundle
```

### A — Compare matrix (fill in gate doc § Shift A table)

| Witness JSON | Lib bundle / test | Fields to check | Expected |
|:---|:---|:---|:---|
| `ui_shell_migration_live.json` | `steward_w3_gate_001` · `refresh_ui_p2a_001_live_witness` | `phase2b_closed`, `egui_pass_count_in_sim`, `phase4.icon_atlas_loaded` | 2B closed; egui 0; atlas **STALE** OK if lib green |
| `stage5_full_app_live.json` | `stage5` · `coder_b_wave3` | `readiness.passes`, `tactical_vfx_witness`, logistics rows | passes true |
| `infrastructure_view_isolation_live.json` | `coder_a_wave3` · `view_runtime` | `fire7_f7_a_exit_001.green`, `vm_08`, `dual_writer_pose_violation` | green; no dual writer |
| `minimap_compositor_live.json` | `minimap_compositor` · W3 gate | `composite_ok`, `ui_p3_m4_green` | composite_ok true |
| `stage7_behavioral_live.json` | `steward_s7b_preflight_001` | `s7b_preflight_green`, `s7b_m1/m2/m3_green`, `s7b_m4_play_green` | M1–3 true; M4 tail **qualified** |
| `industrial_activation_live.json` | `coder_b_wave3` | `ind_e02_green`, `s7p_grid_ux_001` | play path green |
| `construction_stage_live.json` | `coder_b_wave3` | `construction_mv_001.green` | green |
| `fire_streaming_live.json` | `coder_a_wave3` | `green`, `runtime_writer` | green |
| `wave_p_live.json` / `wave_c_live.json` | wave3 bundles | layout / depth greens | match bundle asserts |

### A — Known stale flags (do not reopen lanes)

| Field | Verdict if lib green |
|:---|:---|
| `phase4.icon_atlas_loaded: false` | **STALE** — refresh witness only |
| `phase2.minimap_gpu_path: false` | **QUALIFIED** — compositor JSON authoritative |
| `ui_p3_001.closed: false` | **QUALIFIED** — GPU compositor path |
| `s7b_m4_play_green: false` | **QUALIFIED** — optional sim tail |
| `continuation_queue.json` rows for **closed** slices (VM-06, FIRE-STREAM, MD-F2-*) | **DRIFT** — mark queue **STALE** in Shift C YAML; do not execute |

### A — Deliverable

- Update [`witness_status_live_v1.md`](witness_status_live_v1.md) **Date** + any changed values.
- Append § **Shift A triage** table to gate record (below) with **PASS / STALE / DRIFT** per row.
- If any row is **CONTRADICTS_LIB** → carry to Shift B as **BLOCK** candidate.

---

## Shift B — Authority check

**Goal:** Confirm viewport + fire/visual extract **single-writer** spine before terrain U3 touches `src/terrain/` or preview paths.

**Trigger (run full B even if A is green):**

- `cargo run -p proc_A_dine01 --release -- --test visual` fails or hangs
- `stage5_full_app_live.json` → `readiness.passes: false`
- `infrastructure_view_isolation_live.json` → `dual_writer_pose_violation: true`
- Any **CONTRADICTS_LIB** row from Shift A

### B — Static authority (read-only)

| Check | Authority path | Pass criterion |
|:---|:---|:---|
| Fire ECS scan | `extract_fire_simulation_snapshot` only | one `fn extract_fire_simulation_snapshot` |
| Per-view fire frames | `build_fire_visual_frames_by_view` | sole `ResMut<FireVisualFramesByView>` writer |
| Producer registry | `fire_visual_producer_count() == 1` | `stage5` test `single_fire_visual_producer_registered` |
| Minimap fire | `src/render/minimap_compositor/` | no `ChunkSurfaceFire` / `FireSimulationSnapshot` queries |
| VM-06 pose | `view_manager_sole_writer` | infra JSON `vm_06` aligned |
| VM-08 overlays | `overlay_masks_aligned` | infra `vm_08` + isolation block |

**Docs:** [`steward_fire7_preflight_gate_v1.md`](steward_fire7_preflight_gate_v1.md) · [`recovery_viewport.md`](recovery_viewport.md) · [`post_stage6_vm09_audit.md`](post_stage6_vm09_audit.md)

### B — If visual run fails

```powershell
$env:CARGO_TARGET_DIR = "target\test-alt-steward"
cargo test -p proc_A_dine01 --lib stage5 infrastructure_view_isolation view_runtime fire_view_extract -- --test-threads=1

# Optional operator (fresh timestamps):
# cargo run -p proc_A_dine01 --release -- --test visual
```

Capture: first panic line, `viewport_drift.json` / `full_render_diagnostic_*.json` if written.

| Failure class | Route in Shift C |
|:---|:---|
| Shader / GPU teardown | `@coder` render lane · `visual_run_blockers.md` |
| Viewport dual-writer | `@coder` infra · `viewport_cleanup_agent` playbook |
| Fire extract duplicate | `@coder` · `fire_view_extract.rs` (forbidden second extract) |
| Unrelated compile | **HALT** — fix compile before U3 |

### B — Deliverable

**Verdict:** `GO` | `GO (qualified)` | `BLOCK`

- **GO:** A green + authority table green + (visual pass **or** lib stage5/infra green with documented operator skip).
- **GO (qualified):** lib green; visual not run or stale JSON only.
- **BLOCK:** authority violation or CONTRADICTS_LIB — no U3 Rust until coder clears.

---

## Shift C — Bounded fix OR route

**Goal:** Close steward package; unblocks U3 markdown/Rust runbook execution.

### C — If fix is allowed

| Rule | Limit |
|:---|:---|
| Max files touched | **≤ 3** |
| Allowed | witness refresh writers, `witness_status_live_v1.md`, `continuation_queue.json` status fields, gate doc |
| Forbidden | U3 terrain/material Rust, second fire extract, minimap ECS fire query |

**Example bounded fixes:**

- Re-run `refresh_*_live_witness` from existing `*_proof.rs` (no new systems).
- Set `continuation_queue.json` stale rows to `"status": "superseded"` with note pointing to `coder_active_queue.json`.
- Bump `agent_debug_index.json` via any lib bundle that already refreshes index.

### C — If fix is NOT allowed → YAML route

```yaml
shift: C
issue:
  id: STEWARD-U3-ONWARD-ABC-001
  severity: HIGH | MED | LOW
shift_a_verdict: PASS | STALE | DRIFT | CONTRADICTS_LIB
shift_b_verdict: GO | GO_qualified | BLOCK
route:
  steward: close package with qualified notes only
  delegate:
    - track: <lane>
      agent: "@coder"
      budget: "≤3 files per PR"
      read:
        - docs/archive/2026-06-src-dev/plans/orchestrator_signoff_snapshot_20260526_v1.md
        - <failing witness path>
      verify: "cargo test -p proc_A_dine01 --lib <bundle>"
      do_not:
        - second global fire extract
        - U3 terrain until steward ABC GO
  u3_onward:
    blocked_until: shift_b_verdict == GO
    runbook_plan: "C:/Users/oz_/.cursor/plans/u3_onward_execution_runbook_62ad3252.plan.md"
    rust_entry: "docs/archive/2026-06-prompts-guides/runbooks/guides/terrain_unification_runbook_v1.md (when landed)"
    first_step: "U3-S01 — after ABC GO only"
block: <none | U3-S01..S08 until coder clears>
```

### C — Sign-off record

Create or update: `docs/archive/2026-06-src-dev/plans/steward_u3_onward_shift_abc_gate_v1.md` with **PASS / BLOCK** and link from [`stage_steward_todos_v1.md`](stage_steward_todos_v1.md).

---

## Copy-paste — full package

### U3-STEWARD-ABC-A

```
Lane: STEWARD-U3-ONWARD-ABC-001-A
Agent: @sim-steward
Read: debug_runs/agent_debug_index.json, witness_status_live_v1.md, orchestrator_signoff_snapshot_20260526_v1.md
Act: run lib bundles; compare JSON vs bundle; flag icon_atlas_loaded, continuation_queue stale slices
Deliver: witness_status_live_v1.md updated; triage table STALE/DRIFT/PASS
Do NOT: reopen closed F7/S7B/W3 steward gates; start U3 Rust
```

### U3-STEWARD-ABC-B

```
Lane: STEWARD-U3-ONWARD-ABC-001-B
Agent: @sim-steward
Read: steward_fire7_preflight_gate_v1.md, recovery_viewport.md, infrastructure_view_isolation_live.json
Act: single-writer audit; if visual failed run stage5 + view_runtime + infra tests
Deliver: GO | GO (qualified) | BLOCK
Do NOT: edit >3 files in Shift B (observe only)
```

### U3-STEWARD-ABC-C

```
Lane: STEWARD-U3-ONWARD-ABC-001-C
Agent: @sim-steward
Prereq: Shift B GO or GO (qualified)
Act: bounded witness/queue doc fix (≤3 files) OR YAML route to @coder
Deliver: steward_u3_onward_shift_abc_gate_v1.md PASS/BLOCK
Unblocks: U3 onward runbook Rust (U3-S01+) per .cursor/plans/u3_onward_execution_runbook_62ad3252.plan.md
Do NOT: implement terrain/material systems in steward shift
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | **STEWARD-U3-ONWARD-ABC-001** A/B/C todos for witness + authority pre-U3 |
