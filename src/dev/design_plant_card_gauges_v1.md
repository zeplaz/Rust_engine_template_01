# Plant focus card gauges `v1` — nuclear + coal read

| Field | Value |
|:---|:---|
| **ID** | **DES-ART-PLANT-CARD-001** |
| **Program** | PLAN-POWER-GRID-ART-ASSETS-001 · Lane D |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Depends** | [`design_hud_power_icons_v1.md`](design_hud_power_icons_v1.md) · [`design_sim_hud_cohesion_charter_v1.md`](design_sim_hud_cohesion_charter_v1.md) · [`design_nuclear_containment_failure_v1.md`](design_nuclear_containment_failure_v1.md) §5.2 |
| **Handoff** | COD-NUCLEAR-UX-WIRE-001 · COD-ART-HUD-ICON-ATLAS-001 (gauge sprites) |
| **Verdict** | **PASS** |

```text
DES-ART-PLANT-CARD-001 Q✓
Focus card layout + gauge art — core heat, diesel fuel, containment, offsite
```

---

## 0. Scope

**Layout + gauge art only** — no sim logic. Card opens from map select on **generation plants** (nuclear P1+, coal optional output row).

**Chrome:** egui satellite · `bg_elevated` · `wire_magenta` 1px · width **280px**.

---

## 1. Card wire (nuclear PWR)

```text
┌ Nuclear plant — SCRAM ─────────────────── ✕ ┐  ← status tint header
│ Offsite   ● Connected          Diesels  ~ Run │
│ ─────────────────────────────────────────── │
│ Core heat   [████████░░░░░░░░░░]  62%       │  ← gauge §2
│ Diesel fuel [██████████░░░░░░░░]  48 h      │  ← gauge §3
│ Containment [███░░░░░░░░░░░░░░░]  18%       │  ← gauge §4 (P1)
│ ─────────────────────────────────────────── │
│ ~18 min to core damage                      │  ← mono fg_data
└─────────────────────────────────────────────┘
```

| Header status | Header wash |
|:---|:---|
| Operational | `bg_vellum` + `fg_primary` |
| SCRAM | `warn` @ 15% header tint |
| Meltdown | `danger` @ 20% header tint |

---

## 2. Core heat gauge — `gauge_core_heat`

| Field | Value |
|:---|:---|
| Track size | **200×8** px · radius 2px |
| Fill direction | L → R |
| Range | 0–100% (normalized sim) |

| Band | % | Fill color | Track |
|:---|:---:|:---|:---|
| Safe | 0–40 | `fg_data` green family | `bg_paper` |
| Elevated | 40–70 | `warn` amber | `bg_paper` |
| Critical | 70–90 | `warn` → `danger` gradient segment | `bg_paper` |
| Damage | 90–100 | `danger` + 1px pulse ring (2 Hz) | `bg_paper` |

**Icon adjunct (12×12):** `icon_gauge_heat` — three ascending bars + top flame tick (nuclear only).

```text
············
···┌┐·······
··███·······
·█████······
```

---

## 3. Diesel fuel gauge — `gauge_diesel_fuel`

| Field | Value |
|:---|:---|
| Track size | **200×8** px |
| Label | **hours remaining** when running · `Off` / `Starting` / `Failed` text replaces bar |

| State | Bar |
|:---|:---|
| Off | Empty track + `fg_muted` `Off` |
| Starting | 25% fill pulse @ 1 Hz · `accent_terminal` |
| Running | Fill ∝ fuel % · `fg_data` cyan |
| Failed | Full track `danger` hatch · `Failed` label |

**Icon adjunct:** reuse `icon_diesel` from HUD atlas §4.

**Coal variant:** hide row — coal has no diesel backup.

---

## 4. Containment pressure — `gauge_containment`

| Field | Value |
|:---|:---|
| Track size | **200×6** px (slimmer — secondary read) |
| Visibility | Nuclear PWR only · hidden for coal |

| Band | % | Fill |
|:---|:---:|:---|
| Nominal | 0–60 | `fg_muted` |
| Elevated | 60–85 | `warn` |
| Breach risk | 85–100 | `danger` |

**P1 sim:** bar may stay at nominal until Phase 2 — layout slot reserved.

---

## 5. Offsite feed row (status, not gauge)

| State | Glyph | Copy |
|:---|:---|:---|
| Connected | `●` `fg_data` | `Connected` |
| Islanded | `○` `warn` | `Islanded` |
| Restoring | `◐` `accent_terminal` | `Restoring` |

**Icon id:** `icon_offsite_feed` — 8×8 dot in circle (atlas row 3 col 3 spare).

---

## 6. Coal plant card (subset)

```text
┌ Coal plant — Operational ───────────────── ✕ ┐
│ Offsite   ● Connected    Output   420 MW     │
│ ─────────────────────────────────────────── │
│ Stockpile [████████░░░░░░░░░░]  4.2 d        │  ← optional P2
└─────────────────────────────────────────────┘
```

No core heat / diesel / containment rows.

---

## 7. Gauge sprite export

| File | Size | Notes |
|:---|:---|:---|
| `gauge_track_8.png` | 200×8 | neutral track — 9-slice |
| `gauge_fill_safe.png` | 1×8 | green cap |
| `gauge_fill_warn.png` | 1×8 | amber cap |
| `gauge_fill_danger.png` | 1×8 | red cap |
| `gauge_hatch_failed.png` | 8×8 | tile for failed diesel |

Folder: `assets/ui/infrastructure/gauges/`

**Witness:** `debug_runs/sim_hud_plant_card_gauges_live.json`

---

## 8. Acceptance

| # | Check |
|:---:|:---|
| G1 | Line tool icon distinct from road on build rail |
| G2 | L/M/H chips differ by **bar count**, not color alone |
| G3 | Curved vs 90° active state visible without reading label |
| G4 | Core heat ≥70% reads amber/red without opening log |
| G5 | Diesel `Failed` uses hatch — not empty bar |
| G6 | Coal card omits nuclear-only rows |

---

## 9. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-18 |
