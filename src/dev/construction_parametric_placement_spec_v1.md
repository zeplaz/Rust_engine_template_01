# Construction parametric placement — product / UX spec `v1`

| Field | Value |
|:---|:---|
| **Spec ID** | **CONSTRUCTION-PARAM-001** |
| **Parent** | [`construction_invariants.md`](construction_invariants.md) · [`recovery_construction.md`](recovery_construction.md) |
| **Planner companion** | [`plan_construction_parametric_placement_v1.md`](plan_construction_parametric_placement_v1.md) (**PLAN-CONSTRUCTION-PARAM-001**) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` (UX) + `@planner` (authority) |
| **Status** | **Planner SIGNED** · **Designer PASS (qualified)** — implementation open |
| **Witness** | `debug_runs/construction_stage_live.json` → `construction_parametric_placement_001` |

**Goal:** Replace opaque “blueprint queue + Shift+LMB” with an intuitive **ghost-first** placement loop: anchor → adjust size/rotation → commit (or stage many ghosts → build all). Building **size is continuous (fractional)** and drives **weighted tile occupation**, **economy**, and **security/efficiency tradeoffs**.

---

## Executive summary

| Today | Target |
|:---|:---|
| Shift+LMB queues hidden “blueprints” | Explicit **Stage placements** toggle + visible staged-ghost list |
| Size = pick different catalog row / 1×1 vs 2×2 stub | **Continuous scale** on one catalog row |
| Footprint = integer `width × depth` rectangle | **Weighted footprint** — per-tile coverage ∈ [0, 1] |
| Rotation stored on ghost but dropped at commit | Rotation + scale **committed to sim** |
| Economy = `catalog_id` only | Economy = `catalog_id` × **scale curves** (production **and** expense) |

**North star preserved:** tool → intent → preview → validation → execute. Preview never mutates gameplay until commit.

---

## Player mental model

```text
Pick building type
  → Ghost appears on map (not built yet)
  → Adjust position, size (Shift+drag), rotation (Ctrl / R)
  → Either:
       A) Enter — build this ghost now          (Stage placements OFF)
       B) Add to staged list → Build all/selected (Stage placements ON)
```

Everything the player sees before Enter / Build is a **ghost**. Staged ghosts are ghosts with a checkbox — not a separate “blueprint” concept.

---

## Modes

### Stage placements OFF (default — fast single build)

| Action | Result |
|:---|:---|
| **LMB** on map | Anchor / move active ghost |
| **Shift + vertical mouse drag** | Change `scale_factor` (continuous) |
| **Ctrl + scroll** or **R** | Rotate ghost 90° CW |
| **X** | Mirror ghost on X (unchanged) |
| **Enter** | Commit current ghost if valid |
| **RMB** | Clear active ghost (keep tool) |
| **Esc** | Clear active ghost (keep tool) |

No pending list UI unless player opens it for history; single-ghost flow only.

### Stage placements ON (batch planning)

| Action | Result |
|:---|:---|
| **LMB** on valid ghost | **Add ghost to staged list** (snapshot: origin, scale, rotation, mirror, catalog) |
| **Shift + drag** | Still adjusts **active** ghost before adding |
| Staged list **checkbox** | Mark row **Approved** |
| **Build approved** button | Commit all approved staged ghosts |
| **Build all valid** button | Approve + commit all valid staged ghosts |
| **Clear unapproved** | Remove unchecked / invalid rows |
| **Enter** (with staged rows) | Same as **Build approved** |

Active ghost remains editable after adding to list; list rows are **snapshots** — editing active ghost does not mutate a staged row until re-added.

---

## HUD / tray (designer deliverables)

### Build tray additions

1. **Toggle:** `[ ] Stage placements` — persists per session (not save game v1).
2. **Active ghost readout** (when origin set):
   - Building name · **Scale 1.24×** · **Rot 90°** · footprint mass **3.7 tiles**
   - **Production ~118%** · **Expense ~124%** · **Risk index 1.31** (derived from curves below)
   - Validity badge: Valid / Risky / Invalid + first error line
3. **Staged list panel** (visible when toggle ON or `staged_count > 0`):
   - Columns: ☑ Approved · Label · Scale · Rot · Validity · ✕ remove
   - Footer buttons: **Build approved** · **Build all valid** · **Clear unapproved**
4. **Tool hints** (`tool_hints.rs`) — replace Shift+LMB queue text:

```text
LMB: anchor / move ghost
Shift+drag: scale size
Ctrl+scroll / R: rotate
Enter: place building
[Stage ON] LMB: add ghost to list · Build approved: commit checked
```

### Visual language

| Element | Treatment |
|:---|:---|
| Active ghost tiles | Outline + fill; **alpha = tile weight** (partial occupation visible) |
| Staged ghosts (not yet built) | Same palette, **25% desaturated**, dashed outer bound |
| Invalid overlap | Red tile weights where Σ weights > 1 |
| Scale drag | Vertical ghost “height” cue optional; primary feedback = readout + tile alpha |

Multiview / Round 4 ghost tokens remain compatible — parametric ghosts use the same corridor/site color tokens; scale does not change token hue, only footprint mass and label.

---

## Weighted footprint (fractional occupation)

### Definition

Replace binary 0/1-only occupation for **player-placed buildings** with **per-tile weight** `w ∈ [0, 1]`:

- **Base shape:** catalog `FootprintMatrix` (row-major 0/1 cells) at `scale_factor = 1.0`.
- **Placement params:** `origin`, `scale_factor` (fractional), `rotation_quarter_turns`, `mirror_x`.
- **Derived:** `WeightedFootprint` = map `IVec2 → f32` (sparse or dense within bounding box).

### Rasterization (authoritative for preview + commit)

For each world tile `T` in the axis-aligned bounds of the transformed base matrix:

1. Transform tile center to **local base-cell space** (apply inverse rotation, mirror, divide by `scale_factor`).
2. Compute **coverage** = area fraction of tile `T` inside occupied base cells via **4×4 subcell grid (16 samples)** — **planner-approved** deterministic algorithm ([`plan_construction_parametric_placement_v1.md`](plan_construction_parametric_placement_v1.md) § Planner decisions).
3. `weight(T) = coverage` clamped to [0, 1].

**Effective scale** (for economy):

```text
s_eff = sum(weight(T)) / sum(base_cell_count)   // base cells count as 1.0 at s=1
```

Clamp `s_eff` to per-building `[s_min, s_max]` from catalog (defaults 0.35 .. 2.75).

### Validation rules

| Rule | Gate |
|:---|:---|
| **Overlap** | For each tile: `existing_weight + new_weight ≤ 1.0` (ε = 0.001) |
| **Minimum mass** | `sum(new_weights) ≥ min_occupied_mass` (default 0.55 tiles) |
| **Terrain / network** | Existing stub scores; weighted **center of mass** for logistics sample |
| **allows_commit** | No errors; warnings allowed (Risky confidence) |

Preview and commit **must use the same rasterizer** (single function in `src/construction/`).

### Invariant update (proposed)

Extend [`construction_invariants.md`](construction_invariants.md) §15:

> Sim owns **weighted** tile occupation for parametric buildings (`weight ∈ [0,1]`). Integer `FootprintTiles` remains for legacy/stub paths until migrated.

---

## Economy scaling (production **and** expense)

Base stats from `assets/configs/buildings/*.json`. At commit, store `scale_factor` + derived `s_eff` on the site. Activation applies multipliers — **do not** duplicate catalog rows for size tiers.

### Curve parameters (catalog defaults + optional JSON overrides)

Per building (or per `BuildingFamily`), define exponents:

| Symbol | Meaning | Default |
|:---|:---|:---:|
| `k_prod` | Production throughput exponent | `0.90` |
| `k_exp` | Running expense (power, labour, consumables) exponent | `1.00` |
| `k_capex` | Construction cost / time exponent | `1.10` |
| `k_risk` | Single-site exposure exponent | `1.35` |
| `k_detect` | Sensor / sound footprint exponent | `0.70` |
| `fixed_overhead` | Per-site flat expense (currency/tick) | family table |

```text
prod_mult(s)   = s ^ k_prod
expense_mult(s)= s ^ k_exp
capex_mult(s)  = s ^ k_capex
risk_mult(s)   = s ^ k_risk
detect_mult(s) = s ^ k_detect

total_expense  = base_expense * expense_mult(s) + fixed_overhead
total_production = base_production * prod_mult(s)
```

**Design intent:** `k_prod < k_exp` at defaults → larger sites are **more efficient per unit output**; many small sites pay **fixed_overhead × count**.

### Security vs efficiency tradeoff

**Single-site exposure** (strike / detection / disaster seed):

```text
exposure_i = exposure_base(catalog) * risk_mult(s_i)
```

**Portfolio redundancy** (player spreads many small sites):

```text
P(catastrophic_hit) = 1 - Π_i (1 - p_strike * exposure_i)
```

| Strategy | Efficiency | Security |
|:---|:---|:---|
| One large hub | High throughput / expense | Low redundancy; one hit hurts more |
| Many small scattered | Lower (overhead + logistics) | Higher redundancy; lower per-site exposure |

Optional v1.1: **logistics distance penalty** when `fixed_overhead` alone is insufficient — defer to planner Phase 3 unless needed for playtest.

### HUD preview numbers

Show **before commit** (computed from active ghost + registry):

- Production, power, construction cost — each × respective multiplier
- **Risk index** = `risk_mult(s_eff)` (unitless, compare across placements)
- Tooltip explains tradeoff in one line: *“Larger: more output, higher exposure.”*

---

## Deprecations / input migration

| Removed | Replacement |
|:---|:---|
| Shift+LMB → queue blueprint | Stage placements ON + LMB add |
| Shift+Enter → approve all | **Build all valid** button |
| Hidden `PendingConstructionQueue` UX | **Staged ghosts** panel (same resource, new UI) |
| Integer-only ghost footprint for catalog buildings | `WeightedFootprint` derived from scale |

**Shift** is reserved for **scale drag** globally in building tool (conflict matrix updated in `build_tool_authority.rs`).

Roads / rail / zone paint: Shift semantics **unchanged** (path finalize / batch where already defined).

---

## Acceptance criteria (design)

1. Player can place one building with Enter without enabling staging.
2. Player can stage ≥3 ghosts, check 2, **Build approved** commits exactly 2.
3. Scale drag changes tile weights visibly (partial alpha) and readout continuously.
4. Two scaled ghosts cannot commit onto overlapping tiles (Σw > 1 blocked).
5. Committed site carries scale; operational production ≠ base catalog when `s_eff ≠ 1`.
6. Tool hints never mention “Shift+LMB queue blueprint”.

---

## Out of scope (v1)

- Non-uniform scale (different X vs Y) — single scalar `scale_factor` only
- Save/load of staged ghost list across sessions
- AI auto-approve staged rows
- Land value / housing market fields (still excluded per Round 3)

---

## Sign-off

| Role | Status | Date |
|:---|:---|:---|
| Product / user intent | **Captured** | 2026-05-26 |
| Designer UX | **PASS (qualified)** — [`construction_parametric_placement_design_v1.md`](construction_parametric_placement_design_v1.md) | 2026-05-26 |
| Planner authority | **PASS** ([`plan_construction_parametric_placement_v1.md`](plan_construction_parametric_placement_v1.md)) | 2026-05-26 |
| Coder implementation | **Unblocked** — **CONSTRUCTION-PARAM-CODER-001…006** | 2026-05-26 |
