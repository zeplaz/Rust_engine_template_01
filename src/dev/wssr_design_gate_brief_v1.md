# WSS design gate brief `v1` (WSS-DESIGN-GATE-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **WSS-DESIGN-GATE-001** |
| **Parent plans** | [`wssr_index_v1.md`](wssr_index_v1.md) · WSS-PLAN-002/003/004 |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Blocks** | **WSS-CHUNK-SLAB-001**, **WSS-ATMOS-CLIPMAP-001**, **WSS-HYDRO-RUNTIME-001** (no substrate Rust until PASS) |
| **Status** | **CLOSED** — parent 4/4 + G1–G2/G6 — [`wss_design_gate_parent_closure_v1.md`](wss_design_gate_parent_closure_v1.md) |

**Rule:** Design / evaluation / sign-off only. No Rust. Coders implement **after** this gate — not in parallel with first substrate slice unless planner explicitly waives one row.

---

## Why this gate exists

WSSR is a **structural** refactor (L1 substrate → L2 extraction → L3 visualization). Replacing working VFX/spine paths without evaluation risks:

- Losing superior patterns already in repo (per-view fire extract, closed water W1/W2, ViewManager isolation)
- Aesthetic drift away from project identity (industrial logistics, planetary command, DF persistence — see index § North star)
- Silo reintroduction (`OceanSystem`, `DustSystem`) despite planner policy

**Designer role:** arbitrate **identity + readability + long-term growth** — not block infrastructure, but force **hybrid** choices where existing systems win.

---

## Identity anchors (evaluate every recommendation)

| Anchor | Source | Question for each WSS change |
|:---|:---|:---|
| **Manifested artifact** | [`wssr_index_v1.md`](wssr_index_v1.md) § North star | Does this read as one coherent world sim, not a feature bundle? |
| **Visual ethos** | [`prompts/guides/ui/design_theme.md`](../prompts/guides/ui/design_theme.md) | Do tactical overlays stay readable at default zoom? Warm industrial palette preserved? |
| **Operational vs infrastructure** | [`operational_readiness_vs_infrastructure_perf_v1.md`](../prompts/guides/operational_readiness_vs_infrastructure_perf_v1.md) | Is this L1 truth or L3 polish? |
| **Elemental VFX charter** | [`planner_elemental_vfx_domain_charter_v1.md`](planner_elemental_vfx_domain_charter_v1.md) | Does extract path stay single-writer? |
| **Stage 5 spine** | [`stage5_convergence_directive_v1.md`](../prompts/guides/stage5_convergence_directive_v1.md) | No parallel representation stacks? |

---

## Deliverables (designer — all required)

### 1. `wssr_identity_alignment_record_v1.md`

| Section | Content |
|:---|:---|
| **Verdict** | PASS / PASS (qualified) / DEFER / REJECT (per domain) |
| **Ethos fit** | 1–2 paragraphs: how WSS serves “planetary industrial sim” vs generic engine |
| **Preserve list** | Systems/patterns that must **not** be deleted (min 5 bullets with `path` + reason) |
| **Replace list** | What WSS supersedes — only with **successor named** |
| **Hybrid list** | Where slab + existing ECS must **coexist** (e.g. fire front on `ActiveChunkRuntime`, weather on clipmap + `ChunkWeather` bridge) |

### 2. `wssr_readability_impact_v1.md`

Per player-facing surface:

| Surface | WSS touch | Readability risk | Mitigation spec |
|:---|:---|:---|:---|
| Tactical map | fire/smoke/dust/contamination | … | … |
| Minimap | heat-only fire, hydrology dim | … | … |
| World preview | no new substrate bleed | … | … |
| Construction ghosts | unchanged authority | … | … |
| Strategic zoom | D-F09 / D-W09 cull policy | … | … |

### 3. `wssr_migration_visual_contract_v1.md`

| Topic | Designer decision |
|:---|:---|
| Sim clipmap L0–L3 vs render clipmap | What player **sees** at each zoom band |
| Contamination | Color + pattern language (not color-only) — align accessibility |
| Ocean / river | Hydrology-driven water **look** — reference [`water_surface_target_v1.png`](../assets/vfx/reference/water/water_surface_target_v1.png) |
| Smoke / dust | Layer A vs B — when partial alpha vs field-only |
| Hanabi (future) | Event VFX **style** bounds — no arcade particles in industrial sim |

### 4. `wssr_design_signoff_v1.md`

Sign-off table:

| Child plan | Designer verdict | Conditions |
|:---|:---|:---|
| WSS-PLAN-002 chunk slabs | | |
| WSS-PLAN-003 hydrology | | |
| WSS-PLAN-004 atmosphere | | |
| Hanabi spike | | experiments only |

**Unblocks:** coder orders in [`wssr_coder_hybrid_orders_v1.md`](wssr_coder_hybrid_orders_v1.md)

### 5–6. Slab preflight designer slices (G1–G2)

| Deliverable | Path | Status |
|:---|:---|:---:|
| Diagnostics hybrid copy | [`wss_substrate_diagnostics_copy_v1.md`](wss_substrate_diagnostics_copy_v1.md) | ☑ |
| Overlay + witness key names | [`wss_substrate_debug_overlay_names_v1.md`](wss_substrate_debug_overlay_names_v1.md) | ☑ |

Preflight checklist: [`wss_design_gate_001_v1.md`](wss_design_gate_001_v1.md) — steward **G3–G4** still open for full `pass: true`.

---

## Evaluation worksheet (copy per domain)

```yaml
domain: chunk_slab | hydrology | atmosphere | contamination | hanabi
current_system: path + Symbol
wss_proposal: one sentence
superior_incumbent: yes | partial | no
recommendation: adopt | hybrid | defer | reject
hybrid_shape: "slab owns X; ECS retains Y until Z"
identity_risk: low | med | high
witness_impact: wss_substrate_live.json keys
```

---

## Do not (designer)

- Mandate deletion of closed tracks (F7 exit, water W1/W2 witnesses) without coder hybrid plan
- Spec gameplay mutations from UI-only review
- Approve new top-level `*VfxSystem` / `*OceanSystem` modules (planner policy forbids)

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | Design gate opened before WSS coder spine |
