# GPU minimap compositor M3 — `D-MINIMAP-M3` sign-off `v1`

| Field | Value |
|:---|:---|
| **Review ID** | **D-MINIMAP-M3** |
| **Version** | `0.1.0` |
| **Date** | 2026-05-24 |
| **Owner** | `@designer` → `@coder` **UI-P3-M4-001** (+ sub-slices) |
| **Status** | **SIGNED — DESIGN GATE** · **IMPLEMENTATION OPEN** |
| **Prerequisites** | [`minimap_d_m1_signoff_v1.md`](minimap_d_m1_signoff_v1.md) · [`minimap_d_m2_signoff_v1.md`](minimap_d_m2_signoff_v1.md) |
| **Design** | [`ux_gpu_minimap_design_v1.md`](ux_gpu_minimap_design_v1.md) §7 M3 |
| **Spec** | [`../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/minimap_m3_operational_overlay_spec_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/minimap_m3_operational_overlay_spec_v1.md) |
| **Brief** | [`../docs/archive/2026-06-prompts-guides/runbooks/guides/experience_layer_ux_hud_designer_brief_v1.md`](../docs/archive/2026-06-prompts-guides/runbooks/guides/experience_layer_ux_hud_designer_brief_v1.md) §2 M2/M3 |
| **Witness (target)** | [`debug_runs/minimap_compositor_live.json`](../debug_runs/minimap_compositor_live.json) — extend with M3 fields |

---

## Executive summary

**M3 — operational shell** adds **fog-of-war**, **EW**, **unit aggregation markers**, and **replay / intel scrub** markers on the **existing** GPU minimap compositor — still **no** `MinimapOnlyExtract`.

**Today:** M1+M2 landed; **UI-P3-M4-001** FoW+EW compositor **done** (`ui_p3_m4_green` in witness). Units/replay **open**.

**Verdict:** ☑ **SIGNED — DESIGN GATE** (scope + decisions locked). **M3 COMPLETE** only after witness + coder slices below.

**Naming:** queue id **UI-P3-M3-001** in code = **M2** construction/ecology — do **not** confuse with this **D-MINIMAP-M3** track.

---

## Signed decisions (M3 scope)

| ID | Channel | Choice | Authority (read-only) | Code today |
|:---|:---|:---|:---|:---:|
| **M3-01** | Fog-of-war | **A** — dim unexplored; reveal from intel snapshot | `MinimapOperationalSnapshot` + compositor veil | ☑ |
| **M3-02** | EW coverage | **A** — denial / congestion tint on corridors | `ew_tex` + strategic snapshot seed | ☑ |
| **M3-03** | Unit aggregation | **A** — cluster glyphs at strategic zoom | Stage 7 unit LOD / aggregation snapshot | ☐ |
| **M3-04** | Replay / intel scrub | **B** — timeline tick marks when replay active | Replay editor parity resource (when present) | ☐ |
| **M3-05** | Extract rule | **A** — compositor reads published buffers only | Same as M1/M2 | ☑ policy |
| **M3-06** | Overlay mask | Extend `MinimapOverlayMask` | `fow`, `ew`, `units`, `replay_scrub` bits | ☐ |
| **M3-07** | Tray bridge | Inherit **UI-P3-M2-TRAY-OPT** | HUD overlay tray → mask | ☐ optional |

**Rejected for M3 v1:**

| Option | Why not |
|:---|:---|
| Separate minimap ECS query | Breaks spine; forbidden since M1 |
| egui-owned FoW paint | Conflicts with GPU compositor authority |
| Full tactical unit sprites on minimap | M3 = **aggregation** markers only |

---

## Acceptance criteria (M3 COMPLETE)

| # | Criterion | Witness / test | Met |
|:---:|:---|:---|:---:|
| 1 | FoW channel visible when enabled | `fow_enabled: true` + non-trivial mask sample | ☐ |
| 2 | EW tint when transport EW active | `ew_overlay_enabled: true` + rows or scalar > 0 | ☐ |
| 3 | Unit markers when units on map | `unit_marker_rows > 0` or documented empty fixture | ☐ |
| 4 | Replay scrub when replay session | `replay_scrub_enabled` + markers when timeline active | ☐ |
| 5 | M1+M2 regression | `ui_p3_001_green`, `composite_ok`, isolation witness | ☑ baseline |
| 6 | No new extract | grep / review | ☐ gate |
| 7 | Lib tests | `minimap_compositor` + `stage5` | ☐ |

**Target witness fields (add to live JSON):**

```json
"fow_enabled": true,
"ew_overlay_enabled": true,
"unit_marker_rows": 0,
"replay_scrub_enabled": false,
"ui_p3_m4_green": true
```

---

## Coder slices (priority)

| ID | Goal | First files | Blocks M3 exit |
|:---|:---|:---|:---:|
| **MINIMAP-DESIGN-M3-001** | Designer: spec **SIGNED** | [`minimap_m3_operational_overlay_spec_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/minimap_m3_operational_overlay_spec_v1.md) | ☑ design |
| **UI-P3-M4-001** | FoW + EW compositor passes | `minimap_composite.wgsl`, `composite.rs`, `MinimapOverlayMask` | ☑ |
| **UI-P3-M3-UNITS-001** | Unit aggregation glyphs | `composite.rs`, projection read | optional split from M4 |
| **UI-P3-M3-REPLAY-001** | Replay scrub markers | `live_proof.rs`, replay resource hook | after replay parity green |
| **UI-P3-M2-TRAY-OPT** | Tray → mask (all layers) | `dock_shell.rs`, `minimap_shell.rs` | optional |

**Recommended order:** **MINIMAP-DESIGN-M3-001** (designer) → **UI-P3-M4-001** → units → replay → tray opt.

---

## §11 Designer sign-off checklist (design gate)

| # | Item | Done |
|:---|:---|:---:|
| 1 | M1+M2 **SIGNED** | ☑ |
| 2 | M3 scope distinct from M2 logistics/construction/ecology | ☑ |
| 3 | [`minimap_m3_operational_overlay_spec_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/minimap_m3_operational_overlay_spec_v1.md) drafted | ☑ v0.1 |
| 4 | M3-01…M3-07 decisions recorded | ☑ |
| 5 | Stage 7 / replay dependencies named | ☑ |
| 6 | M3 implementation verified | ☐ |

**Verdict (design gate):** ☑ **SIGNED — DESIGN GATE**

| Role | Date | Verdict |
|:---|:---|:---|
| Designer | 2026-05-24 | **SIGNED** — scope locked |
| Coder | — | **OPEN** — await UI-P3-M4-001 |

**Done when (full D-MINIMAP-M3):** `ui_p3_m4_green: true` in `minimap_compositor_live.json` + § Acceptance table all ☑ + spec **SIGNED**.

---

## Dependencies

| Dependency | M3 need |
|:---|:---|
| **S7B-DESIGN-001** / Stage 7 behavioral | Unit aggregation read model |
| **Replay parity** | `replay_editor_parity_live.json` for scrub markers |
| **LOG-E01 / transport EW** | EW scalar feeds M3-02 |
| **VM-08** | Per-view overlay caps unchanged |

---

## Verification (when implementing)

```powershell
cargo test -p proc_A_dine01 --lib minimap_compositor stage5
$env:MINIMAP_GPU_COMPOSITOR = "1"
cargo run -p proc_A_dine01 --release -- --test visual
```

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v0.1.0 | 2026-05-24 | **D-MINIMAP-M3** design gate SIGNED; implementation open |
