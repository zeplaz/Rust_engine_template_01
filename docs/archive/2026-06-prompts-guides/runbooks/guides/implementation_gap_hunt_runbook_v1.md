# Implementation gap hunt (stub / placeholder sweep) `v1`

> **STATUS:** Process doc — **not** a terrain-only runbook. Use on a **cadence** (see §3) to find code paths that compile but **do not carry intended behavior**, then triage into tickets, matrices, or designer questions before large refactors.

Version: `v1.0.0`  
Audience: tech leads, agents doing hygiene passes; pairs with [`rulebook_backlog_designer_brief_v1.md`](rulebook_backlog_designer_brief_v1.md) when gaps need product answers.

---

## 1. What counts as a “gap”

| Signal | Examples | Severity hint |
|:---|:---|:---:|
| Explicit TODO / FIXME / stub comments | `// TODO:`, `placeholder`, `simplified`, `In a real implementation` | **High** if on gameplay / save / generation hot paths |
| Empty or no-op bodies with misleading names | `fn foo_placeholder(...) {}`, loops that never mutate state | **High** |
| `unimplemented!` / `todo!` in non-test code | Calls that will panic if hit | **Critical** if reachable |
| Doc-only “implementation” | Comments describing steps 1–2–3 with no code | **High** |
| Test-only or feature-gated stubs | Document in matrix; still track | **Medium** |

**Out of scope for this pass:** intentional `deprecated` shims, generated glue, and third-party `TODO` in `target/` or vendored trees.

---

## 2. How to hunt (repeatable commands)

Run from repo root (PowerShell or bash). Refresh patterns when the team adds new euphemisms.

**A. Comment / prose markers (Rust)**

```text
rg -n "TODO:|FIXME:|unimplemented!|todo!|placeholder|simplified placeholder|In a real implementation|not implemented|stub only" --glob "*.rs" src
```

**B. Python / tools**

```text
rg -n "TODO:|FIXME:|placeholder|NotImplemented" --glob "*.{py,md}" src assets prompts
```

**C. Bevy / ECS “empty” systems (spot-check)**

```text
rg -n "fn .*\(_.*\)\s*\{\s*\}" --glob "*.rs" src
```

**D. Pair with compiler hygiene (optional)**

```text
cargo check -p proc_A_dine01 2>&1
```

Treat **`dead_code`** on large public modules as a *secondary* list — not always a gap, but worth scanning.

**E. Baseline file list (examples fixed or still tracked)**

Maintain a short rolling table in your tracker (not required in this doc). Example entries that prompted this runbook:

- `src/terrain/generation/world_generator_enhanced.rs` — **G1 closed:** rivers/lakes from `compute_hydrology_world` (D8 + priority-flood); markers remain visual-only.
- `src/bevysubengines/world_generator_plugin.rs` — legacy JSON export path; hydrology explicitly out-of-scope (see `world_assets_tools_rulebook_v1.md` §1).
- `src/terrain/generation/passes/p4_hydrology.rs`, `p5_agent_overlay.rs` — **p4 Applied** for hydrology tags; p5 still stub / matrix Partial.

---

## 3. Cadence (subsystem in the iteration process)

| When | Action |
|:---|:---|
| **Each milestone / release candidate** | Run §2A–B; export hits to a spreadsheet or issue label **`impl-gap`**. |
| **Monthly** | Full §2 sweep + spot-check §2C; reconcile with [`material_unification_matrix_v1.md`](../matrix/terrain_biome/material_unification_matrix_v1.md) / domain matrices for **Pending** rows. |
| **Before spawning new orchestrators** | Run §2; ensure gaps in that domain are listed or deferred with owner — aligns with [`system_runbook_authoring_meta_v1.md`](system_runbook_authoring_meta_v1.md) “front-load decisions”. |
| **After large merges** | Quick §2A only (15 min). |

Optional **CI** job: run `rg` with `--quiet` and fail if **new** matches appear under `src/` compared to a committed allowlist file — only adopt if noise is low.

---

## 4. Triage → plan (per hit)

For each finding, record:

1. **Path + symbol** (file, fn, approx lines).  
2. **Reachability:** Startup / gameplay / editor-only / debug-only?  
3. **Correctness impact:** Silent wrong output, panic, or missing feature?  
4. **Owner:** gameplay, tools, terrain, production, etc.  
5. **Resolution type:** code fix, `ASK:` in matrix, designer question doc, or **won’t fix** (with reason).

**Plan template (one paragraph):** “Replace X with Y behavior, touching modules A/B, verify with test or manual step Z.”

### 4.5 Routing to remediation

Once triaged, route each code gap to [`gap_remediation_runbook_v1.md`](gap_remediation_runbook_v1.md). If a row does not fit these buckets, add `ASK:` to the ticket rather than creating a new phase in prose only.

| Phase | Route | Typical anchors |
|:---:|:---|:---|
| **G1** | Hydrology / world-generation stubs | **Applied** — `p4_hydrology`, `world_generator_enhanced`, `world_generator_plugin` comment boundary; terrain matrix §3 pass 4 |
| **G2** | Power placeholders | `failure_modes.rs`, `power_systems.rs`, power parity matrix §8 |
| **G3** | **Active** — GUI TODOs | `faction_tools_ui.rs`, `in_game_hud.rs`, `diagnostics_ui.rs`, UI boundary guide |
| **G4** | Serialization stubs | `systems/production/serialization.rs`, serialization hybrid matrix, paired runbook **S** |
| **G5** | Navigation, damage, manufacturing placeholders | `potental_feild_nav.rs`, `damage_system.rs`, `manufacturing_plugin.rs` |

**Applied / archive packs** (hydrology G1 full text, terrain U maintenance): [`../legacy_runbooks/README.md`](../legacy_runbooks/README.md).

---

## 5. Questions for designers / leads (when product truth is missing)

Use these when §4.3 is unclear; paste answers next to the gap ticket.

| # | Question |
|:---:|:---|
| Q1 | What should the player **see** or **persist** when this code path runs? |
| Q2 | Is wrong silent output acceptable for this milestone, or must we block ship? |
| Q3 | Does this interact with **save format**, **multiplayer**, or **mods** — i.e. stable schema? |
| Q4 | What existing **matrix row** or **designer_questions** doc should hold the spec until code lands? |

If Q1–Q4 are answered, engineering can schedule work without expanding scope in prose-only runbooks.

---

## 6. Cross-links

| Doc | Role |
|:---|:---|
| [`rulebook_backlog_designer_brief_v1.md`](rulebook_backlog_designer_brief_v1.md) | Priority / scope before meta + paired §8b |
| [`system_runbook_authoring_meta_v1.md`](system_runbook_authoring_meta_v1.md) | Authoring new execution runbooks once gaps are triaged |
| [`gap_remediation_runbook_v1.md`](gap_remediation_runbook_v1.md) | Execution destination for triaged gaps (G1-G5) |
| [`terrain_unification_runbook_v1.md`](terrain_unification_runbook_v1.md) | Terrain-specific execution (distinct from this hygiene process) |

---

## 7. Agent prompt fragment

> Run [`implementation_gap_hunt_runbook_v1.md`](implementation_gap_hunt_runbook_v1.md) §2 on `src/`. For each hit in §1, fill §4 triage fields. Do not delete legacy comments until behavior is replaced or ticketed. Propose **one** consolidated plan per subsystem and surface **`ASK:`** items to the human when §5 Q1–Q4 cannot be answered from existing designer specs.
