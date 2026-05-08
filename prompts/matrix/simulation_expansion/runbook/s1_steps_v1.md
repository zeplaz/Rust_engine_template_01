# S1 — Chunk scheduler / dirty regions — step pack `v1`

> **Parent:** [`../../guides/chunk_scheduler_runbook_v1.md`](../../guides/chunk_scheduler_runbook_v1.md)  
> **Parallel with:** S0 ([`s0_steps_v1.md`](s0_steps_v1.md))

Version: `v1.0.1`  
**STATUS:** **Verified** (2026-05-06)

---

## S1-S01 — Baseline: document existing chunk dirty pattern (terrain)

| Field | Content |
|:---|:---|
| **Goal** | Chunk scheduler runbook §4 cites a **concrete** in-repo precedent: `ChunkDirty` / `ChunkDependency` on material chunks. |
| **Anchor reads** | [`chunk_scheduler_runbook_v1.md`](../../guides/chunk_scheduler_runbook_v1.md) §4; `src/terrain/material/dependency.rs`; `src/systems/terrain/material_plugin.rs` (`rebuild_dirty_chunks`) |
| **Touch** | `prompts/guides/chunk_scheduler_runbook_v1.md` — add short bullet under §4 with file paths (or cross-link to this step). |
| **Verify** | Markdown only. |
| **DoD** | New contributors can find dirty invalidation without reading the whole plugin. |

**Result (2026-05-06):** §4 of [`chunk_scheduler_runbook_v1.md`](../../guides/chunk_scheduler_runbook_v1.md) links paths + S1 step pack.

---

## S1-S02 — Inventory: what else should be chunk-scoped (gaps)

| Field | Content |
|:---|:---|
| **Goal** | Table of **candidate** systems (weather, ecology, petroleum fields) vs **current** chunk coupling; mark Implemented / Missing. |
| **Anchor reads** | [`simulation_expansion_orchestrator_v1.md`](../../guides/simulation_expansion_orchestrator_v1.md) §3; `src/terrain/world.rs`; grep `Chunk` in `src/systems` |
| **Touch** | [`chunk_scheduler_gap_table_v1.md`](../chunk_scheduler_gap_table_v1.md) |
| **Verify** | `cargo check -p proc_A_dine01` (unchanged). |
| **DoD** | One committed doc lists gaps for S2 weather + later domains. |

**Result (2026-05-06):** Gap table committed (`v1.0.0`); `rg Chunk` survey on `src/systems` + `src/terrain` reflected in §2.

---

## S1-S03 — Persistence policy stub per chunk-owned state (design only)

| Field | Content |
|:---|:---|
| **Goal** | For each row in S1-S02 **Implemented**, one-line **persist / recompute** note per chunk scheduler runbook §5. |
| **Anchor reads** | [`chunk_scheduler_runbook_v1.md`](../../guides/chunk_scheduler_runbook_v1.md) §5 |
| **Touch** | Same gap table / runbook appendix as S1-S02 |
| **Verify** | Review with matrix owner. |
| **DoD** | No code change required; saves DB design arguments for later implementation PRs. |

**Result (2026-05-06):** Gap table §2 includes **Persist** / **Recompute** columns for terrain, overlay, nav, transport, damage, weather, ecology, petroleum.

---

## Document history

- **2026-05-06:** Initial S1 step pack (parallel with S0).
- **2026-05-06:** `v1.0.1` — S1-S01–S03 executed; gap table added.
