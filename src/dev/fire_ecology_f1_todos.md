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

- [ ] **F2-01** Per-tile / hot-cell extract contract for GPU (`TRIAGE-FIRE-EXTRACT`)
- [ ] **F2-02** Smoke beyond stub (`chunk_smoke_field` + render)
- [ ] **F2-03** Fuel-linked spread (ember + neighbor fuel depletion)
- [ ] **F2-04** Align `fire_inst` readiness metric with sim heat stability in proof JSON

**Backlog:** [`stage5_triage_backlog.md`](stage5_triage_backlog.md) T3.
