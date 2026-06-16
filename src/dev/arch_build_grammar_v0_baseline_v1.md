# ARCH-BUILD-GRAMMAR-v0 — baseline schema `v1`

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **BUILD-READ-GRAMMAR-v0-001** |
| **Owner** | @planner-mcp (schema) → @coder-mcp (APS) → @coder (evaluator) |
| **Status** | **SIGNED** |
| **Lang** | $ref:src/dev/agent_lang_v1.md · $ref:prompts/SYMBOLIC_LANGUAGE.meta.md |
| **Operator guide** | $ref:prompts/guides/build_grammer2_exman.md |
| **Landscape mirror** | $ref:src/dev/guide_landscape_grammar_v1.md · $ref:prompts/guides/landscape_grammar_lexicon_v1.md (v1.4.0) · $ref:tools/mcp/schemas/landscape_grammar_v0.schema.json |
| **Parent exec** | $ref:src/dev/plan_operator_build_readability_exec_001_v1.md |
| **Prior schema** | $ref:docs/archive/2026-06-src-dev/plans/arch_build_grammar_001_schema_v1.md |
| **JSON schema** | $ref:tools/mcp/schemas/arch_build_grammar_v0.schema.json |
| **Pilot preset** | $ref:tools/mcp/schemas/examples/arch_dna_logistics_rail_warehouse_v0.json |
| **Date** | 2026-06-13 |

**Rule:** Schema + preset tables only — **no Rust/Python in this deliverable.** v0 **ranks** existing `massing.strategies` weights; it does **not** author new geometry ops.

---

## Meta-rules (guide)

```text
Function > shape · Site > building · Pressure > shape
Era = weak · Lineage + function = strong
```

**Site occupancy:** primary structure ≈ **15–40%** of site (not one rect filling the chunk).

---

## Pipeline (v0)

```text
SITE◈WORLD-DNA (stub)
      ▼
ARCH-DNA { F,L,C,D,W,I,S,P,M,A }
      ▼
PRESSURE-FIELD { β v0 × 8 }
      ▼
SHAPE-GRAMMAR  → re-weight massing.strategies (existing ids)
      ▼
SITE-COMPOSITION stub (primary · yard · service · rail edge)
      ▼
MODULE-RUNS → MATERIAL → WEATHERING  → WRK/ATL lanes (unchanged)
      ▼
FINAL-ASSEMBLY / commit funnel (unchanged)
```

**BLANG spine hook:**

```text
AUTH: … ⇢ SNAP★ ⇢ WRK○ ⇢ ATL○
GRAMMAR: ARCH-DNA◈ → β◈ → SHAPE◈ → SITE◈ → MODULE-RUNS◈
```

---

## 1. `ArchDna` struct

| Key | Name | Enum values (v0) |
|:---|:---|:---|
| `F` | function | `logistics` · `manufacturing` · `housing` · `military` · `utility` · `port` · `commercial` · `government` |
| `L` | lineage | `roman` · `industrial_british` · `soviet` · `nato` · `germanic` · `japanese` · `colonial` · `corporate` · `arcology` · `frontier` |
| `C` | climate | `arctic` · `temperate` · `mediterranean` · `tropical` · `desert` · `mountain` · `wetland` · `monsoon` |
| `D` | density | `sparse` · `rural` · `town` · `urban` · `dense_urban` · `megacity` |
| `W` | wealth | `poor` · `working` · `middle` · `industrial` · `elite` · `state` |
| `I` | infrastructure | `road` · `rail` · `port` · `airport` · `canal` · `pipeline` · `power_hub` |
| `S` | security | `open` · `controlled` · `protected` · `fortified` · `military` |
| `P` | philosophy | `utilitarian` · `monumental` · `humanist` · `corporate` · `efficiency` · `defensive` · `prestige` |
| `M` | material_strategy | `steel` · `concrete` · `brick` · `stone` · `glass` · `composite` · `hybrid` |
| `A` | age | `new` · `maintained` · `weathered` · `declining` · `ruined` |

**Storage:** `arch_dna` object on assembly snapshot / APS grammar panel (optional v0 extension).

---

## 2. `PressureField` v0 (8 β keys)

| Key | Meaning | Range | Notes |
|:---|:---|:---:|:---|
| `βsym` | symmetry | 0.0–1.0 | ↑ favors `long_hall` · `double_hall` |
| `βirr` | irregularity | 0.0–1.0 | ↑ favors `l_shape` |
| `βyard` | yard pressure | 0.0–1.0 | ↑ favors `yard_complex` · site yard zone |
| `βsvc` | service space | 0.0–1.0 | ↑ favors `l_shape` · utility wing |
| `βmod` | modularity | 0.0–1.0 | module-run density (PG-2) |
| `βexp` | expansion | 0.0–1.0 | ↑ favors `yard_complex` · links **Shift+scale** |
| `βvert` | verticality | 0.0–1.0 | caps `floors` in evaluator |
| `βroof` | roof complexity | 0.0–1.0 | roof slot selection in existing `roof.by_massing` |

### Deferred β (document only — vNext)

`βorn` · `βdef` · `βctl` · `βentropy` · `βinertia` · `βdepth` · `βcourt` · `βind` · `βland`

---

## 3. Preset — `logistics_rail_warehouse_v0`

### ARCH-DNA row (pilot)

| Key | Value |
|:---|:---|
| F | `logistics` |
| L | `industrial_british` |
| C | `temperate` |
| D | `sparse` |
| W | `industrial` |
| I | `rail` |
| S | `controlled` |
| P | `utilitarian` |
| M | `steel` |
| A | `weathered` |

### Pressure field (from guide §ARCH-DNA EXAMPLE)

| β | Value |
|:---|:---:|
| βsym | 0.72 |
| βirr | 0.24 |
| βyard | 0.93 |
| βsvc | 0.88 |
| βmod | 0.92 |
| βexp | 0.84 |
| βvert | 0.18 |
| βroof | 0.63 |

### Program stub (data only v0)

```json
{
  "storage": "high",
  "loading": "high",
  "office": "low",
  "service": "medium",
  "expansion": "high"
}
```

Topology hint (not solver): `Loading → Storage → Office|Utility`

---

## 4. β → massing strategy weights (v0 algorithm)

**Input:** `industrial_warehouse_v1.ron` base strategies (or any grammar with `massing.strategies[]`).

**Output:** re-ranked weights — same **ids**, new **weight** integers (sum normalized to 100).

### Label map (guide → repo id)

| Guide label | Repo `massing.strategies[].id` | `footprint_mode` |
|:---|:---|:---|
| Bar | `long_hall` | `rect` |
| DoubleBar | `double_hall` | `rect` |
| RailEdge · ServiceYard | `l_shape` | `l_shape` |
| Courtyard · expansion yards | `yard_complex` | `yard_interior` |
| SawtoothHall · FactoryCluster | *(labels only v0)* | weights via βroof · βmod tags |

### v0 weight formula (deterministic)

For each strategy `s` in `{long_hall, double_hall, l_shape, yard_complex}`:

```text
score(s) = base_weight(s)
         + k_yard  × βyard  × yard_bias(s)
         + k_svc   × βsvc   × svc_bias(s)
         + k_sym   × βsym   × sym_bias(s)
         + k_exp   × βexp   × exp_bias(s)
         + k_irr   × βirr   × irr_bias(s)
```

| `s` | yard_bias | svc_bias | sym_bias | exp_bias | irr_bias |
|:---|:---:|:---:|:---:|:---:|:---:|
| long_hall | 0 | 0 | 1.0 | 0 | 0 |
| double_hall | 0 | 0 | 0.8 | 0.2 | 0 |
| l_shape | 0.6 | 1.0 | 0 | 0.3 | 0.8 |
| yard_complex | 1.0 | 0.5 | 0 | 1.0 | 0.4 |

Constants (v0): `k_yard=k_svc=k_sym=k_exp=k_irr=40` (integer-friendly).

**Floors cap:** `floors = clamp(round(1 + βvert × 3), min_floors, max_floors)`.

### Preset result — `logistics_rail_warehouse_v0` (seed-stable)

| id | base | **v0 adjusted** | footprint_mode |
|:---|:---:|:---:|:---|
| `long_hall` | 35 | **22** | rect |
| `double_hall` | 30 | **24** | rect |
| `l_shape` | 20 | **28** | l_shape |
| `yard_complex` | 15 | **26** | yard_interior |

**Pick:** weighted random by adjusted weights + `district_style` + seed (same API as v1 `generate()`).

---

## 5. Site-composition stub (view / data v0)

Zones (ASCII from guide — designer mock authority):

```text
Rail Spur═══════╗
                ║
 Warehouse A════╬════Loading
                ║
 Utility Yard═══╝
```

| Zone | Role | Occupancy target |
|:---|:---|:---|
| primary | warehouse hall | 15–40% site tiles |
| loading_wing | dock / rail edge | attached to primary |
| utility_yard | tanks · parking | peripheral |
| rail_spur | infrastructure edge | `I=rail` hint |

**v0:** overlay only — **no** new commit path. Data fields: `site_zones[]` with `{id, role, footprint_hint}`.

---

## 6. Downstream lanes (skills)

| Stage | Lane | Skill |
|:---|:---|:---|
| MODULE-RUNS | geometry | $ref:.claude/skills/blender-geometry/SKILL.md |
| MATERIAL | APS / MAT | $ref:.claude/skills/mcp-asset-pipeline/SKILL.md |
| WEATHERING | tiles state | $ref:.claude/skills/tile-generation/SKILL.md |
| `A=weathered` | `variant_tags` | maps to tile `damage` / `age` bands |

**$sym hooks:**

| Symbol | Consumer |
|:---|:---|
| `$sym:BuildingGrammar@src/construction/procedural/building_grammar.rs` | evaluator |
| `$sym:generate@tools/mcp/python/rust_engine_mcp/building_grammar.py` | APS parity |
| `$sym:FootprintMatrix@assets/configs/buildings/` | commit path |

---

## 7. OUT OF SCOPE (v0 — do not assign)

| Concept | Why defer |
|:---|:---|
| ProgramGraph / FlowGraph | Needs ECS solver |
| AdjacencyMatrix / topology solver | v0 uses program **stub** JSON only |
| OperatorStack (AddVolume, Carve, …) | Needs volume graph |
| GrowthEpochs / operator history | Save + sim slice |
| SettlementGrammar / steelworks graphs | District lane |
| `βctl` · `βentropy` · `βinertia` · `βdepth` | Schema doc only |
| Topology classes LINEAR / RADIAL / NETWORK | Classify in vNext planner doc |
| Remove `massing_strategy` enum | After volume graph lands |
| File split `program/` · `topology/` · `growth/` | vNext repo layout |
| Chat-only bpy / mesh | MCP JSON only |

---

## 8. Migration path

| Phase | Repo state | v0 action | vNext |
|:---|:---|:---|:---|
| **Today** | `building_grammar_v1` + `footprint_mode` enum | Add optional `arch_dna` + `pressure_field` on snapshot | — |
| **v0 witness** | APS preset + evaluator re-weight | `BUILD-READ-GRAMMAR-v0-002` pytest | — |
| **vNext** | Volume graph + operators | Retire shape-as-input; `footprint_mode` derived | ProgramGraph |

**Invariant:** v0 keeps `footprint_mode` on massing strategies — evaluator **selects** among existing modes; does not invent L geometry without `FootprintMatrix` + weighted raster (@coder).

---

## 9. Acceptance (schema done)

| # | Criterion |
|:---:|:---|
| 1 | `arch_build_grammar_v0.schema.json` validates example preset |
| 2 | Eight β keys documented with ranges |
| 3 | `logistics_rail_warehouse_v0` DNA + β + adjusted weights table |
| 4 | OUT OF SCOPE list explicit |
| 5 | Skills cross-link MODULE-RUNS / MATERIAL / WEATHERING |

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-13 | @planner-mcp — v0 baseline per BUILD-READ-GRAMMAR-v0-001 |
