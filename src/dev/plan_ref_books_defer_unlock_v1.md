# PLAN-REF-BOOKS-DEFER-UNLOCK-v1 — full phased unlock for v3 residual DEFER/PARTIAL

| Field | Value |
|:---|:---|
| **Id** | **PLAN-REF-BOOKS-DEFER-UNLOCK-v1** |
| **Status** | **OPEN (residual only) — 2026-11-24.** The 2026-11-16 re-grades are resolved: TAC-03, MOVE-05, MOVE-10, TAC-05, PCG-07 are ADOPTED with production callers (cleanup CQ-50/51/52/54/57g/58); RND-07 test-shaped verdict deleted (CQ-31), stays DEFER. Ledger: **125 ADOPTED · 3 PARTIAL (EXEC-05, RND-02, RND-12) · 9 DEFER**. |
| **Date** | 2026-11-15 |
| **Parents** | [`plan_ref_books_upgrade_v3.md`](plan_ref_books_upgrade_v3.md) (**PROGRAM CLOSE**) · [`plan_ref_books_upgrade_v2.md`](plan_ref_books_upgrade_v2.md) §8 · [`plan_deferral_registry_v1.md`](plan_deferral_registry_v1.md) |
| **Child charters** | [`plan_unit_member_v1.md`](plan_unit_member_v1.md) · [`plan_town_growth_v1.md`](plan_town_growth_v1.md) **TWN-PCG07** · [`plan_voxel_occupation_v1.md`](plan_voxel_occupation_v1.md) **OCC-P4…P6** · [`plan_dense_heat_wgsl_v1.md`](plan_dense_heat_wgsl_v1.md) (conditional) |
| **Rule** | Flip `adoption.tsv` DEFER→OPEN only when **trigger true** + owner charter signed. ADOPTED only with Accept evidence. No forever park. |

**Deferrals:** cite **DR-BOOKS-DEFER-UNLOCK** · **DR-BOOKS-UNIT-MEMBER** · **DR-BOOKS-TAC03-DENSE**.

---

## Review 2026-11-16 — reopened

> Operator code review of the Cursor wave ([`plan_cleanup_cursor_wave_v1.md`](plan_cleanup_cursor_wave_v1.md)). Re-grades below are mirrored in `ref_books_index/adoption.tsv`.

| Row | Was | Now | Why | Cleanup |
|:---|:---|:---|:---|:---|
| TAC-03 | ADOPTED | PARTIAL | GPU dispatch/readback missing; parity test tautological | CQ-50 |
| MOVE-10 | ADOPTED | UNWIRED | herd never seeded in play; soft cost unread | CQ-51/52 |
| MOVE-05 (UM-05) | ADOPTED | PARTIAL | member systems run on zero entities | CQ-54 |
| TAC-10 | ADOPTED | UNWIRED | `CombatAuthorityTier` doc-only | CQ-57n (delete) |
| PCG-07 | ADOPTED | UNWIRED | road skeleton no production caller | CQ-57g |
| TAC-05 | ADOPTED | PARTIAL | decide consumer real; fort path passes `None` (`fortification.rs:157`) | CQ-58 |
| RND-07 | DEFER (HOLD) | DEFER (no evidence) | "profile run" was a unit test on `Stage6VirtualizationFrame::default()` (all zeros); verdict code is test-shaped | CQ-31 |
| PCG-03 / TAC-05 "user world-look sign-off" | — | **retracted** (CQ-84, 2026-11-24) | no record exists, so no sign-off is claimed. PCG-03 stays DEFER until a sign-off is recorded in this plan; TAC-05 is ADOPTED on its production caller only. Erosion mass fixed (CQ-04, `9a26d6d5`) | CQ-04, CQ-84 |

**Close again only when:** each row above has a production caller `file:line` (or a real profile from a play run for RND-07). Every ADOPTED claim cites a production caller `file:line` (a test alone does not count).

## 0. Master DAG

```text
DR-BOOKS-DEFER-UNLOCK
├─ W0 ledger: DEC-04-GATE → LEARN-03-GATE
├─ W1 unit-member [DR-BOOKS-UNIT-MEMBER]
│    UM-00 → UM-05 (MOVE-05) → UM-06 (MOVE-06) → UM-07 (MOVE-07) → UM-10 (TAC-10)
├─ W2 town / world
│    TWN-OV1 → TWN-OV2 → TWN-GEN → TWN-PCG07 (PCG-07) → TWN-AI
│    PCG-03 ── HARD: user world sign-off
├─ W3 terrain / learning / MCTS
│    TAC-05 (PLAN-TERRAIN-ANALYSIS-v1) · LEARN-02 (after LEARN-03) · TAC-14 (T6×2)
├─ W4 render evidence: RND-06 · RND-07
├─ W5 late: MOVE-10 · TAC-03 [DR-BOOKS-TAC03-DENSE] · EXEC-05 LOCKED
└─ OCC productize (parallel): OCC-P4 → OCC-P5 → OCC-P6
```

**Coder pick when triggers fire:** W0 anytime → **UM-*** ‖ **TWN-*** ‖ **OCC-P*** → W3/W4 on evidence → MOVE-10 / TAC-03 last.

**Global rejects:** dual writers · test-shaped prod · Studio writing sim occupation · UI writing Formation.tile/posture · second `TransportTopology` writer · finishing TAC-03 by renaming sparse Jacobi.

---

## 1. TAC-03 plain language

Book wants **dense separable convolution** (fixed stencil, two 1-D passes, CPU≈GPU).

Shipped: `compute/heat_diffusion.rs` = **sparse Jacobi-like** (active cells only, variable neighbor divisor, decay+source blend). GPU **uploads** CPU rows — no WGSL diffusion kernel. `relax.rs` / `hybrid_fields.rs` are different operators.

PARTIAL ≠ broken. Unlock only via [`plan_dense_heat_wgsl_v1.md`](plan_dense_heat_wgsl_v1.md) when scale/GPU twin is measured.

---

## 2. Wave W0 — ledger hygiene

### DEC-04-GATE
| Field | Value |
|:---|:---|
| **Needs** | BT proposed **or** close-as-gate |
| **Files** | `adoption.tsv` · this § (no `src/` code) |
| **Do** | Read aig3 5.4; note “BT not adopted; concurrency/limits acknowledged” → ADOPTED-as-gate |
| **Don't** | Add BT runtime · claim code-adopted |
| **Accept** | `--row DEC-04` shows ADOPTED-as-gate **or** DEFER+trigger |
| **Unlocks** | **DEC-04** |

### LEARN-03-GATE
| Field | Value |
|:---|:---|
| **Needs** | Any learning **or** close-as-gate |
| **Do** | Acknowledge over-learning warnings (aig3 7.1.4–7.1.5) before LEARN-02 |
| **Accept** | `--row LEARN-03` → ADOPTED-as-gate |
| **Unlocks** | **LEARN-03** (gates LEARN-02) |

---

## 3. Wave W1 — unit-member → MOVE-05/06/07 → TAC-10

**Owner:** [`plan_unit_member_v1.md`](plan_unit_member_v1.md)  
**Trigger:** Front wants **visible unit positions** (not one Formation blob).  
**DR:** **DR-BOOKS-UNIT-MEMBER**

| Slice | Unlocks | Needs |
|:---|:---|:---|
| UM-00 | — | Trigger + charter signed |
| UM-05 | **MOVE-05** | UM-00 |
| UM-06 | **MOVE-06** | UM-05 |
| UM-07 | **MOVE-07** | UM-06 |
| UM-10 | **TAC-10** | UM-05/06/07 |

---

## 4. Wave W2 — town / world gen

### Town serial (amended)
**TWN-OV1 → TWN-OV2 → TWN-GEN → TWN-PCG07 → TWN-AI** — see [`plan_town_growth_v1.md`](plan_town_growth_v1.md).

| Slice | Unlocks | Needs |
|:---|:---|:---|
| TWN-OV1…GEN | — | Town program |
| **TWN-PCG07** | **PCG-07** | TWN-GEN green · road skeleton needed |
| TWN-AI | — | Prefer after PCG07 |

### PCG-03-EROSION
| Field | Value |
|:---|:---|
| **Needs** | **User signs new world look** (hard gate) |
| **Files** | `terrain/generation/hydrology/flow.rs` · `passes/p4_hydrology.rs` · golden fixtures |
| **Authority** | ⊚WorldGen hydrology ═▶ height/flow; D8 RO |
| **Do** | Thermal → hydraulic → D8; regenerate fixtures |
| **Don't** | Ship without sign-off · dual height writer |
| **Accept** | `--lib pcg03` + fixture update + sign-off in ledger |
| **Unlocks** | **PCG-03** |

---

## 5. Wave W3 — terrain / learning / MCTS

### TAC-05 (PLAN-TERRAIN-ANALYSIS-v1 — author when consumers named)
| Field | Value |
|:---|:---|
| **Needs** | Fort + faction AI name consumers |
| **Files** | `passes/p5_agent_overlay.rs` · `terrain/analysis/` · RO `fortification.rs` · `decide.rs` |
| **Authority** | ⊚TerrainAnalysisBook at gen; fort/AI RO |
| **Accept** | `--lib tac05` · fort RO consumer test |
| **Unlocks** | **TAC-05** |

### LEARN-02-NGRAM
| Field | Value |
|:---|:---|
| **Needs** | LEARN-03-GATE · **sim-truth vs derived** decision |
| **Files** | `ai/faction/perception.rs` · prefer derived frame (rebuild on hydrate) |
| **Accept** | `--lib learn02` |
| **Unlocks** | **LEARN-02** |

### TAC-14-MCTS
| Field | Value |
|:---|:---|
| **Needs** | Plain scoring fails **T6 goal twice** (recorded) |
| **Files** | `ai/faction/decide.rs` · seeded MCTS helper |
| **Accept** | `--lib tac14` · same seed → same choice |
| **Unlocks** | **TAC-14** |

---

## 6. Wave W4 — render evidence

| Id | Trigger | Files | Accept | Unlocks |
|:---|:---|:---|:---|:---|
| **RND-06-LOD-SEAM** | Capture shows LOD cracks | `render/layer_mesh.rs` · `pixel_world/terrain.rs` | before/after capture + `--lib rnd06` | **RND-06** |
| **RND-07-VT-RESIDENCY** | Profiler: atlas memory bottleneck | `stage6_virtualization.rs` · `profile_atlas_memory` | **DEFER** — the 2026-11-16 "profile" was a unit test on zeros; verdict code deleted (CQ-31). Needs a real play-run profile. (was: `rnd07_atlas_memory_profile` → gitignored JSON; `bottleneck_proven=false`) | **RND-07** |

---

## 7. Wave W5 — late / locked

| Id | Trigger | Notes | Unlocks |
|:---|:---|:---|:---|
| **MOVE-10-FLOCK** | Wildlife/crowd agents exist | **CLOSED** 2026-11-16 — [`plan_move10_wildlife_v1.md`](plan_move10_wildlife_v1.md); ecology-only ⛔ Formation.tile | **MOVE-10** |
| **TAC-03** | Dense scale / GPU twin | [`plan_dense_heat_wgsl_v1.md`](plan_dense_heat_wgsl_v1.md) | **TAC-03** |
| **EXEC-05** | — | Stay **PARTIAL LOCKED**; further LOD = consumer slices | — |

---

## 8. Parallel — occupation productize

[`plan_voxel_occupation_v1.md`](plan_voxel_occupation_v1.md) **OCC-P4 → OCC-P5 → OCC-P6** (flag-on / EV-Cx / DR close). X2-P0–P3 already ADOPTED; **default remains off** (CQ-07 save overlay wired; flags stay off until OCC-P4 product PASS).

---

## 9. Anti-rot

| Illegal | Required |
|:---|:---|
| “Still DEFER” with no owner | Cite this plan + charter |
| Cutting DEFER as polish | Trigger true + charter signed |
| TAC-03 without dense+WGSL | Keep PARTIAL |
| Town silent on PCG-07 | Use **TWN-PCG07** |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| 1.0.0 | 2026-11-15 | Initial unlock roadmap. |
| 1.1.0 | 2026-11-15 | Full waves + slice cards; child charters; master DAG; OCC parallel. |
| 1.2.0 | 2026-11-16 | **REOPENED** — review re-grades TAC-03/MOVE-10/MOVE-05/TAC-10/PCG-07/TAC-05/RND-07; see cleanup plan §5. |
