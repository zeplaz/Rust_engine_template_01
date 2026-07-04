# City block debug overlay read `v1` — DES-CITY-BLOCK-DEBUG-READ-001

| Field | Value |
|:---|:---|
| **ID** | **DES-CITY-BLOCK-DEBUG-READ-001** |
| **Issue** | CITY-C2 / CITY-C3 debug presentation |
| **Parent** | [`design_city_block_recipe_v1.md`](design_city_block_recipe_v1.md) §6 · [`plan_city_grammar_upgrade_v1.md`](plan_city_grammar_upgrade_v1.md) |
| **Date** | 2026-07-03 |
| **Owner** | `@designer` |
| **Audience** | F3 diagnostics + map editor overlay — **not** default sim player HUD |
| **Depends** | `CITY-G1-C2-001` · `CITY-G1-C3-001` witnesses green |
| **Handoff** | `@coder` — `block_frame` / recipe eval debug tint pass |
| **Verdict** | **PASS** |

```text
DES-CITY-BLOCK-DEBUG-READ-001 Q✓
Engineer overlay legend — block frame, lots, edges, scatter — word + hatch
```

---

## 0. Purpose

When block grammar is debugged, engineers must read **frame anchor**, **street facing**, **lot footprints**, **edge furniture**, and **scatter** without opening RON. This spec locks **copy**, **legend**, and **non-color-only** encoding for the debug overlay path only.

**Rule:** PLAY-01 sim session keeps overlays **off** by default ([`simulation_session.rs`](../gui/hud/simulation_session.rs)). Toggle lives under F3 **Settlement / Block grammar** section.

---

## 1. Toggle + header copy

| Control | Label | Tooltip |
|:---|:---|:---|
| Master toggle | `Block grammar overlay` | Shows block frames and evaluated recipe lots for the focused district. Does not change gameplay. |
| Sub-toggle | `Recipe evaluation` | Colors tiles from the latest BlockRecipe eval (lots, edges, scatter). Requires frame overlay on. |
| Header line | `Block {block_id} · {archetype} · street {street_side}` | Example: `Block blk_12 · Industrial · street +X` |

**Archetype nouns (player-safe):** use `BlockArchetype` display names — `Industrial`, `Low density`, `Medium density`, `Forest park`, etc. — not enum raw ids in UI.

---

## 2. Tile encoding (tactical map)

| Layer | Fill | Hatch / glyph | Label in legend |
|:---|:---|:---|:---|
| **Block frame bounds** | 15% cyan wire | Dashed perimeter | `Frame` |
| **Street edge row** | 20% gold wash | `▌` curb tick on street side | `Street edge` |
| **Building lot** | 25% by archetype (see §3) | Footprint outline 2px | `Lot · {archetype}` |
| **Edge furniture** | 30% green | `═` fence / `♦` lamp / `▲` tree row | `Edge · {asset}` |
| **Scatter** | 25% brown stipple | `·` density dots | `Scatter · {asset}` |
| **Park fill** | 20% grass green flat | diagonal hatch | `Park fill` |
| **Plaza** | 20% grey paved | grid hatch | `Plaza` |

**Accessibility:** every row in §2 must appear in the legend with **word + sample swatch** — never color alone ([`design_sim_hud_a11y_v1.md`](design_sim_hud_a11y_v1.md) A1).

---

## 3. Lot color by building archetype (debug only)

| Grammar id | Legend label | Hue family |
|:---|:---|:---|
| `IndustrialWarehouse` | Warehouse lot | warm amber |
| `CivicBlock` | Civic / residential lot | cool blue |
| `FactoryCluster` | Factory lot | orange |
| `RailEdge` | Rail-edge infill | purple-grey |

---

## 4. F3 diagnostics panel (copy blocks)

```text
Settlement — Block grammar
  Block frame overlay     [toggle]
  Recipe evaluation       [toggle]
  Focus block             {block_id or —}
  Recipe id               {recipe_id or —}
  Lots / edges / scatter  {lot_count} / {edge_count} / {scatter_count}
  Lot hash (stable)       {lot_list_hash}
```

**Empty states:**

| Condition | Copy |
|:---|:---|
| No frame for tile | `No block frame at cursor — pick a settled block or enable district growth debug.` |
| Frame but no recipe | `Frame ready — no BlockRecipe eval yet.` |
| Eval errors | `Recipe eval failed: {first_error}` (red text + icon, not color-only) |

---

## 5. Minimap rule

Minimap **does not** show lot-level recipe debug in v1. When frame overlay on at strategic zoom, show **block archetype wash only** (single color per block centroid) + legend entry `Block type (debug)`.

---

## 6. Acceptance

| # | Check |
|:---:|:---|
| D1 | Toggle labels §1 locked |
| D2 | Legend table §2 complete for five recipe primitives |
| D3 | Archetype lot colors §3 mapped to v1 grammar ids |
| D4 | F3 panel fields §4 listed |
| D5 | Sim default off §0 |
| D6 | Minimap degradation §5 documented |

---

## 7. Non-goals

- APS Assembly tab block editor
- Player-facing block archetype badge in ops strip (future settlement read)
- Palette variation debug (see [`design_city_palette_variation_v1.md`](design_city_palette_variation_v1.md) — separate witness after C5)

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-07-03 |

**Unblocks:** coder debug overlay wiring on `BlockFrameBook` + recipe eval visualization tail.
