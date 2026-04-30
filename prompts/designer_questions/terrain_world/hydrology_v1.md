# Designer Q — hydrology: worldgen vs runtime `v1`

---

## Locked intent

### World generation

- **High detail** — full hydrology pass (flow accumulation, basins, rivers/lakes) as already committed in design direction.

### In-game (runtime)

- Water should still **move / react** in a **lighter** model most of the time.
- **Player / war actions** must matter:
  - Breach **dam**, **earthworks**, **redirect** flow, **flood** floodplain, **drain** lake (strategic — e.g. cooling water denial).
- **Deep hydraulic sim** runs **only when triggered** (event-driven activation), not every tick globally.

---

## Sub-questions (⏳) — partially superseded by **Answers & implementation cues** below

1. **Trigger list**: damage threshold, construction complete, explosive, scripted scenario?
2. **Coupling to power**: reservoir head, intake flow — same event graph as electrical? 📎
3. **Persistence**: store “pending hydrology dirty region” in save?
4. **MP**: who simulates active flood — server only?

---

## Repo touchpoints

- Biome / moisture: `src/terrain/biome.rs`, ecology `src/terrain/ecology.rs`
- Gen: `src/terrain/generation/world_generator_enhanced.rs`, `world_generator_plugin.rs`

Runtime hydrology **not implemented** — spec ahead of code.

---

## Answers & implementation cues (append-only)

**Round:** 2026-04-27 · **For:** LLM + engineers

### Trigger list (deep / event-driven sim)

Activate **high-fidelity** hydraulics when **any** of:

- **Construction / destruction** completes (dam, canal, culvert, pump, intake).
- **Damage** crosses thresholds; **critical infrastructure failure** flags.
- **Designer / scenario scripts** fire.
- **Upstream overflow** or pressure propagation forces downstream cell update.

Lightweight **background** flow can still tick cheaply between events; **heavy** solve is **localized + time-sliced**.

### Power × water coupling

| Topic | Decision |
|:---|:---|
| Shared vs split | **Many events hit both**; **not all** water events need electrical coupling (e.g. passive flood) and **not all** grid events need hydraulic graphs. |
| Design direction | Prefer **shared event bus** + **subsystem tags** / **component bundles** so entities can be `HydraulicNode`, `ElectricalNode`, or **both**, without one mega-type. |
| 📎 Refine | Which **invariants** are cross-domain (e.g. pump power ↔ head) vs decoupled? |

### Persistence

- Mark **hydrology dirty regions** in saves when needed for **reload accuracy**; allow **culling** of stale scratch state that can be recomputed from terrain + buildings.

### Multiplayer / client

| Topic | Decision |
|:---|:---|
| Flood / flow presentation | Run **basic simulation on both** client (preview / interpolation) and **server (authority)**. |
| Parity | After significant events, run **light parity checks** (volume balance, key height thresholds, or sample probes) — **confirm** client in sync or snap to server. |

### 📎 Sub-questions for designer

1. Acceptable **client pre-roll** of flood extent before server ack?
2. **Rollback** if client predicted wrong (gameplay vs purely visual)?
3. Single **FluidDomain** resource vs per-chunk **WaterChunk** components?

### Implementation hints

- Tag components: `#[require(HydraulicAffected)]` pattern or bitflags on `InfrastructureKind`.
- Tie into trigger system from `prompts/designer_questions/production_economy/power_damage_ui_persistence_v1.md` (damage events).
