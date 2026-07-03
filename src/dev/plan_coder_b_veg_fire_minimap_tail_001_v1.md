# PLAN-CODER-B-VEG-FIRE-MINIMAP-TAIL-001 — leftover product integration `v1`

```text
⟦SYMLANG⟧⟐v1  ◈PLAN
⟨ID⟩ PLAN-CODER-B-VEG-FIRE-MINIMAP-TAIL-001
Date: 2026-06-17
Status: **CLOSED** (@coder B · witness bundle green · `post_drain_phase6_b_tail_queue.json` seq 1–9 done)
Owner: @planner → **@coder B**
Prior: POST-DRAIN-PHASE-6-001 (32/32 done) · parallel_wave wave-0 CDR-B done
Honest: $ref:src/dev/vegetation_system_honest_status_v1.md
Burn spine: $ref:src/dev/plan_veg_burn_extract_001_v1.md
Authority: $ref:.cursor/skills/bevy-simulation-grade/07-repo-authority-map.md
Hardening: $ref:src/dev/coder_queue_hardening_rules_v1.md
```

**Situation:** Phase 6 + wave-0 CDR rows are **marked done**, but veg/fire/minimap **product wiring** is still thin — lib witnesses green, compositor/HUD/catalog consumer paths incomplete.

**Rule:** @coder A owns sim burn overlay + extract build + FULL_APP ecology rows. **@coder B** owns **GUI/minimap/compositor consumers**, **catalog loader**, **map stamp runtime**, **Stage 7 minimap readers**.

---

## Done (do not re-pick)

| ID | Owner | Truth |
|:---|:---|:---|
| VEG-BURN-OVERLAY-001..003 | A | `landscape_grammar_burn.rs` + witnesses green |
| VEG-BURN-EXTRACT-004..007 | A/B split | `vegetation_visual_extract.rs` + chain witness |
| VEG-MINIMAP-OVERLAY-002 | B | **Partial** — tint modulator witness only; **not** HUD legend UI |
| CDR-B-VEG-RESOLVER-PARITY-001 | B | doc + byte parity |
| CDR-B-TILE-RESOLVER-VEG-001 | B | `resolve_landscape_tile_from_extract_key` |
| CDR-B-MAP-STAMP-CONTRACT-001 | B | lib contract test |
| UI-P3-M3-001 / M4-001 | B | compositor M2/M3 channels lib-green |

---

## Leftover queue (coder B — pick top-down)

### Wave 1 — P0 (one primary per session, ≤3 files)

| Seq | ID | Goal | Territory | Witness exit |
|:---:|:---|:---|:---|:---|
| **1** | **VEG-CATALOG-LOADER-001** | Load `assets/configs/landscape/_vegetation_variant_catalog.ron` as `VegetationVariantCatalog` Resource (mirror `VariantCatalog` pattern) | `src/systems/ecology/` or `src/construction/procedural/` | `vegetation_variant_catalog_load_live.json` · all 13 `veg_*` parity keys present |
| **2** | **VEG-MINIMAP-BURN-MERGE-001** | Minimap ecology pass reads **`VegetationExtractFrame`**; when `burn_active` + `veg_burn_*`, **override** topology tint (Q4a signed) | `src/render/minimap_compositor/pass.rs` | `minimap_compositor_live.json` → `veg_burn_rows >= 1` · `burn_overrides_topology: true` |
| **3** | **FIRE-MINIMAP-COHERENCE-001** | Same sim tick: fire heat overlay revision aligns with veg extract revision; no double-wash when ecology on + fire heat off (default sim policy) | `overlay_field_buffers.rs` · compositor | `minimap_fire_veg_coherence_live.json` · `revision_aligned: true` |

### Wave 2 — P1

| Seq | ID | Goal | Territory | Witness exit |
|:---:|:---|:---|:---|:---|
| **4** | **VEG-MINIMAP-LEGEND-UI-001** | Collapsible topology legend strip per **DES-MINIMAP-VEG-LEGEND-001** (not tint-only lib test) | `src/gui/hud/minimap_bevy_interaction.rs` · `assets/ui/minimap/` | `minimap_topology_legend_live.json` → `legend_ui_wired: true` |
| **5** | **VEG-LG5-STAMP-RUNTIME-001** | Runtime map stamp: `veg_topo_*` / catalog `topology_*` → atlas UV when `_landscape_atlas_index` row exists | extend stamp path used in `landscape_map_stamp_contract_live_proof.rs` | `landscape_map_stamp_runtime_live.json` · `stamped_chunks >= 1` |
| **6** | **VEG-CATALOG-RESOLVE-001** | `resolve_vegetation_variant(catalog, extract_row)` — clamp to catalog entries (PT-4 parallel) | beside `variant_key_for_burn_row` consumer | extends `veg_resolver_parity_live.json` · `catalog_clamp_green: true` |

### Wave 3 — P2 (minimap + Stage 7)

| Seq | ID | Goal | Territory | Witness exit |
|:---:|:---|:---|:---|:---|
| **7** | **S7B-M3-MINIMAP-001** | Wire recon + logistics stress overlay **readers** on minimap (D-S7-02) — read compositor snapshots only | `src/gui/hud/stage7_ui_shell.rs` · compositor | `stage7_behavioral_live.json` → `m3_minimap_readers_wired: true` |
| **8** | **MINIMAP-ECO-FIRE-TRAY-001** | Overlay tray copy: separate **Fire heat** vs **Ecology / burn scar** toggles; defaults match `simulation_minimap_overlay_defaults` | `src/gui/hud/dock_shell.rs` | `minimap_compositor_live.json` tray fields |
| **9** | **VEG-DIAG-EXTRACT-PANEL-001** | Diagnostics read-only panel: sample `VegetationExtractFrame` rows (variant_key, glyph, burn frame) | `src/gui/diagnostics_ui.rs` | `landscape_grammar_extract_live.json` linked in diag witness |

---

## Explicitly NOT coder B (route @coder A)

| ID | Why |
|:---|:---|
| VEG-BURN-OVERLAY / SM / SUCCESSION | Sim writers — `landscape_grammar_burn.rs` |
| CDR-A-VISUAL-SMOKE-ECO-001 | `--test visual` ecology raster |
| FIRE7-F7-A-EXIT-001 | Fire extract product gate — render spine |
| CDR-A-EXTRACT-SPRITE-001 | Real LG-5 sprite instances in extract |
| VEG-F02-MCP-ATLAS-001 | MCP bake — @coder-mcp |

---

## Authority (07 map — consumer side)

```text
VegetationExtractFrameSet::BuildProfiles   (A builds — B reads only)
FireVisualFrameSet::BuildProfiles          (A — unchanged)
MinimapCompositor pass                     (B — reads EcologyVisualSnapshot + VegetationExtractFrame)
SharedOverlayFieldBuffers                  (fire heat — single writer from FireSimulationSnapshot)
```

⛔ minimap pass mutating SuccessionState / ActiveBurn  
⛔ second ECS fire scan in compositor  
⛔ egui-owned long-term minimap raster

---

## Regression

```powershell
cargo test -p proc_A_dine01 --lib landscape_grammar minimap_compositor vegetation_visual_extract
python -m rust_engine_mcp.cli validate-report cargo --compress 3
```

---

## Machine queue hook

Append rows **1–9** to `tools/orchestrator/queues/post_drain_phase6_coder_queue.json` **tail** or new `post_drain_phase6_b_tail_queue.json` when first slice lands.

```text
⟦/PLAN-CODER-B-VEG-FIRE-MINIMAP-TAIL-001⟧  ΔWF→@coder B seq 1 VEG-CATALOG-LOADER-001
```
