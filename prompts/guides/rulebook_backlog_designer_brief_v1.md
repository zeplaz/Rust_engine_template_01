# Rulebook backlog — designer & lead brief `v1`

> **STATUS:** One-page **input** for designers and project leads before engineering runs [`terrain_paired_runbooks_queue_v1.md`](terrain_paired_runbooks_queue_v1.md) (**Q0–Q6**) or [`system_runbook_authoring_meta_v1.md`](system_runbook_authoring_meta_v1.md) to spawn new orchestrators. **No Rust** — decisions and scope only.

Version: `v1.0.0`

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
| [`../matrix/terrain_biome/runbook/README.md`](../matrix/terrain_biome/runbook/README.md) | Step-pack index (maintenance / audit entry) |
| [`implementation_gap_hunt_runbook_v1.md`](implementation_gap_hunt_runbook_v1.md) | **Cadence sweep** for stubs / TODOs / placeholder behavior in `src/` — run before big planning or after merges (§3) |

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
| **Q5 `ASK:` ownership** | User / designer owns unresolved product answers. Agents should surface a clean TODO list at the end of each cycle. |
| **Q6 non-goals / scope** | **LLM tooling is out of scope. Diplomacy is in scope.** |

---

## 4. Open follow-ups bubbled to next cycle

These are designer / lead TODOs that should be answered before the next implementation wave turns them into Rust steps.

- Define the **Diff updates** smooth-transition contract in [`material_unification_matrix_v1.md`](../matrix/terrain_biome/material_unification_matrix_v1.md) §16: what changed tile indices should be sent to `TileStorage`, when, and how visual transitions stay reliable.
- Define each GUI surface's **data source** before Rust work: faction editor, production HUD, diagnostics, and runtime in-game UI.
- Decide the **biome pack on/off UX**: menu placement and config key naming for enabling/disabling loaded biome packs.
- Choose **schema-version strategy** during the first serialization wave: per-file by default, or registry when many schemas share a lifecycle.
- Confirm GUI invariants for [`gui_runbook_v1.md`](gui_runbook_v1.md): egui placeholders must be marked `TEMP-EGUI`; desktop asset tools never cross into in-game UI.

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
| [`gap_remediation_runbook_v1.md`](gap_remediation_runbook_v1.md) | Executes triaged gaps by phase (**G1** hydrology first; **G2-G5** require owner / matrix answers before promotion) |
