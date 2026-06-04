# Fleet wave 5 coder dispatch `v1` — START HERE

| Field | Value |
|:---|:---|
| **Date** | 2026-05-27 |
| **Wave 4 partial sign-off** | [`fleet_signoff_wave4_coder_partial_20260527_v1.md`](fleet_signoff_wave4_coder_partial_20260527_v1.md) |
| **PR-4 exec** | [`plan_wss_pr4_exec_001_v1.md`](plan_wss_pr4_exec_001_v1.md) — **CLOSED** |
| **PR-5 criteria** | [`plan_wss_hybrid_retire_pr4_001_v1.md`](plan_wss_hybrid_retire_pr4_001_v1.md) § PR-5 |
| **H-A2 gate** | [`hanabi_spike_review_h_a2_gate_v1.md`](hanabi_spike_review_h_a2_gate_v1.md) |
| **Machine queue** | [`coder_active_queue.json`](../tools/orchestrator/queues/coder_active_queue.json) v4.8+ |

---

## Pick ONE primary per coder

### @coder A — **WSS-SLAB-PR-5** (primary)

| Field | Value |
|:---|:---|
| **Parent** | [`plan_wss_hybrid_retire_pr4_001_v1.md`](plan_wss_hybrid_retire_pr4_001_v1.md) |
| **Prereq** | PR-4 **CLOSED** — persist + overlay migrated on disk |
| **Exit** | `ecs_retire_fixture_green: true`; authority flags flip per plan after drift window |

```powershell
cargo test -p proc_A_dine01 --lib wss_substrate substrate
```

**Then:** **H-A2-001** — feature-flag Hanabi plugin (no default `EnginePlugin` merge)

**Fallback:** **INFRA-VM-FOLLOWON-001** — Phase C per [`post_stage6_active_todos.md`](post_stage6_active_todos.md)

**Mutex:** `src/substrate/*` · no `src/construction/*`

---

### @coder B — **BQ-128-APPLY-002** (primary)

| Field | Value |
|:---|:---|
| **Plan** | [`bq128_editor_path_plan_v1.md`](bq128_editor_path_plan_v1.md) Phase 2b |
| **UX** | Merge vs replace on blueprint import + confirm |
| **Exit** | construction/wave_s witness for import mode |

```powershell
cargo test -p proc_A_dine01 --lib construction blueprint_preset
```

**Optional:** **LOG-E01-FULLAPP-UPGRADE-001** — operator `--test visual` upgrade

**Mutex:** no `src/substrate/active_runtime*`

---

## Do not re-run

WSS-SLAB-PR-4 · INFRA-SLICE3-001 · IND-E02 play · BQ-128-APPLY-001

---

## Regression bundle

```powershell
cargo test -p proc_A_dine01 --lib wss_substrate construction industrial_activation coder_a_wave3 coder_b_wave3
cargo check -p hanabi_validation
```
