# Construction recovery — actionable todos



**Live boards (authoritative status):**



| Board | File | Rows | Status |

|-------|------|------|--------|

| P0–P4 closure | [`construction_live_todos.rs`](construction_live_todos.rs) | 18× `BUILD-P*` | **Done** |

| Finish / migration | [`construction_finish_todos.rs`](construction_finish_todos.rs) | 8× `FINISH-BUILD-*` | **Done** |

| **Phase 2** | [`construction_phase2_todos.rs`](construction_phase2_todos.rs) | 20× `PHASE2-BUILD-*` | **Active** (close P6→P9 first) |
| **Round 2 (feel)** | [`construction_round2_todos.rs`](construction_round2_todos.rs) | 15× `CONSTRUCTION-R2-*` | **Done** |
| **Operational green** | [`construction_operational_todos.rs`](construction_operational_todos.rs) | 8× `CONSTRUCTION-OP-*` | **Active** (after P6–P8) |
| **Round 3** | [`construction_round3_todos.rs`](construction_round3_todos.rs) | 24× `CONSTRUCTION-R3-*` | **Active** (after operational green) |



Witness: [`ConstructionStageWitness`](../construction/construction_stage_witness.rs), [`ConstructionFinishWitness`](construction_finish_todos.rs), [`ConstructionPhase2Witness`](construction_phase2_todos.rs). **Not** on Stage 5.



Derived from [`recovery_construction.md`](recovery_construction.md) vs spine in **`src/construction/`**.

**Invariants:** [`construction_invariants.md`](construction_invariants.md) · **Round 3 plan:** [`construction_round3_plan.md`](construction_round3_plan.md) · **Operational gate:** [`construction_operational_gate.md`](construction_operational_gate.md)

**North star:** every placement = **intent → ghost → validate → commit** (RULE 1–5).

**Next work order (line 962+):** Phase 2 P6→P9 → `CONSTRUCTION_OPERATIONAL_GREEN` → Round 3 (R3-A catalog first).



---



## Completed lanes (reference only)



- **BUILD-P*** — toolbox, `ActiveBuildTool`, `BuildMode`, road/rail path, zone paint, module tree.

- **FINISH-BUILD-*** — physical move, import migration, demolish pick stub, legacy road gate, docs, finish board.



---



## Phase 2 — `PHASE2-BUILD-*` (active)



**Suggested order:** P6 (01–05) → P7 (06–11) → P8 (12–16) → P9 (17–20) when P6–P8 green.



### P6 — Authority & cleanup (do first)



| ID | Goal |

|----|------|

| **PHASE2-BUILD-01** | Remove `gui::build` shim; `crate::construction` only |

| **PHASE2-BUILD-02** | Demolish **execute** (real remove/despawn, not fake housing pending) |

| **PHASE2-BUILD-03** | Zone commit → strategic `Zone` / overlay — not `CivilHousing` mislabel |

| **PHASE2-BUILD-04** | Delete legacy tile-road intent systems |

| **PHASE2-BUILD-05** | `BuildingArchetypeId` → real `SiteArchetype` map |



### P7 — Tool UX



| ID | Goal |

|----|------|

| **PHASE2-BUILD-06** | Commercial submenu + placement |

| **PHASE2-BUILD-07** | Industrial submenu + placement |

| **PHASE2-BUILD-08** | Utilities submenu → PowerPlant / WaterPlant |

| **PHASE2-BUILD-09** | Building-only intent pipeline (isolated from zone/road input) |

| **PHASE2-BUILD-10** | Dedicated `src/construction/rail/` module |

| **PHASE2-BUILD-11** | Road popup live cost + invalid segment gate |



### P8 — Hardening & tests



| ID | Goal |

|----|------|

| **PHASE2-BUILD-12** | Ghost RULE 1 — preview only / no hidden commit |

| **PHASE2-BUILD-13** | Road e2e integration test |

| **PHASE2-BUILD-14** | Zone e2e integration test |

| **PHASE2-BUILD-15** | Shift/Alt/RMB input conflict matrix + test |

| **PHASE2-BUILD-16** | `debug_runs/construction_stage_live.json` proof |



### P9 — Advanced — **Done** (2026-05-20)

| ID | Goal |
|----|------|
| **PHASE2-BUILD-16** | `debug_runs/construction_stage_live.json` |
| **PHASE2-BUILD-17** | Curved Catmull-Rom preview |
| **PHASE2-BUILD-18** | Grid + node snap |
| **PHASE2-BUILD-19** | Upgrade nearest segment |
| **PHASE2-BUILD-20** | Terrain conform Y |

Detail registry: [`construction_p9_todos.rs`](construction_p9_todos.rs).



---



## Input conflict matrix (target for PHASE2-BUILD-15)



| Input | Zone tool | Building tool | Road/Rail | Demolish |

|-------|-----------|---------------|-----------|----------|

| LMB | Paint tile | Set ghost origin | Add path point | Pick demolish target |

| Alt+LMB drag | Paint strip | — | — | — |

| RMB | Undo last tile | — | Undo last point | — |

| Shift+LMB | Commit zone → pending | Queue blueprint | Finalize path → plan | — |



---



## Out of scope (recovery “Later Additions” — post phase 2)



Bulldozer mass-clear, bridges, tunnels, parallel road placement, lane editing editor, multiplayer sync.



---



## Progress log



| Date | Note |

|------|------|

| 2026-05-20 | BUILD-P* + FINISH-BUILD-* witness green; phase 2 board opened (20 rows). |
| 2026-05-20 | P6–P8 coded (01–15): shim removed, demolish/zone/building/rail; 10 construction tests pass. |
| 2026-05-20 | **P9 complete (16–20):** live proof JSON, splines, snap, upgrade, terrain conform; 15 construction tests pass. |

