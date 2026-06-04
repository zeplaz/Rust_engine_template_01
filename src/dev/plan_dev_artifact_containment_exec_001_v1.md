# PLAN-DEV-ARTIFACT-CONTAINMENT-EXEC-001 — Runtime witness containment `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-DEV-ARTIFACT-CONTAINMENT-EXEC-001** |
| **Policy** | [`dev_artifact_containment_policy_v1.md`](dev_artifact_containment_policy_v1.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-28 |
| **Owner** | `@planner` → **`@coder`** (dev lane + domain collectors) |
| **Status** | **SIGNED** — Slice B–C **PARTIAL**; execute Slice D tail |

**Board rows:** `DEV-CONTAIN-001` (Phase 1) · `DEV-CONTAIN-002` (Phase 2) · `DEV-CONTAIN-003` (Phase 3 enforce)

---

## 1. Scope + non-goals

### Scope

- Centralize runtime witness **file I/O** under `src/dev/runtime_witness/`.
- Keep domain folders as **read-only collectors** (`build_*_payload`, `*_witness_fields`).
- Gate all writes via `RUNTIME_WITNESS_WRITES` / `RUNTIME_WITNESS_WRITES_FORCE_OFF` ([`runtime_witness/gate.rs`](runtime_witness/gate.rs)).
- CI containment scan: [`tools/orchestrator/scripts/check_live_proof_containment.ps1`](../../tools/orchestrator/scripts/check_live_proof_containment.ps1).

### Non-goals

- Deleting witness JSON capability or changing acceptance predicates.
- Moving domain snapshot structs out of render/construction/economy trees.
- Blocking release builds when witness writes are off (writes skip; sim continues).

---

## 2. Current-state inventory

| Lane | Domain collector (stay) | Writer today | Target writer | Migration |
|:---|:---|:---|:---|:---:|
| Wave C streaming | `src/io/streaming/` | shim → `wave_c.rs` | `runtime_witness/wave_c.rs` | **DONE** |
| Wave S hydrate | `src/io/save/` | shim → `wave_s.rs` | `runtime_witness/wave_s.rs` | **DONE** |
| Stage 6 virtualization | `src/render/stage6_*` | shim → `stage6.rs` | `runtime_witness/stage6.rs` | **DONE** |
| View runtime isolation | `src/render/view_runtime/` | shim → `view_runtime.rs` | `runtime_witness/view_runtime.rs` | **DONE** |
| Minimap compositor | `minimap_compositor/live_proof.rs` | domain + fs | `runtime_witness/minimap.rs` | OPEN |
| Construction | `construction/live_proof.rs` | domain + fs | `runtime_witness/construction.rs` | OPEN |
| Industrial activation | `economy/activation/live_proof.rs` | domain + fs | `runtime_witness/industrial.rs` | OPEN |
| Logistics throughput | `economy/logistics/live_proof.rs` | domain + fs | `runtime_witness/logistics.rs` | OPEN |
| Fire system | `systems/fire/live_proof.rs` | domain + fs | `runtime_witness/fire.rs` | OPEN |
| Wave P preview | `gui/editor/world_preview/wave_p_live_proof.rs` | domain + fs | `runtime_witness/wave_p.rs` | OPEN |
| Stage 7 behavioral | `dev/stage7_behavioral_live_proof.rs` | dev + fs | `runtime_witness/stage7_behavioral.rs` | OPEN |
| Stage 7 play | `dev/stage7_play_live_proof.rs` | dev + fs | `runtime_witness/stage7_play.rs` | OPEN |
| WSS substrate | `substrate/` collectors | mixed | `runtime_witness/wss_substrate.rs` | OPEN |

**Scaffold landed:** [`runtime_witness/mod.rs`](runtime_witness/mod.rs), `gate.rs`, `io.rs`, `common.rs`, [`exceptions_manifest.json`](runtime_witness/exceptions_manifest.json).

---

## 3. Target architecture

### Authority rules

| Rule | Enforcement |
|:---|:---|
| **No new out-of-root `*live_proof*.rs`** | CI script + PR checklist |
| **Domain = collectors only** | No `std::fs::write` in `src/render/`, `src/construction/`, etc. |
| **Writer = runtime_witness** | `write_enveloped_witness` + envelope wrap |
| **Release safe** | `witness_writes_enabled()` false unless env / debug default |
| **Schema compatibility** | Witness keys unchanged unless versioned in PR |

### Schedule / call pattern

```text
Domain system (Update/PostUpdate)
  → updates in-memory proof state / collector inputs
runtime_witness::<lane>::write_*_system
  → gate check → build payload from domain → envelope → debug_runs/*.json
```

Temporary shims re-export `write_*` from old paths until Slice D retires them.

---

## 4. Phased migration

### Phase 0 — Freeze (DONE)

| Task | Exit |
|:---|:---|
| Policy doc published | [`dev_artifact_containment_policy_v1.md`](dev_artifact_containment_policy_v1.md) |
| No new out-of-root writers | PR checklist active |
| CI warning script | `check_live_proof_containment.ps1` |

### Phase 1 — Scaffold + gate (PARTIAL)

| Task | Status | Exit |
|:---|:---:|:---|
| **P1-A** `runtime_witness/` tree + gate/io | **DONE** | `cargo check` green |
| **P1-B** wave_c + wave_s writers | **DONE** | shims delegate; parity tests pass |
| **P1-C** stage6 + view_runtime writers | **DONE** | `stage6_virtualization_live.json` / `infrastructure_view_isolation_live.json` refresh |
| **P1-D** Wire gate on all **new** writers | **DONE** | `RUNTIME_WITNESS_WRITES_FORCE_OFF=1` skips file |

### Phase 2 — Extract remaining lanes (OPEN)

| Task | Exit |
|:---|:---|
| **P2-A** minimap compositor writer | `minimap_compositor_live.json` unchanged keys |
| **P2-B** construction + industrial + logistics | `construction_stage_live.json`, `industrial_activation_live.json`, `logistics_throughput_live.json` |
| **P2-C** fire + wave_p | `fire_ecology_live.json`, `wave_p_live.json` |
| **P2-D** stage7 behavioral + play | `stage7_behavioral_live.json`, `stage7_play_live.json` |
| **P2-E** wss_substrate | `wss_substrate_live.json` single writer path |

Each lane: split `build_*_payload` (stay local) + `commit_*` (move to runtime_witness).

### Phase 3 — Enforce (OPEN)

| Task | Exit |
|:---|:---|
| **P3-A** Witness parity diff 60s after each lane migrate | byte-stable or documented delta |
| **P3-B** CI `-HardFail` on containment script | `.github/workflows/ci.yml` or `tools/orchestrator/ci/run.ps1` |
| **P3-C** Retire wave/stage6/view_runtime shims | remove from `exceptions_manifest.json` |
| **P3-D** Remove remaining shims | zero out-of-root `*live_proof*.rs` |

---

## 5. PR-sized execution slices

Parallelizable by lane after **Slice 1** (gate parity).

| Slice | Phase | Files | Tests | Witness |
|:---:|:---|:---|:---|:---|
| **0** | P1 | *(landed)* `runtime_witness/{mod,gate,io,common,wave_c,wave_s,stage6,view_runtime}.rs` | `cargo check` | wave_c, wave_s, stage6, view_runtime |
| **1** | P2-A | `runtime_witness/minimap.rs`, `minimap_compositor/live_proof.rs` (shim) | `cargo test -p proc_A_dine01 --lib minimap_compositor` | `minimap_compositor_live.json` |
| **2** | P2-B | `runtime_witness/construction.rs`, `construction/live_proof.rs` | `cargo test -p proc_A_dine01 --lib construction` | `construction_stage_live.json` |
| **3** | P2-B | `runtime_witness/industrial.rs`, `runtime_witness/logistics.rs` | industrial + logistics lib tests | `industrial_activation_live.json`, `logistics_throughput_live.json` |
| **4** | P2-C | `runtime_witness/fire.rs`, `runtime_witness/wave_p.rs` | fire + wave_p tests | `fire_ecology_live.json`, `wave_p_live.json` |
| **5** | P2-D | `runtime_witness/stage7_behavioral.rs`, `runtime_witness/stage7_play.rs` | `stage7_behavioral`, `stage7_play` | behavioral + play JSON |
| **6** | P2-E | `runtime_witness/wss_substrate.rs` | `wss_substrate` lib | `wss_substrate_live.json` |
| **7** | P3 | CI `-HardFail`, shim removal, manifest trim | containment script exit 0 | all `*_live.json` refresh |

**Do not** migrate perf-critical paths in the same PR as witness writer moves — one lane per PR.

---

## 6. Verification + witness matrix

### Commands

```powershell
# Containment scan (warning mode today)
.\tools\orchestrator\scripts\check_live_proof_containment.ps1

# Hard fail (after Slice 7)
.\tools\orchestrator\scripts\check_live_proof_containment.ps1 -HardFail

# Gate off smoke
$env:RUNTIME_WITNESS_WRITES_FORCE_OFF = "1"
cargo test -p proc_A_dine01 --lib stage5 wss_substrate construction
Remove-Item Env:RUNTIME_WITNESS_WRITES_FORCE_OFF

# Lane regression bundle
cargo test -p proc_A_dine01 --lib stage5 wss_substrate construction stage7 minimap_compositor
```

### Witness keys (must stay green after each slice)

| File | Keys |
|:---|:---|
| `stage6_virtualization_live.json` | `infra_slice3_001`, virtualization rollup |
| `infrastructure_view_isolation_live.json` | view isolation fields |
| `wave_c_live.json` / `wave_s_hydrate_live.json` | wave spine keys |
| `minimap_compositor_live.json` | `composite_ok`, `presentation_source` |
| `stage7_behavioral_live.json` | `s7b_steward_green`, `s7b_m4_play_001` (preserve on migrate) |

### Parity protocol (Slice 7 prep)

1. Copy `debug_runs/<lane>_live.json` before migrate.
2. Run lib refresh test after migrate.
3. Diff JSON keys (ignore `_agent_meta.written_at_epoch_secs`).

---

## 7. Observability

- **Daily:** containment script in CI warning mode — zero new violations.
- **Per slice:** refresh affected `debug_runs/*_live.json` + `agent_debug_index.json`.
- **Gate debug:** log once per session when writes skipped (`witness_gate_snapshot`).

---

## 8. Risks and mitigations

| Risk | Phase | Mitigation | Rollback |
|:---|:---|:---|:---|
| Witness key drift on move | P2 | Shim re-export + parity diff | Revert slice; keep shim |
| Schedule ordering break | P2 | Register writer systems same set/order as before | Restore domain writer |
| CI false positive | P3 | `exceptions_manifest.json` timed shims | Narrow manifest |
| Release depends on witness I/O | All | Gate off path tested each slice | N/A — policy violation |
| Dual writers same JSON | P2 | Single `commit_*` in runtime_witness | Delete duplicate write |

---

## 9. Definition of done

- [ ] All lanes in §2 migration matrix **DONE** (no domain `std::fs` witness writes).
- [ ] `check_live_proof_containment.ps1 -HardFail` passes in CI.
- [ ] `exceptions_manifest.json` `allowed_shim_paths` empty (or dev-only stubs documented).
- [ ] `RUNTIME_WITNESS_WRITES_FORCE_OFF=1` — lib tests green, no file writes.
- [ ] Policy doc migration table updated to **DONE** per lane.

---

## 10. Start Here (@coder — next 48 h)

| Priority | Owner | Action |
|:---:|:---|:---|
| 1 | Coder A | **Slice 1** — minimap compositor writer extract (smallest render lane) |
| 2 | Coder A | Parity diff on migrated wave_c/wave_s/stage6/view_runtime JSON |
| 3 | Coder B | **Slice 5** — stage7 writers (after M4 play remedy — avoid witness churn) |
| 4 | Coder A | **Slice 6** — wss_substrate writer + post-spine sub-block fix |
| 5 | Operator | Refresh `agent_debug_index.json` after each slice |

**Parallel with perf lane:** containment Slice 1–2 does not block `PERF-VIS-002`; avoid same PR as `tile_world_fallback.rs`.

---

## Ownership map

| Role | Responsibility |
|:---|:---|
| **Planner** | This exec plan, lane matrix, CI gate timing |
| **Coder (dev)** | `runtime_witness/*`, gate, CI script, shim retirement |
| **Coder (domain)** | Collector splits in render/construction/economy/strategic |
| **Operator** | Witness parity diff, index refresh |
| **Designer** | — |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-28 | PLAN-DEV-ARTIFACT-CONTAINMENT-EXEC-001 — Slice B–C partial landed |
