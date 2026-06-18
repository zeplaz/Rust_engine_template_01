# Power tier bands — designer units → APS glyphs `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-POWER-TIER-001** |
| **Program** | PLAN-INDUSTRIAL-FACILITY-GRAMMAR-001 · Track E1-B |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Authority** | `power_consumption` in catalog + chains JSON · MW mapping [`supply_chain.rs`](../economy/supply_chain.rs) |
| **Design system** | [`aps_design_system_v1.md`](aps_design_system_v1.md) §3.4 — glyph + word |
| **Research** | [`design_industrial_process_research_v1.md`](design_industrial_process_research_v1.md) |
| **Verdict** | **PASS** |

```text
DES-POWER-TIER-001 Q✓
power_tier derived from catalog — never typed in grammar alone
```

---

## 0. Designer units → grid load

| Designer units | Grid load (MW proxy) | Formula |
|:---:|:---:|:---|
| 18 | 0.18 | `power_consumption / 100` |
| 72 | 0.72 | same |
| 200 | 2.00 | same |

**Display in APS:** show **tier word** + optional `(NN units)` in tooltip — not raw MW unless diagnostics expanded.

---

## 1. Tier bands (authoritative)

| Tier | Designer units | Inclusive range | Typical roles |
|:---|:---:|:---:|:---|
| **light** | 0–30 | `[0, 30]` | mine, mixer, parking, transformer |
| **medium** | 31–80 | `[31, 80]` | kiln, refinery, fabrication, integrated plant |
| **heavy** | 81–200 | `[81, 200]` | smelter |
| **grid** | utility infra | `utility_role` set | substation, power_plant, transformer pad |

**Derivation function (coder-mcp):**

```python
def power_tier_from_units(units: float, *, utility_role: str | None = None) -> str:
    if utility_role:
        return "grid"
    if units <= 30:
        return "light"
    if units <= 80:
        return "medium"
    if units <= 200:
        return "heavy"
    return "heavy"  # clamp — flag validator warn above 200
```

**Generation exception:** `power_generation > power_consumption` with `utility_role == power_plant` → always **grid** regardless of consumption field.

---

## 2. Chain step → tier map (from disk)

### concrete_portland

| Role | catalog_id | Units | Tier |
|:---|:---|:---:|:---:|
| aggregate_mine | `concrete_aggregate_mine` | 18 | light |
| cement_kiln | `concrete_cement_kiln` | 72 | medium |
| concrete_mixer | `concrete_mixer_plant` | 28 | light |
| integrated_plant | `concrete_basic_production_plant` | 50 | medium |

### aluminum_primary

| Role | catalog_id | Units | Tier |
|:---|:---|:---:|:---:|
| bauxite_mine | `aluminum_bauxite_mine` | 22 | light |
| alumina_refinery | `aluminum_alumina_refinery` | 85 | medium |
| aluminum_smelter | `aluminum_smelter1` | **200** | **heavy** |
| aluminum_fabrication | `aluminum_fabrication_plant` | 48 | medium |

### utility (grid tier)

| catalog_id | utility_role | Tier |
|:---|:---|:---:|
| `grid_substation` | substation | grid |
| `grid_distribution_transformer` | transformer | grid |
| `utilities_coal_plant` | power_plant | grid |

---

## 3. APS power glyph (`power_tier_atom`)

Distinct from **status** atoms (✓ ✗ ◐ ○ ⟳) — power uses **bolt family** + tier word.

| Tier | Glyph | Word | FG token | BG token |
|:---|:---:|:---|:---|:---|
| light | `⚡` | light | `COLOR_MUTED` | `COLOR_INPUT_BG` |
| medium | `⚡⚡` | medium | `COLOR_WARN` | `COLOR_WARN_BG` |
| heavy | `⚡⚡⚡` | heavy | `COLOR_FAIL` | `COLOR_FAIL_BG` |
| grid | `⊞` | grid | `COLOR_ACCENT` | `COLOR_PASS_BG` |

**Line format:** `{glyph} {word} power[ — {detail}]`

Examples:
- `⚡ light power — aggregate mine`
- `⚡⚡ medium power — cement kiln (72)`
- `⚡⚡⚡ heavy power — smelter`
- `⊞ grid power — substation`

**A11y:** never glyph-only — word **light|medium|heavy|grid** always present.

**Implement:** `power_tier_atom(tier, *, detail=None) -> (glyph, word, fg, bg)` in `aps_inline_feedback.py` (coder-mcp).

---

## 4. Grammar module density hints

Tier drives **optional module budget** in grammar iterate — not sim authority.

| Tier | Stack modules | Pipe / rack | Yard modules | Cooling |
|:---|:---:|:---:|:---:|:---:|
| light | 0–1 | 0 | 0–1 | — |
| medium | 1–2 | 1–2 | 1–2 | optional |
| heavy | 2–3 | 2–3 | **2–4** | **required read** |
| grid | pad / transformer | bus | fence | — |

**Sweep histogram (CMCP-GRAM-SWEEP-PROCESS-001):** count grammars per `power_tier` + `supply_chain_role`.

---

## 5. APS surfaces

### Facility Needs strip (DES-APS-FACILITY-NEEDS-001)

| Field | Source | Display |
|:---|:---|:---|
| Power tier | `power_tier_from_units(catalog.power_consumption)` | `power_tier_atom` |
| Chain step | `supply_chain_role` | human label from copy pack |
| Top I/O | `produces` / `consumes` first 3 each | comma list |

### Build HUD (COD-BUILD-READ-PROCESS-001)

Same tier words — no triple-bolt in world HUD if cluttered; use `⚡ heavy` single glyph + word.

### Diagnostics expanded

Show designer units + MW proxy: `72 units (0.72 load)`.

---

## 6. Edge cases

| Case | Rule |
|:---|:---|
| `power_consumption == 0` | tier `light` + `○ no load` status |
| Missing catalog field | `○ pending — no power data` · block facility binding sign-off |
| Grammar `power_tier` override | **forbidden** — validator strips |
| Geopolymer kiln (58) | medium |
| Legacy monolith (50) | medium |

---

## 7. Verification

| Check | Method |
|:---|:---|
| All chain steps map to tier | unit test `test_power_tier_from_chain_json` |
| Smelter = heavy | assert `aluminum_smelter1` → heavy |
| Substation = grid | assert `utility_role` path |
| APS glyph render | snapshot Facility Needs strip mock |

---

## 8. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-18 |

**Unblocks:** DES-APS-FACILITY-NEEDS-001 · CMCP-GRAM-SWEEP-PROCESS-001 · `power_tier` on `facility_binding`
