# Fire LOD — player readability by band `v1` (FIRE7-DESIGN-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **FIRE7-DESIGN-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@designer` |
| **Status** | **SIGNED** |
| **Prereq** | **FIRE7-PLAN-001** — [`fire_sim_phase7_architecture_v1.md`](fire_sim_phase7_architecture_v1.md) |
| **Plan** | [`stages/fire_sim_phase7_plan_v1.md`](stages/fire_sim_phase7_plan_v1.md) |
| **Coder waves** | **F7-C** (`TRIAGE-FIRE-LOD-TIERS`) — blocked until **F7-A-001** closed |
| **No Rust** | Design table only — caps are **policy targets** until F7-C wires `FireChunkLodState` |

---

## Purpose

Tell players **what fire should look like** at each zoom / view band so F7-C extract clamps match product intent — not engineering defaults.

**Authority (unchanged):** sim truth → `FireVisualFramesByView` → projection graph / compositor. **Forbidden:** second global extract; minimap reading fire ECS ([`steward_fire7_preflight_gate_v1.md`](steward_fire7_preflight_gate_v1.md)).

---

## Primary table — player read by band

| Product band | `WorldLodBand` (view) | Typical views | **Player should see** | Sim / GPU presentation | Policy instance cap | Overlay / channel |
|:---|:---|:---|:---|:---|:---:|:---|
| **Strategic** | `Macro` · `Strategic` | Minimap, far map | **Heat blobs only** — where fire exists, not individual flames | Aggregated chunk heat; no spark VFX | **32** | Compositor fire-heat bin; no tactical particles |
| **Operational** | `Operational` | Mid zoom main map | **Cluster caps** — soft glow + sparse sparks at cluster centroids | Heat + thinned instances; `LowFlame` / sparse smoke | **128** | Heat + sparse spark policy; minimap stays heat-only |
| **Tactical** | `LocalTactical` | `WorldMain` / `SimulationMap` near | **Instances + sparks** — readable fronts, wind-aligned streaks | Full `FireVisualFrame` within cap; VFX track for sparks | **512** (policy) | Full fire extract path + tactical VFX |
| **Cinematic** | Dev / replay / debug zoom-in | Focused debug camera | **Full local detail** — per-chunk flame/smoke mix, highest fidelity | `FireChunkLodState` per chunk; `FullFlame` when sim allows | Budgeted by chunk LOD state (no hard global 512) | VFX + GPU particles; debug overlay optional |

**Designer rule:** When the player cannot name *where* fire is spreading, the band is too detailed (strategic) or too abstract (tactical). Each step should add **actionable** detail.

---

## `FireLodBand` clamp (sim → GPU row)

Maps sim heat to drawable rows before particles ([`fire_view_extract.rs`](../render/fire_view_extract.rs) `clamp_fire_lod_for_world_band`).

| `FireLodBand` | Strategic / Macro view | Operational | Tactical / cinematic |
|:---|:---|:---|:---|
| **None** | Hidden | Hidden | Hidden |
| **SmokeOnly** | Heat blob only | Heat + faint smoke | Smoke column |
| **LowFlame** | ↓ clamped to **SmokeOnly** | Low flame sprite | Low flame |
| **FullFlame** | ↓ **SmokeOnly** | ↓ **LowFlame** | Full flame + sparks |

**Intent:** Distance never upgrades detail — only preserves or downgrades.

---

## Per-view exceptions

| View | Band treatment | Player read |
|:---|:---|:---|
| **Minimap** | Force **Strategic** policy regardless of main-map zoom | Heat blobs only — **no** spark particles |
| **World Preview** | Fire overlay off or heat-only per `ViewRenderPolicy` | No tactical fire chrome in archive table |
| **Infrastructure / isolation views** | Independent `FireVisualFramesByView` entry | Same band table; must not leak tactical instances into minimap |

---

## Acceptance cues (playtest)

| Band | Pass | Fail |
|:---|:---|:---|
| Strategic | Player spots fire **regions** at a glance | Individual flame sprites visible on minimap |
| Operational | Clusters read as **fronts**; sparks rare | Screen filled with particles; unreadable logistics |
| Tactical | Spread direction obvious; sparks reinforce motion | Flat heat sheet with no depth |
| Cinematic | Local variation (smoke vs flame) visible | Identical to tactical with no added detail |

---

## Coder handoff (F7-C)

```
Read: src/dev/fire_lod_player_read_v1.md
      src/dev/fire_sim_phase7_architecture_v1.md § LOD table
Touch: fire_view_extract.rs, FireChunkLodState (≤3 files per slice)
Do: enforce policy caps per WorldLodBand; strategic vs tactical counts differ in lib test
Do NOT: second global FireVisualFrame extract; minimap ECS fire reads
Verify: cargo test -p proc_A_dine01 --lib fire_view_extract stage5
Witness: stage5_full_app_live.json fire rows + per-band cap test
```

| Policy constant (suggested) | Value | Band |
|:---|:---:|:---|
| `FIRE_LOD_CAP_STRATEGIC` | 32 | Strategic |
| `FIRE_LOD_CAP_OPERATIONAL` | 128 | Operational |
| `FIRE_LOD_CAP_TACTICAL` | 512 | Tactical |

---

## Sign-off

| Role | Date | Verdict |
|:---|:---|:---|
| Designer | 2026-05-25 | **SIGNED** — table ready for F7-C |
| Coder | — | Pending **F7-A-001** then F7-C |

**Unblocks:** **FIRE7-DESIGN-002** (optional debug labels); **F7-C** policy implementation.

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | **FIRE7-DESIGN-001** initial SIGNED table |
