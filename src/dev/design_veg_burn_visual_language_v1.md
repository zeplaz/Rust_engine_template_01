# Vegetation burn / scar / recovery visual language `v1` (DMCP-BURN-VISUAL-LANG-001)

| Field | Value |
|:---|:---|
| **Owner** | `@designer-mcp` |
| **Date** | 2026-06-17 |
| **Lexicon** | [`prompts/guides/landscape_grammar_lexicon_v1.md`](../prompts/guides/landscape_grammar_lexicon_v1.md) §1 ⊗ · §2 `x` |

---

## Readability contract (iso zoom)

Artist and operator must distinguish **burn**, **scar**, and **regrowth** at **1 chunk ≈ 64px tile** without opening diagnostics.

| State | Planning glyph | Extract glyph | Silhouette | Hue family |
|:---|:---:|:---:|:---|:---|
| **Burn active** | ⊗ | `x` | Irregular orange core + smoke fringe | Warm `#e87830`–`#ff9930` |
| **Scar** | ○ | `x` (cool) | Flat ash void, sharp edge | Neutral `#3a3a3a` |
| **Regrowth grass** | ▒ | `,` | Fine scatter, low height | Yellow-green `#8aaa58` |
| **Regrowth shrub** | ▒▒ | `.` | Blobby mid height | Mid green `#4a8040` |
| **Regrowth canopy** | * | `*` | Diamond / clump returning | Dark green `#2d7038` |

---

## Frame language (burn_00..07)

| Frame band | Visual |
|:---|:---|
| 00–02 | Ember onset — thin warm rim on scar base |
| 03–05 | Peak — brightest core, widest smoke |
| 06–07 | Decay — dim ember, ash dominates |

**Rule:** Frames are **deterministic** (`fire_frame` axis); same seed → same sheet order.

---

## Topology interaction

| Topology | Burn reads as… |
|:---|:---|
| **Patch** | Crown fire — gap in canopy mass |
| **Corridor** | Linear fire along transport spine |
| **Ring** | Segment ignition on enclosure |
| **Cluster** | Spot fires in regrowth nuclei |
| **Fringe** | Edge creep — low contrast, still visible |

---

## Accessibility

- Never **color-only**: scar uses **flat void** shape; burn uses **bright rim**; regrowth uses **texture scatter**.
- Diagnostics may echo plain labels: “Burn frame 4 · Patch scar”.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS** | 2026-06-17 |
