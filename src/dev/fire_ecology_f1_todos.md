# Fire ecology F1 — fuel + old-growth gate (active)

**Not** a `STAGE5_TODOS` row. Proves sim ignition policy, not render readiness flags.

**Witness:** `debug_runs/fire_ecology_live.json`  
**Code:** `src/systems/fire/chunk_fuel_profile.rs`, `combustion.rs`, `chunk_fire_overlay.rs`, `live_proof.rs`

---

## Done (F1)

- [x] **F1-01** `old_growth` drives canopy/brush fuel in `chunk_fuel_profile_from_vegetation`
- [x] **F1-02** `wildland_fuel_mass` + `fuel_ignition_gate` block ambient spark below mass threshold
- [x] **F1-03** Crown boost gated by `OLD_GROWTH_CROWN_THRESHOLD` (`crown_boost_for_old_growth`)
- [x] **F1-04** Overlay fuel seed `DEFAULT_CELL_FUEL_SEED` (not `1.0`); rain no longer refills fuel
- [x] **F1-05** Spread diffusion blocked when cell fuel `< MIN_CELL_FUEL_FOR_SPREAD`
- [x] **F1-06** `fire_ecology_live.json` writer + lib test

---

## Next (F2+ — triage, not F1)

> **Coder routing (2026-05-26):** **F2-01** / **F2-02** implement from **exec plans**, not this checklist alone — [`tools/orchestrator/queues/coder_active_queue.json`](../tools/orchestrator/queues/coder_active_queue.json) · [`plan_fire_f2_extract_exec_001_v1.md`](plan_fire_f2_extract_exec_001_v1.md) · [`plan_wss_smoke_bridge_exec_001_v1.md`](plan_wss_smoke_bridge_exec_001_v1.md)

- [ ] **F2-01** → **FIRE-F2-EXTRACT-001** — [`plan_fire_f2_extract_exec_001_v1.md`](plan_fire_f2_extract_exec_001_v1.md) (witness: `stage5_full_app_live.json`)
- [ ] **F2-02** → **WSS-SMOKE-BRIDGE-001** — [`plan_wss_smoke_bridge_exec_001_v1.md`](plan_wss_smoke_bridge_exec_001_v1.md)
- [ ] **F2-03** Fuel-linked spread (ember + neighbor fuel depletion)
- [ ] **F2-04** Align `fire_inst` readiness metric with sim heat stability in proof JSON

**Backlog:** [`stage5_triage_backlog.md`](stage5_triage_backlog.md) T3.
