# BUILD-READ / HUD wired spine `v1` (recovery closed 2026-06-14)

```text
⟦SYMLANG⟧⟐v1  ◈WIRED
Authority: mod.rs wiring + witness JSON · queue J_REWIRE drained
Parent: PLAN-BUILD-READABILITY-001 · POST-DRAIN-PHASE-5-001 (drained)
Queue: tools/orchestrator/queues/post_drain_phase5_queue.json · lane J_REWIRE ★
Next product: POST-DRAIN-PHASE-6-001 — $ref:src/dev/coder_longrun_plan_phase6_v1.md
```

> **Filename note:** `build_read_unwired_spine_v1.md` kept for stable `$ref:` links. Content reflects **wired** state after J_REWIRE close (2026-06-11…13).

**Recovery context (2026-06-13):** Working-tree truncation temporarily removed modules from `mod.rs` while source remained on disk. Status docs correctly flagged split-brain; re-wire rows closed before Phase 6.

**Rule:** J_REWIRE is **drained** — do not re-pick REWIRE-* unless regression breaks `mod.rs`. Phase 6 picks from [`post_drain_phase6_coder_queue.json`](../tools/orchestrator/queues/post_drain_phase6_coder_queue.json).

---

## Wired inventory (J_REWIRE ★)

| Wire ID | Source | mod.rs | Witness / exit | Closed |
|:---|:---|:---:|:---|:---:|
| **BUILD-READ-REWIRE-003** | `src/gui/map_zoom_coherence.rs` | `gui/mod.rs` | `map_zoom_coherence_live.json` 🟢 | 2026-06-11 |
| **BUILD-READ-REWIRE-001** | `src/construction/placement_debug.rs` | `construction/mod.rs` | overlay + pick probe compiled | 2026-06-11 |
| **BUILD-READ-REWIRE-002** | `src/gui/hud/simulation_pointer_gate.rs` | `hud/mod.rs` | `sim_build_rail_submenu_block_rect` in ops shell | 2026-06-11 |
| **MINIMAP-REWIRE-001** | `minimap_bevy_interaction.rs` · `minimap_egui_dev.rs` | `hud/mod.rs` · `gui/mod.rs` | `design_minimap_widget_live.json` | 2026-06-11 |
| **BUILD-READ-REWIRE-004** | `pilot_catalog.rs` · `building_set.rs` · `site_stub_overlay.rs` | `construction/mod.rs` | `pilot_catalog_parity_live.json` | 2026-06-11 |
| **APS-QC-REWIRE-001** | `src/gui/assembly_snapshot_qc_ui.rs` | `gui/mod.rs` | `aps_bevy_qc_hud_001_live.json` | 2026-06-13 |
| **BUILD-READ-REWIRE-005** | `src/dev/*_live_proof.rs` | `dev/mod.rs` | key proofs promoted; some `#[cfg(test)]` tails remain | 2026-06-11 |

**Unblocked verify rows (also done):** `BUILD-READ-P0-002` · `BUILD-READ-P0-003` · `BUILD-READ-DEBUG-001` · `MINIMAP-WIDGET-IMPL-001` · `BUILD-READ-PILOT-001`

---

## Phase 6 tail (not re-wire — product hardening)

| Gap | Symptom | Phase 6 row |
|:---|:---|:---|
| Build visual **runtime** | lib green; operator pixel sign-off open | `BUILD-VERIFY-VISUAL-001` · `BUILD-READ-VISUAL-001` runtime |
| Warehouse-named witness strings | tile stamp / commit checklist labels | cosmetic — catalog authority landed |
| Live proofs `#[cfg(test)]` | some dev witnesses test-gated only | promote when refreshing witness JSON |
| Sim effects plugin | `telemetry.rs` · `player_event_log.rs` in `lib.rs` | plugin registration — separate from HUD spine |
| `utility_network` / `nav_agent` live proofs | on disk, not in `dev/mod.rs` | add when promoting witnesses |

---

## Verification gate (regression only)

```powershell
cargo check -p proc_A_dine01 --lib
cargo test -p proc_A_dine01 --lib map_zoom_coherence placement_debug simulation_pointer_gate
python -m rust_engine_mcp.cli validate-report cargo --cached --compress 4
```

```text
⟦/WIRED⟧  ΔWF→ post_drain_phase6_coder_queue.json · BUILD-READ-CONSUMER-MCP-001 · G-PLAY-01
```
