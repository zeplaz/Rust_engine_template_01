# Strategic platforms & EW — boundary matrix `v1`

**STATUS:** ⏳ **Spec + phased delivery** — minimal or no gameplay code yet.  
**Paired designer folder:** `prompts/designer_questions/strategic_platforms/` · **Structured spec:** `strategic_platforms/spec/README.md`.

> **Prompt use:** Open paired folder README first · spec is ahead of most code — verify any “exists” claim in `src/` · short cites only · `ASK:` for doctrine/perf caps.

**Purpose:** Classify **warfighting layers** (munitions, sensors, EW, varied platforms) for **Serializable / ECSRuntime / ToolsUI** splits and dependency order — same convention as `matrix/repo/`.

---

## Domain rows (draft — extend as code lands)

| Domain | Serializable (configs, catalogs) | ECSRuntime (mutable state) | ToolsUI (egui / debug) | Upstream deps |
|:---|:---|:---|:---|:---|
| **Structures / build** | Blueprints, footprint, tier | `Transform`, damage, power attach | Build inspector, grid overlay | `production_economy`, `terrain_world` |
| **Terrain / modifiers** | Height/biome overrides | Chunk tiles, hydrology dirty | Gen + brush tools | `terrain_world` |
| **Surface mobility** | Vehicle configs | Motion, cargo, damage | Vehicle inspector | `navigation` |
| **Rail** | Line capacity, signals 📎 | Block occupancy, train state | Graph debugger | `navigation` |
| **Naval** | Hull class, magazine 📎 | Buoyancy stub, sea state coupling | Port + track overlay | `navigation`, hydrology |
| **Fixed-wing / rotary** | Airframe, stores 📎 | Flight LOD (simplified vs focus) | Sortie / BRA panel 📎 | `simulation_lod`, `navigation` |
| **UAS / loitering** | Orbit params, link budget 📎 | Battery, LOS, C2 latency 📎 | UAS status strip | Same as air + comms stub |
| **Missiles / munitions** | Seeker law, NEZ 📎 | Seeker state, flight segment | Track debugger | `simulation_lod` pockets |
| **Radar / ESM** | Waveform, scan rate 📎 | Tracks, SNR abstraction | Emitter / track list | LOD + **EW** |
| **Jamming / EA** | ERP, notch 📎 | Burn-through, deception state 📎 | EW battlespace panel | Radar domain |
| **Cyber / security layer** *(optional stub)* | Policy enums | Compromise / downtime | Security ops 📎 | Power + comms graph 📎 |

📎 = schema not frozen — **designer_questions/strategic_platforms/** owns intent; **implementation_questions** owns phase gates.

---

## Code already in repo (surface vehicles — do not ignore)

- **`entities/vehicles/`** — `config`, `runtime` (`RoadVehicle` + damage), `tools_ui` (`RoadVehicleToolsUiPlugin`), `components`, `states`.
- **Wiring:** `engine_with_worldgen.rs` adds **`RoadVehicleToolsUiPlugin`** alongside **`Production*`** and **`WorldGenToolsPlugin`**. Strategic **theatre** docs must spell how road logistics upgrades into C2 / fires — this is **not** optional complexity; it is existing code path.

---

## Hard rules

- **Same governing equations** across LOD tiers (per `simulation_lod_v1.md`); simplify time step / aggregation — do not fork “arcade” off-world rules for missiles.
- **Server authority** for tracks, firing solutions, jam outcomes (align multiplayer legacy §A3).
- **Serialization:** platform configs and magazine loads are **Serializable**; in-flight state **snapshot** rules 📎 in `serialization/` matrix.

---

## Agent workflow

1. Read `platforms_ew_munitions_v1.md` + `phased_engine_delivery_v1.md`.
2. Read `implementation_questions_v1.md` before adding types.
3. Register new **production/manufacturing domains** in `ProductionManifest` if platforms consume industrial output.
4. Update **this table** when a domain gets first `struct` in `src/`.
