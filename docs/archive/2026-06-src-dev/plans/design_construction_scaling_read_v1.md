# DESIGN-CONSTRUCTION-SCALING-READ-001 — Parametric scale + overlap player read `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-CONSTRUCTION-SCALING-READ-001** |
| **Parent** | [`construction_product_roadmap_phases_2_10_v1.md`](construction_product_roadmap_phases_2_10_v1.md) Phase 3 |
| **Baseline** | [`construction_parametric_scale_hud_v1.md`](construction_parametric_scale_hud_v1.md) |
| **Plan** | **PLAN-CONSTRUCTION-SCALING-AUDIT-003** / **CON-P3-S1–S6** |
| **Version** | `1.0.0` |
| **Date** | 2026-06-02 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |
| **Unblocks** | **CON-P3** coder audit pairing |
| **No Rust** | HUD + checklist |

---

## Purpose

Extend signed scale HUD with **overlap / partial-alpha** messaging and provide **designer half** of S1–S6 audit checklist for coder verification.

---

## Scale HUD (extends PARAM-SCALE-HUD-001)

When `origin` is `Some` — keep existing lines 1–2 from [`construction_parametric_scale_hud_v1.md`](construction_parametric_scale_hud_v1.md).

### Overlap / partial-alpha badge

| Condition | Badge | Color |
|:---|:---|:---|
| `overlap_tiles > 0` | `Overlap {n} tiles` | `#e8b040` warn |
| `blocked_tiles > 0` | `Blocked {n} tiles` | `#c04040` |
| `terrain_mod_tiles > 0` | `Terrain prep {n}` | `#80a0c0` |
| All clear | `Footprint clear` | `#40a060` |

Show **one** badge priority: Blocked > Overlap > Terrain > Clear.

### Partial-alpha ghost rule

| `overlap_ratio` | Ghost fill α |
|:---|:---|
| 0 | 0.25 |
| (0, 0.5] | 0.20 |
| (0.5, 1) | 0.12 (both sites visible) |

**Player string:** `Overlapping another site — adjust scale or rotation`

---

## When messaging shows

| Surface | Show overlap? | Show economy scale? |
|:---|:---:|:---:|
| Active ghost drag | yes | yes |
| Staged panel confirm | yes | yes |
| Post-commit site | no | no |
| Minimap | icon only | no |

---

## Designer ↔ coder audit checklist (S1–S6)

| # | Designer acceptance | Coder verify | Pass when |
|:---:|:---|:---|:---|
| **S1** | Presets 1×1…12×12 readable on tray | `scale_factor` clamp + matrix cell count | ghost cells == commit cells |
| **S2** | Occupied tiles show yellow footprint | `FootprintTileWitness` occupied flag | legend + map agree |
| **S3** | Blocked tiles red; commit disabled | `allows_commit == false` | cannot execute |
| **S4** | Terrain mod token in legend | mud/cut/fill token wired | witness or ghost legend |
| **S5** | Rotation + scale persist on site | `BuildingScaleParams` on entity | after commit inspect |
| **S6** | Tray resize ≠ building scale | invariant §15–16 | widget bounds independent |

**Witness:** `construction_parametric_placement_001.green` and/or `construction_scaling_audit_001` (future).

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-02 |
