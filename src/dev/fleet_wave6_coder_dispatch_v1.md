# Fleet wave 6 coder dispatch `v1` — START HERE

| Field | Value |
|:---|:---|
| **Maturity routing** | [`fleet_maturity_signoff_routing_20260527_v1.md`](fleet_maturity_signoff_routing_20260527_v1.md) |
| **Audit** | [`planner_status_audit_v14.md`](planner_status_audit_v14.md) |

---

## Wave 5 closed (witness-backed)

**A:** WSS-SLAB-PR-5 fixture (`ecs_retire_fixture_green`, weather/fire authority false)  
**B:** BQ-128-APPLY-002 (`construction_bq128_apply_merge_replace_002.green`)

---

## @coder A — pick one primary

### **WSS-PR5-SMOKE-PROD-001** (recommended)

| Field | Value |
|:---|:---|
| **Goal** | `hybrid_ecs_smoke_authoritative: false` in **live** `wss_substrate_live.json` (not fixture-only) |
| **Plan** | [`plan_wss_pr5_smoke_prod_001_v1.md`](plan_wss_pr5_smoke_prod_001_v1.md) |
| **Criteria** | [`plan_wss_hybrid_retire_pr4_001_v1.md`](plan_wss_hybrid_retire_pr4_001_v1.md) § ChunkSmokeField |
| **Designer** | **DESIGN-PR4-RETIRE-UX-001** when copy needed |

### **H-A2-001** (after planner exec)

| Field | Value |
|:---|:---|
| **Plan** | [`plan_hanabi_h_a2_exec_001_v1.md`](plan_hanabi_h_a2_exec_001_v1.md) |
| **Gate** | [`hanabi_spike_review_h_a2_gate_v1.md`](hanabi_spike_review_h_a2_gate_v1.md) |
| **Rule** | `hanabi_l3` feature only — **no** default `EnginePlugin` Hanabi |

```powershell
cargo test -p proc_A_dine01 --lib wss_substrate substrate
cargo check -p hanabi_validation
```

---

## @coder B — pick one primary

### **S7B-M3-STEWARD-REMEDY-001** (recommended)

| Field | Value |
|:---|:---|
| **Disk** | `s7b_m3_green: false`, `s7b_steward_green: false` |
| **Spec** | [`stage7_behavioral_live_witness_spec_v1.md`](stage7_behavioral_live_witness_spec_v1.md) |
| **Code** | `stage7_behavioral_live_proof.rs` |

```powershell
cargo test -p proc_A_dine01 --lib stage7
```

### **LOG-E01-FULLAPP-UPGRADE-001** (with @operator)

Optional after M3 green — `--test visual` refresh.

---

## Do not re-run

PR-4 · PR-5 fixture · BQ-128-001/002 · INFRA-SLICE3 · IND-E02 · parametric/R4 archives
