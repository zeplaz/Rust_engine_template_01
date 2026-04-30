# Designer Q — pathfinding `v1` (hybrid + hierarchical)

---

## Locked intent

- **Hybrid + hierarchical** — not a single algorithm worldwide.
- **Hand off** between methods by context:
  - **Road / rail bound** → network graph (see rail note below).
  - **Dead end / off-road** → **flow field** or coarse field, then **fine** path polish.
- **Goals**: reliable, **fast glitch detection**, **recalculate** or fall back to lower-cost method quickly; **smooth** motion; **offload** to cheaper methods as soon as safe.
- **LOD coupling**: coarse path at low LOD; refine when entity or pocket promotes — see `prompts/designer_questions/terrain_world/simulation_lod_v1.md`.

---

## Rail vs road

- **Similar graph abstraction**, **different rules**: no passing lanes, signals, blocks, single-track constraints → **separate graph type** or edge metadata on shared `TransportGraph` 📎.
- Likely **different optimization** (fewer branches, more discrete time steps).

---

## Sub-questions (⏳) — see **Answers** below for LOD + authority

1. Flow-field grid resolution vs road graph scale factor?
2. Who owns **dynamic obstacles** (combat, debris) — tile mask, graph edge weight, or both?
3. Server authoritative path vs client propose / server validate? (MP)

---

## Repo touchpoints

- Stub / future: `src/systems/navigation/potental_feild_nav.rs` (typo in filename — fix when implementing)
- Road motion: `src/systems/navigation/road_vehicles_motion.rs`

---

## Answers & implementation cues (append-only)

**Round:** 2026-04-27 · **For:** LLM + engineers

| Topic | Decision |
|:---|:---|
| Flow field vs road graph scale | **Both** participate in **LOD stack**: coarse **field/grid** at low LOD; **network graph** at road-bound LOD; handoff **up/down** the chain as entity’s LOD or context changes (ties `terrain_world/simulation_lod_v1.md` + `terrain_world/chunks_streaming_v1.md`). |
| Resolution | No single global ratio — **tiered** resolutions per LOD level (📎 numeric defaults after profiling). |
| Authority | **Primary:** server-authoritative paths for anything affecting **economy / combat / claims**. |
| Client propose | **Secondary / optional:** client may **propose** path for UX (preview, local units); server **validates or replaces**; reconciliation event to client. |

### 📎 Sub-questions

1. Validation cost cap per tick — max proposed paths per client?
2. Cheat surface: ignore client proposals entirely until late alpha?
3. Rail **block occupancy** validated only at server LOD ≥ N?

### Implementation hints

- Mirror render vs sim split: **RoadPathLod0**, **CorridorPathLod1**, **RegionIntentLod2** as data types or tags on the same logical route request.
