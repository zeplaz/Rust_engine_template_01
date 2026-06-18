# Sim HUD copy registry `v1` — locked strings

| Field | Value |
|:---|:---|
| **ID** | **DES-SIM-HUD-COPY-REGISTRY-001** |
| **Program** | PLAN-SIM-HUD-PROFESSIONAL-POLISH-001 · Track 5 |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Authority** | Merges [`design_build_ux_redesign_v1.md`](design_build_ux_redesign_v1.md) · [`design_build_toolbox_hud_v1.md`](design_build_toolbox_hud_v1.md) |
| **Verdict** | **PASS** |

```text
DES-SIM-HUD-COPY-REGISTRY-001 Q✓
Single source — context strip, picker, tray, toast — no drift
```

**Rule:** All sim HUD build strings **must** reference keys below. Coder: `sim_hud_copy.rs` or const module — no inline duplicates in `contextual_tip.rs` / `industrial_menu.rs` after migrate.

---

## 1. Context strip (`contextual_tip`)

| Key | Template |
|:---|:---|
| `strip.build.preview` | `BUILD · {category} · {archetype} · click map to lock · [{cycle}] category` |
| `strip.build.adjust.valid` | `BUILD · {archetype} · locked {x},{z} · Ctrl rotate · Shift scale · click to place · Esc cancel` |
| `strip.build.adjust.invalid` | `BUILD · locked {x},{z} · blocked: {reason} · Esc cancel` |
| `strip.build.adjust.site` | append ` · site overlay on` when site stub active |
| `strip.build.idle` | *(non-build strings unchanged)* |
| `strip.prefix` | `CONTEXT — ` *(optional dev prefix — omit in shipping sim if redundant)* |

**Removed from building path:** `Enter to commit` primary · `Shift+click queue`.

---

## 2. Build picker sheet

| Key | String |
|:---|:---|
| `picker.title.zone` | `Zone` |
| `picker.title.roads` | `Roads` |
| `picker.title.industry` | `Industry` |
| `picker.title.utilities` | `Utilities` |
| `picker.title.shapes` | `Shapes` |
| `picker.industry.lead` | `Place each step separately — power adds on the grid.` |
| `picker.industry.other` | `Other industry` |
| `picker.generic.factory` | `Generic factory` |
| `picker.generic.depot` | `Generic depot` |
| `picker.empty_category` | `○ No tools in this category` |
| `picker.loading` | `⟳ Loading build catalog…` |
| `picker.error_catalog` | `✗ Catalog unavailable` |
| `picker.close` | `Close` |

### Chain display names

| Key | String |
|:---|:---|
| `chain.concrete_portland` | `Concrete (Portland)` |
| `chain.concrete_geopolymer` | `Concrete (Geopolymer)` |
| `chain.aluminum_primary` | `Aluminum primary` |

### Supply-chain role captions (picker card footer)

| Key | String |
|:---|:---|
| `role.aggregate_mine` | `quarry` |
| `role.cement_kiln` | `kiln` |
| `role.concrete_mixer` | `batching` |
| `role.integrated_plant` | `legacy monolith` |
| `role.bauxite_mine` | `mine` |
| `role.alumina_refinery` | `refinery` |
| `role.aluminum_smelter` | `smelter` |
| `role.aluminum_fabrication` | `fabrication` |

### Power tier HUD compact (picker card)

| Key | Pattern |
|:---|:---|
| `power.light` | `⚡ light` |
| `power.medium` | `⚡ medium` |
| `power.heavy` | `⚡ heavy` |
| `power.grid` | `⊞ grid` |

---

## 3. Build rail & shapes

| Key | String |
|:---|:---|
| `rail.pilot.rail_warehouse` | `Rail Warehouse (pilot)` |
| `rail.tooltip.pilot` | `L footprint · rotate QA · 11 tiles · site stub 10×8` |
| `rail.hint.preview` | `Click map to lock placement` |
| `rail.hint.adjust` | `Ctrl rotate · Shift scale · click again to place` |
| `rail.hint.invalid` | `Blocked — {reason}` |
| `rail.footnote.enter` | `Enter — place (optional)` |

---

## 4. Context tray — Build tab

| Key | String |
|:---|:---|
| `tray.build.tab` | `Build` |
| `tray.build.legend.title` | `Site stub` |
| `tray.build.legend.footprint` | `Green — building footprint` |
| `tray.build.legend.yard` | `Dashed — yard / rail / park` |
| `tray.build.legend.label.yard` | `Yard` |
| `tray.build.legend.label.rail` | `Rail` |
| `tray.build.legend.label.svc` | `Svc` |
| `tray.build.legend.label.park` | `Park` |
| `tray.build.legend.label.load` | `Load` |
| `tray.build.staging.title` | `Staged placement` |
| `tray.build.staging.empty` | `○ No staged placements` |
| `tray.build.queue.title` | `Pending queue` |
| `tray.build.queue.summary` | `{n} pending · {first_label}` |
| `tray.build.queue.empty` | `○ Queue empty` |
| `tray.build.peek.modifiers` | `Ctrl rotate · Shift scale` |

---

## 5. Toasts & validation

| Key | String |
|:---|:---|
| `toast.place.blocked` | `Placement blocked — {reason}` |
| `toast.place.off_map` | `Click on the map to place` |
| `toast.ghost.risky` | `Risky overlap — check footprint` |

### Footprint strip suffixes

| Key | Suffix |
|:---|:---|
| `ghost.risky` | ` · risky overlap` |
| `ghost.invalid` | ` · blocked: {reason}` |
| `ghost.locked` | ` · locked` |

---

## 6. Esc cascade

| Key | Order |
|:---|:---|
| `esc.cascade.1` | Close build picker sheet |
| `esc.cascade.2` | Collapse context tray |
| `esc.cascade.3` | Open pause menu |

Documented for COD — single handler preferred.

### Road tool sheet (DES-SIM-HUD-POPUP-TIERS-001)

| Key | String |
|:---|:---|
| `road.sheet.title.street` | `Road — Street` |
| `road.sheet.title.highway` | `Road — Highway` |
| `road.sheet.title.rail` | `Rail — Standard` |
| `road.sheet.hint.input` | `LMB add · RMB undo · Shift+LMB commit` |
| `road.sheet.build` | `Build` |
| `road.sheet.cancel` | `Cancel` |
| `road.sheet.upgrade` | `Upgrade nearest segment` |

---

## 7. Ban list (primary labels)

| Never show | Use instead |
|:---|:---|
| `concrete_portland` | `chain.concrete_portland` |
| `builtin:*` | strip prefix or hide |
| `({:.0} power)` | `power.{tier}` |
| `Drag title bar to move` | *(remove)* |
| `Enter to commit` as primary | `rail.footnote.enter` secondary only |

---

## 8. Drift audit (pre-migrate)

| File | Action |
|:---|:---|
| `contextual_tip.rs` | wire keys §1 |
| `industrial_menu.rs` | retire strings → picker |
| `build_toolbox.rs` | mirror §3 or sim-gate off |
| `staged_ghost_panel.rs` | tray keys §4 |
| `validation_feedback.rs` | toast keys §5 |

---

## 9. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-18 |

**Unblocks:** all COD-SIM-HUD-* copy wire tasks
