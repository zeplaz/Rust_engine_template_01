# Utility industrial style bible `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-ART-UTILITY-STYLE-001** |
| **Program** | PLAN-POWER-GRID-ART-ASSETS-001 · Lane F |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Authority** | [`plan_power_grid_art_assets_v1.md`](plan_power_grid_art_assets_v1.md) · [`design_infra_network_overlay_v1.md`](design_infra_network_overlay_v1.md) |
| **Pilots** | [`power_substation_yard_site_v0.json`](../../assets/configs/buildings/pilots/power_substation_yard_site_v0.json) |
| **Verdict** | **PASS** |

```text
DES-ART-UTILITY-STYLE-001 Q✓
Blocks all utility module specs — substation yard, transformer pad, bus read
```

---

## 0. Scope

Visual language for **grid infrastructure** — substation yards, transformer pads, bus geometry, warning paint — aligned with **industrial_west** module kit but **utilitarian** (less façade ornament than factories).

**Feeds:** DMCP-SPEC-SUBSTATION-YARD-001 · DMCP-SPEC-TRANSFORMER-PAD-001 · all Lane A modules.

---

## 1. Aesthetic contract

| Token | Utility use | Source |
|:---|:---|:---|
| Galvanized steel | bus bars, transformer tanks, fence | `#a8b0b8` base |
| Ceramic insulator | bushings — **white/cream** read | `#e8e4dc` |
| Concrete pad | equipment footing | `#8a8884` |
| Gravel yard | void fill | `#6a6864` speckle |
| Warning yellow | signs, bollards | `#e8c03a` — sparse |
| Live power accent | map/HUD only | `#e8c040` gold family |
| Status cyan | labels | `UiPalette.fg_primary` |

**Ban:** residential brick · decorative windows on switchgear · rust as default (weathering accent only).

---

## 2. Silhouette rules (@ 32px / iso 64px)

| Asset | Must read |
|:---|:---|
| **Substation yard** | Low horizontal mass + **tall bus** or breaker silhouette |
| **Transformer pad** | **Cylinder + 3 bushings** on top — insulators = white dots |
| **Fence** | Perimeter rhythm — not solid wall |
| **Control shack** | Small box — secondary to yard |

**Height budget:** transformer ≤1.5 grid units · bus structures ≤2u · fence 1u.

---

## 3. Substation yard (4×3 catalog)

**Authority:** `grid_substation.json` · site `power_substation_yard_site_v0`

| Zone | Art content |
|:---|:---|
| **primary** (center) | Bus bays — simplified parallel bars, breaker blocks |
| **utility** (ring) | Gravel, fence, cable drums optional |
| **service** | Control shack 1×1 module slot |
| **buffer** | Setback gravel — no clutter |

**Massing:** `yard_complex` weight 28% per ARCH-DNA pilot — open yard with equipment islands.

**Module whitelist:** `bus_bay_simplified`, `breaker_block`, `fence_chainlink_1u`, `gravel_pad_1u`, `prop_transformer` (slot), `control_shack_1u`, `warning_sign_1u`.

---

## 4. Transformer pad (2×2 catalog)

**Authority:** `grid_distribution_transformer.json`

| Element | Spec |
|:---|:---|
| Footprint | 2×2 tiles · single prop centered |
| Tank | Horizontal cylinder, galvanized |
| Bushings | 3 ceramic caps — **readable @ 32px** |
| Oil berm | Optional 0.25u raised lip — utility tier |
| Pad | Concrete slab flush + 1 tile gravel margin |

**Snap:** `floor_center` · pivot `bottom_center`.

---

## 5. Bus geometry language

```text
═══  bus bar (horizontal MV)
║    riser / breaker gap
▣    breaker housing (simplified cube)
```

- **No** exposed live conductor mesh in gameplay LOD0 — implied geometry only.
- **Corner rule:** bus turns 90° only — matches power routing orthogonal mode.

---

## 6. Materials (profile ids for DMCP)

| Profile id | Use |
|:---|:---|
| `galvanized_steel_01` | tanks, bus, fence |
| `ceramic_insulator_01` | bushings |
| `concrete_pad_01` | footings |
| `gravel_yard_01` | yard void |
| `warning_paint_yellow_01` | signs, bollard caps |

DMCP-MAT-UTILITY-PACK-001 ships profiles — this doc names slots.

---

## 7. Damage / state read (3D + glyph)

| State | 3D cue | Glyph pairs with [`power_glyphs_spec_v1.md`](../../assets/ui/infrastructure/power_glyphs_spec_v1.md) |
|:---|:---|:---|
| Live | Normal materials | base node glyph |
| Damaged | Soot, bent bus | amber hatch overlay |
| Destroyed | Collapsed breaker | × overlay |
| Overload | Heat tint on transformer | spark adjunct |

---

## 8. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-18 |

**Unblocks:** DMCP-SPEC-SUBSTATION-YARD-001 · DMCP-SPEC-TRANSFORMER-PAD-001 · DES-ART-NUCLEAR-PLANT-001
