# BUILD-READ / HUD unwired spine `v1` (2026-06-13 recovery)

```text
⟦SYMLANG⟧⟐v1  ◈UNWIRED
Authority: on-disk .rs + mod.rs wiring · not queue "done" rows
Parent: PLAN-BUILD-READABILITY-001 · POST-DRAIN-PHASE-5-001
Queue: tools/orchestrator/queues/post_drain_phase5_queue.json (lane J_REWIRE)
```

**Context:** Working-tree truncation + partial integration left **full source files on disk** that were **removed from `mod.rs`** to restore `cargo check`. Designer sign-offs and some witness JSON are **green on disk** but **not compiled into the binary**.

**Rule:** Do not mark BUILD-READ-P0 / MINIMAP-WIDGET-IMPL **done** until the row's **Wire gate** passes (`mod.rs` + `cargo check --lib` + witness refresh).

---

## Unwired inventory

| Wire ID | Source file(s) | Plan / design | Was in `mod.rs` | Missing before re-wire | Blocks (queue) |
|:---|:---|:---|:---:|:---|:---|
| **BUILD-READ-REWIRE-001** | `src/construction/placement_debug.rs` | [`plan_operator_build_readability_exec_001_v1.md`](plan_operator_build_readability_exec_001_v1.md) · BUILD-READ-DEBUG-001 | yes → **removed** | `map_pick_closure_math_witness_green`, `egui_footprint_hotfix_a_witness_green`, gui sim-map projection exports, `BuildPlacementMode` | BUILD-READ-P0-003 verify · BUILD-READ-DEBUG-001 |
| **BUILD-READ-REWIRE-002** | `src/gui/hud/simulation_pointer_gate.rs` | [`design_pointer_hud_regions_v1.md`](design_pointer_hud_regions_v1.md) · BUILD-READ-P0-003 | yes → **removed** | `sim_build_rail_submenu_block_rect` in `simulation_shell_phase2.rs` | BUILD-READ-P0-003 verify |
| **MINIMAP-REWIRE-001** | `src/gui/hud/minimap_bevy_interaction.rs` · `src/gui/minimap_egui_dev.rs` | [`design_minimap_widget_v1.md`](design_minimap_widget_v1.md) · MINIMAP-WIDGET-IMPL-001 | hud only / gui missing | `MinimapEdge`, `MINIMAP_TITLE_BAR_H_PX`, `clamp_tactical_viewport_frame_rect` in `minimap_shell` / `minimap_viewport_frame` | MINIMAP-WIDGET-IMPL-001 · G-PLAY-01 minimap row |
| **BUILD-READ-REWIRE-003** | *(new)* `src/gui/map_zoom_coherence.rs` or restore witness fns | [`plan_map_zoom_smooth_exec_001_v1.md`](plan_map_zoom_smooth_exec_001_v1.md) · BUILD-READ-P0-002 | never wired | `map_zoom_coherence_001_witness_green/json` — live proof exists but callee missing | BUILD-READ-P0-002 · TRIAGE-MAP-ZOOM witness refresh |
| **APS-QC-REWIRE-001** | `src/gui/assembly_snapshot_qc_ui.rs` | [`design_aps_bevy_qc_hud_v1.md`](../docs/archive/2026-06-src-dev/plans/design_aps_bevy_qc_hud_v1.md) | never wired | `mod` + plugin in `gui/mod.rs` | grammar_continuation_queue APS-BEVY-QC-HUD-001 |
| **BUILD-READ-REWIRE-004** | `src/construction/pilot_catalog.rs` · `building_set.rs` · `site_stub_overlay.rs` | BUILD-READ-PILOT-001 · SHAPE-002 | **partial** (in `construction/mod.rs`) | parametric_commit / visual_authority still warehouse-hardcoded; live proofs `#[cfg(test)]` only | BUILD-READ-PILOT-001/002 |
| **BUILD-READ-REWIRE-005** | `src/dev/*_live_proof.rs` (map_zoom, build_read, minimap, fire play) | phase5 witnesses | **cfg(test) only** in `dev/mod.rs` | Re-wire after REWIRE-001…003 | Phase 5 witness refresh rows |

---

## On disk but OK (no re-wire needed for compile)

| File | Status |
|:---|:---|
| `src/sim/effects/telemetry.rs` · `player_event_log.rs` | Implemented · `pub mod sim` in `lib.rs` · plugin not registered |
| `src/systems/weather/player_read_hud.rs` | Implemented · wired in `weather/mod.rs` |
| `src/dev/utility_network_live_proof.rs` · `nav_agent_routing_live_proof.rs` · `weather_hud_player_read_proof.rs` | Implemented · **not** in `dev/mod.rs` (add when promoting witnesses) |

---

## Corrected status vs stale docs

| ⟨ID⟩ | Old claim | Correct status (2026-06-13) |
|:---|:---|:---|
| TRIAGE-MAP-ZOOM-SMOOTH-001 | Phase 4 **done** | Tile/zoom **code landed** · witness module **unwired** · JSON may be stale |
| BUILD-READ-P0-003 | **partial** — rail submenu wired | Pointer gate **on disk** · **not compiled** until REWIRE-002 |
| BUILD-READ-DEBUG-001 | open | Blocked on REWIRE-001 |
| MINIMAP-WIDGET-IMPL-001 | ready to implement | **Blocked** on MINIMAP-REWIRE-001 (shell API gaps) |
| BUILD-READ-PILOT-001 | ready | **In progress** — catalog on disk · not authoritative in commit path |

---

## Pick order (rewire before verify)

1. **BUILD-READ-REWIRE-003** — map zoom witness module (unblocks P0-002 verify)
2. **BUILD-READ-REWIRE-001** + **002** — placement debug + pointer gate (unblocks P0-003 + DEBUG-001)
3. **MINIMAP-REWIRE-001** — minimap shell API + hud/gui mods
4. **BUILD-READ-REWIRE-004** — pilot catalog authority in commit/witness path
5. **BUILD-READ-REWIRE-005** — promote live proofs from `#[cfg(test)]` to `dev/mod.rs`
6. **APS-QC-REWIRE-001** — egui QC panel (parallel · dev tooling)

---

## Verification gate (each rewire row)

```powershell
cargo check -p proc_A_dine01 --lib
cargo test -p proc_A_dine01 --lib <witness_filter>
# refresh witness JSON via live_proof refresh fns
```

```text
⟦/UNWIRED⟧  ΔWF→ post_drain_phase5_queue.json lane J_REWIRE
```
