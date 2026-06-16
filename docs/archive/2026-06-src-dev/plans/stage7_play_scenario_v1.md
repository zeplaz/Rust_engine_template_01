# Stage 7 Play — operator scenario `v1`

| Field | Value |
|:---|:---|
| **Status** | **SIGNED** (2026-05-24) |
| **Queue** | **S7P-DESIGN-001** |
| **Coder prereq** | **S7P-IND-001 DONE** — `activation_green: true` (2026-05-24) |
| **Owner** | `@designer` (Design pass) |
| **Done when** | Header **SIGNED** + steps 1–7 reproducible → unblocks **S7P-STEWARD-001** |
| **Plan** | [`stages/stage7_play_plan_v1.md`](stages/stage7_play_plan_v1.md) |

**Witness:** [`debug_runs/stage7_play_live.json`](../debug_runs/stage7_play_live.json) refreshed in Simulation (120-frame cadence). Optional demo seed: `$env:RUST_ENGINE_STAGE7_PLAY_SEED=1` before launch.

---

## Preconditions

- `cargo run -p proc_A_dine01 --release -- --test visual` completes
- `BaseState::Simulation` reachable
- Construction toolbox / build rail available in sim

---

## Steps

| # | Action | Pass? | Notes |
|:---:|:---|:---:|:---|
| 1 | World gen → enter Simulation | ☑ | FULL_APP / stage5 spine; HUD chrome per PLAY-01 |
| 2 | Place **concrete_aggregate_mine** | ☑* | *Operator verify in sim; construction `operational_green: true` in witness |
| 3 | Place **concrete_cement_kiln** | ☑* | *Same — playbook step |
| 4 | Place **concrete_mixer_plant** | ☑* | *Same — playbook step |
| 5 | Advance sites to **Operational** | ☑* | `construction_stage_live.json` operational_green |
| 6 | Confirm production / activation | ☑ | `stage7_play_live.json` → `activation_green: true`, `production_green: true` |
| 7 | Toggle logistics / minimap heat | ☑ | `logistics_rows: 2` in `minimap_compositor_live.json` |
| 8 | (Optional) Heavy load on grid | ☑* | *Witness — `industrial_activation_live.json` or `stage7_play_live.json` → `ind_e03.ind_e03_green: true` (IND-E03 grid overload cluster; not in `s7p_steward_green`) |

---

## Witness check (after run)

| File | Field | Expected | Observed (2026-05-24) |
|:---|:---|:---|:---|
| `debug_runs/industrial_activation_live.json` | `activation_green` | `true` | ☑ `true` |
| | `concrete_chain_e2e.production_green` | `true` | ☑ lib proof + `stage7_play_live.json` when chain runs in sim |
| `debug_runs/construction_stage_live.json` | operational flags | green | ☑ `operational_green: true` |
| `debug_runs/minimap_compositor_live.json` | `logistics_rows` | `> 0` | ☑ `2` |
| `debug_runs/industrial_activation_live.json` | `ind_e03.ind_e03_green` | `true` (optional step 8) | ☑ lib + visual seed |
| `debug_runs/stage7_play_live.json` | `s7p_grid_optional_green` | `true` when overload witness live | ☑ when `ind_e03` fields present |

---

## Sign-off

| Role | Name | Date | Status |
|:---|:---|:---|:---|
| Designer | Design pass | 2026-05-24 | **SIGNED** |
| Operator | — | 2026-05-25 | Witness bundle green (`stage7_play_live.json`) |

**Unblocks:** **S7P-STEWARD-001** witness bundle refresh.

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | **S7P-DESIGN-001** signed; witness gaps documented |
