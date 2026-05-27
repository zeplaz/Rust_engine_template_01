# PLAN-OPERATOR-VISUAL-BUNDLE-001 — operator visual sign-off bundle `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-OPERATOR-VISUAL-BUNDLE-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** |
| **Matrix** | [`visual_run_acceptance_matrix_v1.md`](visual_run_acceptance_matrix_v1.md) |
| **Designer** | VFX + WP ACCEPT records (qualified) |

**No Rust.** One operator session closes three qualified coder tails.

---

## Bundled lanes

| Lane | Qualified today? | Upgraded by visual run |
|:---|:---:|:---|
| **LOG-E01-VISUAL-CONFIRM-001** | lib + `logistics_active_rows: 1` on disk | Same-session timestamp + inv 720+ |
| **VFX-VISUAL-SIGNOFF-001** | lib `vfx_visual_signoff_001.green` | `visual_run_pending: false` |
| **UI-WP-VISUAL-001** | lib `wave_p_live.json` layout greens | Pixel audit optional |

---

## Single command

```powershell
cargo run -p proc_A_dine01 --release -- --test visual
```

**Prereqs:** VR-01/07/08 fixed; tactical zoom if checking `fire_spark_rows` (see [`vfx_coder_phase2_queue_v1.md`](vfx_coder_phase2_queue_v1.md)).

---

## PASS gate (bundle)

| # | Criterion | File |
|:---:|:---|:---|
| B1 | Exit code 0 | terminal |
| B2 | `readiness.passes: true` | `stage5_full_app_live.json` |
| B3 | `projection_graph.logistics_active_rows > 0` | same |
| B4 | `vfx_visual_signoff_001.green: true` | same |
| B5 | No shader panic through inv 720+ | log |
| B6 | Optional: VR-10 teardown clean | exit |

---

## After run

```powershell
cargo test -p proc_A_dine01 --lib coder_a_wave3_closure coder_b_wave3_bundle_001
```

Update [`coder_active_queue.json`](../../tools/orchestrator/queues/coder_active_queue.json): move **LOG-E01-VISUAL-CONFIRM-001** to **done** when B3–B5 pass.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | **PLAN-OPERATOR-VISUAL-BUNDLE-001** |
