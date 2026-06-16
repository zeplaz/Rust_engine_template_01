# DESIGN-IDENTITY-CHECKPOINT-001 — Project identity guard rail `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-IDENTITY-CHECKPOINT-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-27 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |
| **Source** | [`wssr_identity_alignment_record_v1.md`](wssr_identity_alignment_record_v1.md) |
| **Visual contract** | [`wssr_migration_visual_contract_v1.md`](wssr_migration_visual_contract_v1.md) |
| **Unblocks** | Hybrid assessment reviews; steward identity checkpoints |
| **No Rust** | One-page reject/tune/accept policy for reviewers |

---

## Purpose

When reviewing **coder Hybrid Assessments** or parallel VFX/WSS proposals, the designer records **ACCEPT**, **TUNE**, or **REJECT** against project identity — without blocking PRs.

**North star:** *Living industrial archive simulation* — not arcade RTS, not fantasy spell VFX, not siloed “engine rewrite” optics.

---

## Reject immediately

| Proposal pattern | Why |
|:---|:---|
| Neon particle stacks, muzzle flashes, screen-fill spell VFX | Breaks archive identity ([`hanabi_event_vfx_style_bounds_v1.md`](hanabi_event_vfx_style_bounds_v1.md)) |
| Parallel weather/smoke authority outside WSS spine | Dual truth; witness drift |
| Minimap / strategic zoom decorative particles | D-F09 / D-W09 readability |
| “Rewrite engine” framing without visual continuity | MED identity risk — water/fire **look** must hold |
| Gameplay-affecting L3 particles writing L1 | Simulation causality violation |

---

## Tune (do not block merge)

| Proposal pattern | Designer note |
|:---|:---|
| Slightly high particle count / alpha | Point to Hanabi bounds table; reduce |
| HUD density in sim session | Collapse to PLAY-01 defaults; tray optional |
| Contamination color saturation | Align to [`wss_contamination_visual_language_v1.md`](wss_contamination_visual_language_v1.md) |
| Parametric ghost token contrast | Match construction R4 tokens |

---

## Accept

| Proposal pattern | Condition |
|:---|:---|
| WSS slab/hydro/clipmap with **unchanged** player-facing water/fire/smoke look | Witness green on substrate + stage5 tactical rows |
| Field-compute smoke/fire extract extensions | Single extraction spine; no duplicate LOD |
| Construction parametric staging UX | Witness `construction_parametric_placement_001.green` |
| Replay scrub on minimap margin | [`minimap_replay_pass_002_v1.md`](minimap_replay_pass_002_v1.md) |

---

## Hybrid assessment quick map

| Domain | Default stance | Cite |
|:---|:---|:---|
| substrate / slab | ACCEPT when witness-only dual-write | identity record § substrate |
| atmosphere clipmap | TUNE until seam-free witness | stage5 + wss_substrate |
| contamination | ACCEPT separate state | contamination visual language |
| hanabi | TUNE until spike; REJECT main plugin early | hanabi bounds v1 |
| fire extract | ACCEPT when `f2_extract_witness.green` | fire_f2 pass record |

---

## Witness anchors (regression)

| Witness | Row |
|:---|:---|
| `stage5_full_app_live.json` | `tactical_vfx_witness`, `f2_extract_witness` |
| `fire_streaming_live.json` | per-view fire isolation |
| `wss_substrate_live.json` | clipmap, contamination, dual-write UX copy |
| `construction_stage_live.json` | parametric + R4 MV ghost |
| `minimap_compositor_live.json` | replay scrub + unit depth |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-05-27 |
