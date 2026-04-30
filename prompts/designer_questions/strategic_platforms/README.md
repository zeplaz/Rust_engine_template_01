# Designer Q — strategic platforms, munitions & EW

**Paired matrix:** `prompts/matrix/strategic_platforms/strategic_platforms_matrix_v1.md`

> **Prompt use:** Treat rows as spec until `rg` finds types · cite planned module paths only if listed in matrix/docs · `ASK:` for classification gaps (e.g. EW scope).

**Scope:** Technical design for **units and layers** that interact with logistics + terrain + LOD: buildings, terrain, ships, aircraft, UAS, missiles, guns, **radar**, **jamming**, C2-ish latencies — **not** marketing or release planning.

| File / folder | Role |
|:---|:---|
| **`spec/README.md`** | **Start here** — index `00`–`05` (taxonomy, EW, munitions, cargo, phases) |
| `spec/00_scope.md` … `spec/05_*.md` | Structured stubs + integration grid |
| `platforms_ew_munitions_v1.md` | Platform classes, sensors, EW, coupling to chunks/LOD |
| `phased_engine_delivery_v1.md` | **Engine build phases** (what to implement in what order) |
| **`implementation_questions_v1.md`** | Checklists, contracts, test hooks |

**Cross:** `terrain_world/` (theatre, cities, ports), `navigation/` (movement), `production_economy/` (supply, damage), `tools_ui/` (inspectors).
