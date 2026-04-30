# Designer Q — chunks, streaming, simulation partition `v1`

---

## Provisional numbers (📎 confirm with project owner)

| Param | Proposal | Status |
|:---|:---|:---:|
| Chunk size | **256 × 256** tiles | ⏳ provisional |
| Load pattern | **Center + 8 Moore neighbors** → **9 chunks** typical high-attention set; **ring depth / width varies** by **interest orb** priority (see `simulation_lod_v1.md`) | ✅ direction |
| LOD alignment | Chunk loading **must match** **3** sim LOD layers at start | ✅ direction |

**Note:** Typo guard — both dimensions **256** (not 265).

---

## Simulation vs rendering

- **Rendering** follows camera + UI; **simulation** should follow **server interest sets** (players, factions, strikes — see `simulation_lod_v1.md`).
- **Chunk** is the natural **unit of load/save streaming** and **parallel job scheduling** (one chunk per task, 📎 lock SIMD later).
- **Boundary effects**: water flow, power islands, pathfinding — need **ghost cells** or **shared edge buffers** ⏳ (design TBD).

---

## Sub-questions (⏳) — many resolved under **Answers** below; open items → **Tradeoffs** section

1. ~~Chunk world coordinates~~ → partial in **World coordinates** + **Tradeoffs #1** below.
2. ~~Unloading / queue~~ → **Load / save / queues** table below.
3. ~~MP subscription~~ → **Faction vs client** row below (per-client; AI limited).
4. Legacy monolith **A1.a–c** — keep in sync when chunk defaults finalize.

---

## Repo touchpoints (verify, don’t guess)

- World data: `src/bevysubengines/world_generator_plugin.rs` (`WorldData`)
- Generation: `src/terrain/generation/*`

No chunk loader implemented yet — this file is **spec ahead of code**.

---

## Answers & implementation cues (append-only)

**Round:** 2026-04-27 · **For:** LLM + engineers

### Chunk grid & rings (aligned with LOD)

| Topic | Decision |
|:---|:---|
| Sim LOD layers (start) | **3 layers** — must **match** chunk ring / sim detail policy (see `simulation_lod_v1.md`; table there should stay consistent with this file). |
| Default active set | **9 chunks** usual case: **center + 8 Moore neighbors** on chunk grid. |
| Ring depth | Not single global only: **priority-based rings** — *orbs of interest* (multiple foci) use **different ring extents** by **need** (player view, bookmark, strike pocket, etc.). |
| Moving foci | Server tracks **multiple moving centers** (player positions, pinned POIs, promoted LOD pockets). Each focus has **radius / priority** → drives which chunks stay **hot**. |

### Authority & subscriptions

| Topic | Decision |
|:---|:---|
| Simulation | **Server interest set** owns which chunks are **sim-active** and at what LOD. |
| Rendering | **Client** follows **camera + UI** (local presentation only). |
| Player location | **Server tracks** player-associated chunk(s) so nearby world stays **active** even if client throttles render. |
| Bookmarks | Players get a **limited** count of **pinned locations**; each pin has **radius tiers** (constants **tunable** for design/balance). |
| Faction vs client | Chunk subscription is **per-client / per-interest-owner**, not “whole faction” as primary key. **AI factions**: **limited** sim focus — coarse updates unless engaged. |

### Load / save / queues

| Topic | Decision |
|:---|:---|
| Unload / serialize | **Delay** work based on **current load**, **remaining budget**, **queue depth** — targets are **mid defaults** but **adjustable** (design tuning); optionally **user-defined** caps; could **scale** with available **compute budget** (“power” as metaphor or literal in-game — 📎 clarify: CPU budget vs fiction). |
| Persistence hygiene | Persist **dirty chunk state** needed for **accurate reload**; **culling** of ephemeral/low-value dirt is OK where it doesn’t harm fidelity (📎 define “safe” cull rules per subsystem). |

### Boundaries (ghost / coupling)

| Topic | Decision |
|:---|:---|
| Cross-chunk | **Power**, **water**, **pathfinding** must **respect chunk boundaries** — expect **ghost band** or **shared edge buffers** so flows & routes don’t break at seams. |
| Hybrid approach | **Goal:** correctness-first at boundaries; performance via LOD *inside* chunk, not by ignoring neighbors. |

### World coordinates

| Topic | Decision |
|:---|:---|
| Model | **Likely dynamic origin** (floating origin / shifting reference) **while** simulation remains aligned to a **fixed conceptual grid** (chunk indices + local offset). Exact math 📎 — see sub-questions. |

### 📎 Tradeoffs / questions for designer

1. **Dynamic origin**: `i64` global tile index vs **chunk_id + u16 local** only — which is source of truth on wire?
2. Bookmark **max count** and **default radii** (in chunks) per tier?
3. When **two orbs** overlap — merge chunk sets or priority-winner per chunk?
4. **Power budget** knob: pure UX setting vs in-fiction “grid compute” resource?

### Implementation hints

- Data structure sketch: `InterestOrb { center_chunk: IVec2, radius_chunks: u8, priority: u8, kind: Player|Bookmark|LodPocket|Script }` → `HashSet<ChunkId>` merged server-side.
- Serialize queue: back-pressure metrics exposed to **egui** debug HUD (see `prompts/designer_questions/tools_ui/debug_perf_ui_split_v1.md`).
