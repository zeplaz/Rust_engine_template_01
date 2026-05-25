# Stage 7 Play — operator scenario `v1`

| Field | Value |
|:---|:---|
| **Status** | **DRAFT** — designer sign when reproducible |
| **Coder prereq** | **S7P-IND-001 DONE** — `activation_green: true` (2026-05-24) |
| **Owner** | `@designer` (S7P-DESIGN-001) |
| **Plan** | [`stages/stage7_play_plan_v1.md`](stages/stage7_play_plan_v1.md) |

---

## Preconditions

- `cargo run -p proc_A_dine01 --release -- --test visual` completes
- `BaseState::Simulation` reachable
- Construction toolbox / build rail available in sim

---

## Steps

| # | Action | Pass? | Notes |
|:---:|:---|:---:|:---|
| 1 | World gen → enter Simulation | ☐ | HUD: ops strip + context tray + minimap chrome |
| 2 | Place **concrete_aggregate_mine** | ☐ | Construction commit path |
| 3 | Place **concrete_cement_kiln** | ☐ | |
| 4 | Place **concrete_mixer_plant** | ☐ | |
| 5 | Advance sites to **Operational** | ☐ | Watch phase progression |
| 6 | Confirm production / activation | ☐ | Industrial HUD or diagnostics |
| 7 | Toggle logistics / minimap heat | ☐ | `logistics_rows > 0` in witness if seeded |
| 8 | (Optional) Heavy load on grid | ☐ | Smelter or kiln stress |

---

## Witness check (after run)

| File | Field | Expected |
|:---|:---|:---|
| `debug_runs/industrial_activation_live.json` | `activation_green` | `true` |
| | `concrete_chain_e2e.production_green` | `true` |
| `debug_runs/construction_stage_live.json` | operational flags | green |
| `debug_runs/minimap_compositor_live.json` | `logistics_rows` | `> 0` when corridors exist |

---

## Sign-off

| Role | Name | Date | Status |
|:---|:---|:---|:---|
| Designer | | | DRAFT / SIGNED |
| Operator | | | |
