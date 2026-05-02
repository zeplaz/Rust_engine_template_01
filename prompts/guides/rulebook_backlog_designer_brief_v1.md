# Rulebook backlog — designer & lead brief `v1`

> **STATUS:** One-page **input** for designers and project leads before engineering runs [`terrain_paired_runbooks_queue_v1.md`](terrain_paired_runbooks_queue_v1.md) (**Q0–Q6**) or [`system_runbook_authoring_meta_v1.md`](system_runbook_authoring_meta_v1.md) to spawn new orchestrators. **No Rust** — decisions and scope only.

Version: `v1.0.1`

---

## 1. Why this exists

- **Terrain U-packs (U3–U7)** are treated as **stable** for execution purposes: see [`terrain_unification_runbook_v1.md`](terrain_unification_runbook_v1.md) §4 and [`../matrix/terrain_biome/material_unification_matrix_v1.md`](../matrix/terrain_biome/material_unification_matrix_v1.md) §10.
- **Moving “outside terrain”** means: **paired** work in orchestrator [**§8b**](terrain_unification_runbook_v1.md#8b-paired-runbooks-planned) (serialization **S**, preview **P**, chunk streaming **C**; assets **A** already has an orchestrator and packs), **and/or** per-system runbooks from meta [**§3 Target systems**](system_runbook_authoring_meta_v1.md#3-target-systems-and-pre-filled-paths) (power, weapons, buildings, navigation, factions, diplomacy, …).
- The **meta-runbook** and **paired queue** assume **front-loaded** product choices. Filling this brief first keeps new rulebooks aligned with **backlog** and avoids churn in step packs.

---

## 2. Read first (links)

| Doc | Role |
|:---|:---|
| [`terrain_unification_runbook_v1.md`](terrain_unification_runbook_v1.md) §8b | Paired pairs table (S, A, P, C) + sync rules |
| [`terrain_paired_runbooks_queue_v1.md`](terrain_paired_runbooks_queue_v1.md) | **Q0–Q6** authoring order, sync gate template |
| [`system_runbook_authoring_meta_v1.md`](system_runbook_authoring_meta_v1.md) §3 | Fixed paths for **new** system orchestrators; `ASK:` rows need humans before authoring |
| [`../matrix/terrain_biome/material_unification_matrix_v1.md`](../matrix/terrain_biome/material_unification_matrix_v1.md) | Terrain matrix — many **non–U-phase** rows may still be Pending / Partial (backlog beyond U7) |
| [`../matrix/terrain_biome/runbook/README.md`](../matrix/terrain_biome/runbook/README.md) | Step-pack index; **maintenance capsule** [`../legacy_runbooks/terrain/terrain_u_applied_maintenance_v1.md`](../legacy_runbooks/terrain/terrain_u_applied_maintenance_v1.md) |
| [`implementation_gap_hunt_runbook_v1.md`](implementation_gap_hunt_runbook_v1.md) | **Cadence sweep** for stubs / TODOs / placeholder behavior in `src/` — run before big planning or after merges (§3) |
| [`developer_reflective_brief_v1.plan.md`](developer_reflective_brief_v1.plan.md) | **Engineering** mirror (open in Cursor for plan/Mermaid preview): IA, maps, UI wireframes, decisions table · stub [`developer_reflective_brief_v1.md`](developer_reflective_brief_v1.md) |

---

## 3. Questions for designers & leads (answer in-doc or in your tracker)

Copy the table into a ticket or shared doc and assign **Owner** + **Target date**.

| # | Question | Notes |
|:---:|:---|:---|
| **Q1** | **Paired §8b priority:** In what order should we pursue **S** (serialization), **P** (preview / composite UI), **C** (chunk streaming / neighbors) for the *next* milestone? **A** (Bevy assets) is already committed — note any gaps. | Tie-break using [`terrain_paired_runbooks_queue_v1.md`](terrain_paired_runbooks_queue_v1.md) §“Suggested order”; record **explicit deferrals**. |
| **Q2** | **Terrain matrix backlog:** Which **Pending / Partial** rows in [`material_unification_matrix_v1.md`](../matrix/terrain_biome/material_unification_matrix_v1.md) (outside §10 U-status, if any) are **must-have** vs **later** for your release? List row ids or section refs. | Prevents rulebooks from drifting from product truth. |
| **Q3** | **New system orchestrators (meta §3):** Which **one** system should get the **next** orchestrator + `runbook/` after paired gaps are clear? **Power**, **Factions**, **Navigation**, etc. **Do not** start meta authoring for rows that are still **`ASK:`** without resolving anchors. | Confirm designer anchor docs exist for that system. |
| **Q4** | **Coupling / release train:** Any **hard** dependencies? Examples: preview must ship with saves; streaming with max world size; tools parity with [`world_assets_tools_rulebook_v1.md`](world_assets_tools_rulebook_v1.md). | Feeds **sync gate** tables in partner orchestrators (paired queue **Q5**). |
| **Q5** | **`ASK:` ownership:** Who converts open questions into matrix **`ASK:`** or designer checklist items vs who **resolves** them? | Keeps Q1 in paired queue honest (“defer with owner”). |
| **Q6** | **Non-goals:** What is **explicitly out of scope** for the upcoming rulebook wave (e.g. diplomacy, LLM tooling)? **Note:** terrain hydrology **G1** is **Applied**; further terrain work (e.g. pass 5 agent overlay **§71–74**) is matrix-owned, not an automatic G2 item. | Reduces accidental orchestrator sprawl. |

### 3.A Recorded answers (this cycle)

| Question | Recorded answer |
|:---|:---|
| **Q1 paired priority** | Pursue **S** (serialization), then **P** (preview / composite UI), then **C** (chunk streaming / neighbors). |
| **Q2 terrain backlog** | Fold decisions into [`material_unification_matrix_v1.md`](../matrix/terrain_biome/material_unification_matrix_v1.md) §§8, 9, 14, 16, 18. Prefer logical simulation behavior, code clarity, maintainability, and getting working code that can expand safely. |
| **Q3 next system orchestrator** | **UI tools** first, followed by **Navigation**, **Factions**, **Power**, then **Supply chain**. |
| **Q4 coupling** | Do not front-load a heavy dependency matrix yet; keep moving and monitor dependencies as they arise. |
| **Q5 `ASK:` ownership** | User / designer owns unresolved product answers. Agents should surface a **backlog queue snapshot** (**BQ-###**, §4) at the end of each cycle. |
| **Q6 non-goals / scope** | **LLM tooling is out of scope. Diplomacy is in scope.** |

---

## 4. Backlog queue — next cycle (BQ-###)

Use these ids in G3 **results** files and orchestrator notes. **Do not** parallel-track the same subject under informal open-work labels.

| BQ ID | Subject | Anchor |
|:---|:---|:---|
| **BQ-101** | **TileStorage diff / smooth transition** — what changed tile indices are sent, when, and how visuals stay reliable | [`material_unification_matrix_v1.md`](../matrix/terrain_biome/material_unification_matrix_v1.md) §16 |
| **BQ-102** | **Faction editor** — **G3A** §3 living map for slices not yet wired | G3A sub-pack |
| **BQ-103** | **Production HUD** — **G3B** §3 living map + **Next** polish: entity id → display names · optional F9 `dev_tools` gate · in-transit / pipeline row · richer bars/tooltips | G3B sub-pack § Next |
| **BQ-104** | **Diagnostics** — **G3C** §3 living map gaps | G3C sub-pack |
| **BQ-105** | **Runtime in-game UI** — **G3D** §3 living map gaps | G3D sub-pack |
| **BQ-106** | **Biome pack on/off UX** — menu placement and config key naming | Product / tools |
| **BQ-107** | **Schema-version strategy** for first serialization wave — per-file default vs registry when many schemas share lifecycle | Wave **S** + serialization matrix |
| **BQ-108** | **GUI invariants sign-off** — `TEMP-EGUI` labelling; desktop asset tools never render in-game UI | [`gui_runbook_v1.md`](gui_runbook_v1.md) §1 |
| **BQ-109** | **G2 power promotion** — player-visible behavior; save/schema; power parity matrix rows; owner (see §4.2) | G2 pack · gap-hunt §5 |
| **BQ-110** | **G4 serialization execution** — first domain DTO + register/load + ECS hydration test; hybrid matrix row; wave **S** | G4 pack · [`serialization_hybrid_migration_matrix_v1.md`](../matrix/serialization/serialization_hybrid_migration_matrix_v1.md) |
| **BQ-111** | **G5 subsystem promotion** — navigation vs damage vs manufacturing **per finding**; optional matrices; before `G5-SNN` | G5 pack living queue |
| **BQ-112** | **World map editor (M1–M5)** — FullReady→Editor routing, `TEMP-EGUI` palette, brushes, snapshot save/load vs **G4**/serialization matrix | [`map_editor_runbook_v1.md`](map_editor_runbook_v1.md) · [`map_editor_matrix_v1.md`](../matrix/map_editor/map_editor_matrix_v1.md) |

### 4.1 G3 sub-pack §3 → BQ crosswalk

| Sub-pack §3 living map | BQ ID |
|:---|:---|
| **G3A** Faction editor | **BQ-102** |
| **G3B** Production HUD | **BQ-103** |
| **G3C** Diagnostics | **BQ-104** |
| **G3D** Runtime in-game UI | **BQ-105** |

### 4.2 G2 promotion questions (verbatim — **BQ-109**)

Before writing **G2** atomic Rust steps (after **G2-S00**), answer gap-hunt §5:

| Question | Required answer |
|:---|:---|
| Player-visible behavior | What should steam leak / condenser, nuclear containment / decay heat, and renewable derates do to status, efficiency, alerts, and UI? |
| Save/schema impact | Are failure modes persisted as deterministic runtime state, derived from config, or replayed from events? |
| Matrix row | Which power parity rows flip when each failure mode lands? |
| Owner | Power runtime, damage, or production tools? |

---

## 5. After answers are recorded

1. Engineering resumes [`terrain_paired_runbooks_queue_v1.md`](terrain_paired_runbooks_queue_v1.md) at the last completed **Qn** for the chosen **pair**.
2. New **system** runbooks: follow [`system_runbook_authoring_meta_v1.md`](system_runbook_authoring_meta_v1.md) §7 **only** where §3 paths are fully specified (no blocking **`ASK:`**).
3. Update **bidirectional** §8 links: partner orchestrator ↔ [`terrain_unification_runbook_v1.md`](terrain_unification_runbook_v1.md) §8b, per paired-queue **Q6**.

---

## 6. Cross-links (consumers)

| Doc | How it uses this brief |
|:---|:---|
| [`../matrix/terrain_biome/runbook/README.md`](../matrix/terrain_biome/runbook/README.md) | Points here before expanding paired / cross-domain work |
| [`terrain_paired_runbooks_queue_v1.md`](terrain_paired_runbooks_queue_v1.md) | Optional gate before **Q0** for a new pair |
| [`system_runbook_authoring_meta_v1.md`](system_runbook_authoring_meta_v1.md) | Human prep before invoking meta for **`ASK:`**-free rows |
| [`implementation_gap_hunt_runbook_v1.md`](implementation_gap_hunt_runbook_v1.md) | Cadence stub/placeholder sweep in `src/` (§2–§4); triage before runbook waves |
| [`gap_remediation_runbook_v1.md`](gap_remediation_runbook_v1.md) | **G1** hydrology **Applied** (capsule + legacy full); active **G3** GUI; **G2–G5** parked until §4 **BQ-109+** |
| [`../matrix/gap_remediation/runbook/g3_execution_cycle_v1.md`](../matrix/gap_remediation/runbook/g3_execution_cycle_v1.md) | **G3** step order: GUI and backlog orchestrators alternate; cycle **results** use **BQ-###** only |
| [`developer_reflective_brief_v1.plan.md`](developer_reflective_brief_v1.plan.md) | Engineering fills IA + diagrams + decisions; ties **BQ-###** back into this brief |
