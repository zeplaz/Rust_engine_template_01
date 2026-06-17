# DES-ECOLOGY-READ-HUD-001 — World preview ecology legend `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-ECOLOGY-READ-HUD-001** |
| **Date** | 2026-06-16 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |

---

## Purpose

When world preview shows **topology tint** / ecology program on chunk (LG-4+), player/operator must read kind without opening APS.

## Legend (world preview chrome — bottom-left, collapsible)

| Glyph | Word | Tint token | Meaning |
|:---:|:---|:---|:---|
| `N` | **Network** | `#4a6fa5` | Connected graph backbone |
| `C` | **Corridor** | `#7a6a4a` | Linear edge (road/rail/spoil) |
| `P` | **Patch** | `#3d8b5f` | Irregular disturbance patch |
| `R` | **Ring** | `#6a5a8a` | Enclosure / buffer ring |
| `K` | **Cluster** | `#2f7d4a` | Natural cluster mass |
| `F` | **Fringe** | `#8a9a6a` | Edge / margin transition |

**Rule:** glyph + word always; tint reinforces only. Legend collapsed default in sim; expanded in WorldGen preview.

## Minimap cross-link

Tokens align with **DES-MINIMAP-VEG-LEGEND-001** (P3) — same words, smaller chips.

## Operator read @ operational zoom

At default tactical zoom: ≥3 topology kinds visible with distinct tint OR legend shows `○ ecology preview off`.

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-16 |
