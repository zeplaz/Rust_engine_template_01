# PLAN-VEG-BURN-EXTRACT-001 — burn · succession · extract spine `v1`

```text
⟦SYMLANG⟧⟐v1  ◈EXEC
⟨ID⟩ PLAN-VEG-BURN-EXTRACT-001
Date: 2026-06-14
Status: **SIGNED** (@planner)
Owner: @planner → @coder (A) · @designer-mcp (LG-5 atlas) · @coder-mcp (catalog)
Parent: $ref:src/dev/plan_landscape_grammar_exec_001_v1.md §7.1
Charter: $ref:src/dev/guide_landscape_grammar_v1.md
Lexicon: $ref:prompts/guides/landscape_grammar_lexicon_v1.md (§1.11 ⊗ · §1.17 MACRO-REGROWTH-CHAIN)
Authority map: $ref:.cursor/skills/bevy-simulation-grade/07-repo-authority-map.md
Existing sim: $ref:src/systems/ecology/landscape_grammar_lg2.rs
Construction pattern: $ref:src/construction/procedural/tile_variant_resolver.rs
Fire extract: $ref:src/render/extraction/fire_visual_extract.rs (FireVisualFrameSet)
Runtime proof: $ref:src/dev/plan_veg_runtime_proof_001_v1.md
```

**Goal:** Procedural burn and regrowth **via landscape grammar** — sim writes succession + disturbance; **ActiveBurn** is a transient overlay; extract resolves **glyphs + modifiers → variant_key**; **sprites last** (LG-5).

**Rejected:** per-tree global ECS ∝ map area · density-only burn · render mutating `SuccessionState` · separate plant generator · AI art frames.

---

## 0. Principles (P-001..P-005)

| ID | Principle | Implication |
|:---|:---|:---|
| **P-001** | **Plants are grammar consequences** — topology graph + population fields + deterministic instances | No standalone `PlantGenerator`. Instances sample `VegetationPopulation` + `LandscapeProgramOnChunk.evaluation` |
| **P-002** | **Burn/effects are procedural via grammar** — ⊗ / ○ / `MACRO-REGROWTH-CHAIN` in planning; `SuccessionState` + `LandscapeDisturbanceQueue` in sim | Fire/construction/harvest enqueue disturbance; succession ladder advances on tick |
| **P-003** | **Extract is terminal** — planning glyphs (§1) → extract glyphs (§2) + environmental modifiers; sprites/atlas last | LG-4 must green before LG-5 bake; tint/proxy extract may ship first |
| **P-004** | **ActiveBurn is transient overlay** on `SuccessionState`, not a replacement stage | `BurnScar` persists in succession; `ActiveBurn` holds frame index + heat for extract only |
| **P-005** | **LG-5 uses variant_key lookup** — mirror construction `burning_00..07` tick-driven frame pattern | `VegetationVariantCatalog` + resolver; same seed → same frame at same sim tick |

---

## 1. Authority map (read first)

Repo spine from **07-repo-authority-map** + charter **§12**:

```text
Sim Update (ecology / landscape)
  ChunkEnvironmentSet chain
    ├─ drain_landscape_disturbance_queue     → writes DisturbanceHistory only
    ├─ advance_succession_from_disturbance   → sole writer SuccessionState.stage/age
    ├─ apply_fire_disturbance_to_succession  → ⊗ → BurnScar (graph stage)
    └─ sync_fuel_bridge_from_succession      → reads SuccessionState → VegetationField / ChunkFuelProfile

Sim Update (burn overlay — NEW, same schedule family)
  LandscapeBurnSet::ApplyActiveBurn          → writes ActiveBurn component (transient)
  LandscapeBurnSet::AdvanceRegrowthMacro     → reads SuccessionState + MACRO-REGROWTH-CHAIN

Update (view — after bridge)
  FireVisualFrameSet::BuildProfiles          → fire sim scan (existing)
  VegetationExtractFrameSet::BuildProfiles   → NEW read-only: population + ActiveBurn + succession
    .after(FireVisualFrameSet::BuildProfiles)  (fuel/heat coherence)
    .after(ViewAuthoritySystemSet::SyncViewManager)

Render / preview
  ecology preview tints                    → reads VegetationExtractFrame (not raw ECS scan in UI)
  LG-5 sprite instances                    → variant_key from resolver (terminal)
```

### Single writers (hard)

| Surface | Sole writer | Readers |
|:---|:---|:---|
| `SuccessionState` | `advance_succession_from_disturbance` (+ fire apply) | population derive, fuel bridge, extract |
| `DisturbanceHistory` | disturbance drain + fire/construction hooks | succession advance, diagnostics |
| `VegetationPopulation` | `derive_vegetation_population_from_graph` | ecology integrator, extract |
| `ActiveBurn` | `LandscapeBurnSet::ApplyActiveBurn` | extract frame only |
| `VegetationExtractFrame` | `BuildProfiles` in extract set | preview, minimap, LG-5 resolver |
| `FireVisualFrame` | existing fire extract | VFX, minimap (unchanged) |

⛔ extract writing succession · ⛔ UI writing `ActiveBurn` · ⛔ fire extract writing topology graph

---

## 2. Data model

### 2.1 Succession (existing — extend, do not fork)

```rust
// landscape_grammar_lg2.rs — SuccessionTopologyStage already includes BurnScar
// Add optional metadata only via DisturbanceHistory + graph eval — not parallel enum
```

**Graph path (LG-2-003):** `OldGrowth` ──⊗──► `BurnScar` ──MACRO-REGROWTH──► `Grass` → `Shrub` → …

### 2.2 ActiveBurn (new overlay)

```rust
#[derive(Component, Clone, Debug, Default)]
pub struct ActiveBurn {
    pub heat: f32,              // 0..1 — links ChunkSurfaceFire / fuel
    pub frame_index: u8,        // 0..N-1 — tick-driven like burning_00..07
    pub started_tick: u64,
    pub severity: f32,          // ⊗ cluster intensity
    pub regrowth_macro_phase: RegrowthMacroPhase,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RegrowthMacroPhase {
    #[default]
    None,
    Scar,           // ○ gaps
    Nuclei,         // ◇▒▒▒
    Front,          // ⊕⊕ expansion
    Closing,        // ▓
    Mature,         // █ — hands off to succession stage
}
```

**Rule:** When `heat == 0` and `regrowth_macro_phase == Mature`, **remove** `ActiveBurn` — `SuccessionState` owns long-term stage.

### 2.3 Extract frame (new resource)

```rust
#[derive(Resource, Default)]
pub struct VegetationExtractFrame {
    pub revision: u64,
    pub rows: Vec<VegExtractRow>,  // per subcell or partition tile
}

pub struct VegExtractRow {
    pub coord: IVec2,
    pub planning_glyph: char,       // §1 — debug/diagnostics only in sim HUD
    pub extract_glyph: char,        // §2 — @#%*…
    pub modifiers: VegExtractModifiers,
    pub variant_key: String,        // LG-5 lookup — empty until atlas
    pub succession_stage: SuccessionTopologyStage,
    pub burn_active: bool,
}
```

Mirror **`FireVisualFrame`** immutability: built once per frame from sim snapshot, consumed by render/preview.

---

## 3. MACRO-REGROWTH-CHAIN (planning → sim)

Lexicon **MACRO-REGROWTH-CHAIN:** `○ → ◇▒▒▒ → ⊕⊕ → ▓ → █`

| Macro phase | Planning | Sim signal | Extract glyph bias |
|:---|:---|:---|:---|
| Scar | ○ | `BurnScar` + low canopy | `@` sparse / `%` scar |
| Nuclei | ◇▒▒▒ | `RegrowthMacroPhase::Nuclei` | `*` clusters |
| Front | ⊕⊕ | `RegrowthMacroPhase::Front` | `^`/`v` edge bias |
| Closing | ▓ | shrub stage rising | `#` mid density |
| Mature | █ | remove `ActiveBurn`; `YoungForest`+ | `#`/`=` full |

**Determinism:** phase advance keyed on `(chunk_coord, subcell, disturbance_tick, preset_seed)` — no unseeded RNG.

---

## 4. Burning-tree state machine (overlay SM)

```text
                    ┌─────────────────┐
     ChunkSurfaceFire / fuel threshold
                    ▼
              ┌───────────┐
              │  IGNITE   │  ActiveBurn spawned · frame_index=0
              └─────┬─────┘
                    ▼
         ┌──────────────────────┐
    ┌───►│  BURN_TICK (loop)    │◄───┐
    │    │  frame = f(tick, N)  │    │ heat > ε
    │    └──────────┬───────────┘    │
    │               │ heat → 0       │
    │               ▼                │
    │    ┌──────────────────────┐    │
    │    │  COOL / SCAR         │    │ SuccessionState → BurnScar
    │    └──────────┬───────────┘    │
    │               ▼                │
    │    ┌──────────────────────┐    │
    └───►│  REGROWTH_MACRO      │    │ MACRO-REGROWTH-CHAIN phases
         └──────────┬───────────┘    │
                    ▼ Mature         │
              ┌───────────┐          │
              │  REMOVE   │          │ despawn ActiveBurn
              │  overlay  │          │
              └───────────┘          │
```

**Construction parity:** same formula family as `tile_variant_resolver` fire frames:

```text
frame_index = ((sim_tick - started_tick) * 1000 / frame_period_ms) % frame_count
variant_key = format!("{prefix}{:02}", frame_index)   // e.g. veg_burn_00
```

Catalog lives in `assets/configs/landscape/_vegetation_variant_catalog.ron` (planner-mcp schema; coder-mcp validates).

---

## 5. Coder slices (sequenced)

| ID | Phase | Task | Witness |
|:---|:---:|:---|:---|
| **VEG-BURN-OVERLAY-001** | A | `ActiveBurn` component + `LandscapeBurnSet` schedule in `ChunkEnvironmentSet` | `landscape_grammar_burn_overlay_live.json` |
| **VEG-BURN-SM-002** | A | Ignite/cool from `ChunkSurfaceFire` + fuel; frame tick without atlas | `burn_frame_determinism` unit test |
| **VEG-BURN-SUCCESSION-003** | A | ⊗ disturbs graph stage; `MACRO-REGROWTH-CHAIN` advances on tick | extends `landscape_grammar_lg2_live.json` |
| **VEG-BURN-EXTRACT-004** | B | `VegetationExtractFrame` + `BuildProfiles` read-only extract | `landscape_grammar_extract_live.json` |
| **VEG-BURN-GLYPH-005** | B | Planning → extract glyph map per lexicon §3 on rows | same witness `extract_glyph_deterministic` |
| **VEG-BURN-FULLAPP-006** | B | FULL_APP refresh includes burn rows + `variant_key` stub | `stage5_full_app_live.json` |
| **VEG-BURN-PLAY-007** | B | `play_scenario_live.json` key `veg_burn_visible_at_operational_zoom` | G-PLAY extension |
| **VEG-LG5-CATALOG-008** | C | `@coder-mcp` + `@designer-mcp`: minimal veg atlas + catalog keys | blocked on MCP-LANDSCAPE-GRAMMAR-SIGN-001 |
| **VEG-LG5-RESOLVER-009** | C | `resolve_vegetation_variant()` — PT-4 pattern for plants | `procedural_veg_runtime_live.json` |

**Phase A** may ship with **empty `variant_key`** + preview tint/modifier only — LG-5 atlas is Phase C.

---

## 6. LG-5 variant_key catalog (sketch)

Minimal ship set (designer-mcp expands):

| variant_key | Condition |
|:---|:---|
| `veg_clean_day` | default · no ActiveBurn |
| `veg_damaged` | succession `BurnScar` or damage modifier |
| `veg_burn_00` … `veg_burn_07` | `ActiveBurn` frame loop |
| `veg_regrowth_nuclei` | `RegrowthMacroPhase::Nuclei` |
| `veg_regrowth_front` | `RegrowthMacroPhase::Front` |
| `veg_old_growth` | `OldGrowth` + Cx5 |

**Atlas:** iso tile batch under `assets/staging/tiles/tile_veg_*` — keyframe bake spine when designer signs; until then extract uses **tint + glyph overlay** (LG-4).

---

## 7. Witness acceptance

| Probe | Pass |
|:---|:---|
| Fire on old-growth sets `ActiveBurn` + `DisturbanceHistory` fire event | linked ticks |
| Same seed + tick → same `frame_index` | determinism |
| Cool-down removes `ActiveBurn`; `SuccessionState` stays `BurnScar` until macro completes | overlay ≠ stage |
| `MACRO-REGROWTH-CHAIN` visible in diagnostics timeline | 5 phases ordered |
| Extract frame revision bumps once per sim tick max | no dual rebuild |
| No global Tree entity count ∝ map area | bounded rows |
| LG-5 keys resolve when catalog present | optional until Phase C |

---

## 8. Queue placement

| Queue | Row |
|:---|:---|
| `post_drain_phase6_coder_queue.json` | Insert after seq 11 **VEG-FIRE-CORRIDOR-FULLAPP-001** — VEG-BURN-OVERLAY-001 … EXTRACT-004 |
| `planner_active_queue.json` | **PLAN-VEG-BURN-EXTRACT-001** signed |
| `mcp_active_queue.json` | `_vegetation_variant_catalog.ron` schema after MCP-LANDSCAPE-GRAMMAR-SIGN-001 |

**Regression:**

```powershell
cargo test -p proc_A_dine01 --lib landscape_grammar fire_ecology
```

---

## 9. Designer / MCP hooks

| Owner | Deliverable |
|:---|:---|
| **@designer-mcp** | LG-5 minimal iso atlas spec — burn frame count, anchor, scale band S/M |
| **@coder-mcp** | `validate-report landscape_grammar` + variant catalog validator |
| **@designer** | Diagnostics panel: succession ladder + active burn frame (read-only) |

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-14 | Initial plan — P-001..P-005 · authority map · ActiveBurn overlay · LG-5 defer |

```text
⟦/PLAN-VEG-BURN-EXTRACT-001⟧  ΔWF→@coder A VEG-BURN-OVERLAY-001 · @planner-mcp veg catalog schema
```
