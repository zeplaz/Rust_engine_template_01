# DESIGN-HANABI-BOUNDS-001 — Hanabi event VFX style bounds `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-HANABI-BOUNDS-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-27 |
| **Owner** | `@designer` |
| **Verdict** | **PASS (qualified)** — bounds signed; spike re-reviewed [`hanabi_spike_review_h_a2_gate_v1.md`](hanabi_spike_review_h_a2_gate_v1.md) (2026-05-27) |
| **Parent** | [`wssr_migration_visual_contract_v1.md`](wssr_migration_visual_contract_v1.md) § Layer 3 / H-A |
| **Identity** | [`wssr_identity_alignment_record_v1.md`](wssr_identity_alignment_record_v1.md) § hanabi |
| **Coder matrix** | [`coder_fleet_multistage_matrix_v1.md`](coder_fleet_multistage_matrix_v1.md) — **DESIGN-HANABI-BOUNDS-001** |
| **Unblocks** | `H-A` spike merge review; W4-C event embellishment proposals |
| **Forbidden** | Main-app `bevy_hanabi` plugin until this row + spike report are green |

---

## Purpose

Define **when Hanabi-style particles may appear** in this engine and **how they must look** so tactical VFX stays an **industrial living archive**, not arcade spell VFX.

**Scope:** event embellishment only (embers, micro-sparks, local wisps). **Out of scope:** weather, smoke authority, minimap, strategic map, gameplay collision.

---

## Accept / tune / reject rubric

| Signal | Designer action | Coder action |
|:---|:---|:---|
| Material kick-up at fire/water edge | **ACCEPT** | Spike in `experiments/hanabi_validation/` only |
| Muzzle-flash stacks, neon trails, screen-fill bursts | **REJECT** | Do not merge to main plugin |
| Fewer/smaller/lower-α than game-trailer refs | **TUNE** | Reduce count, lifetime, saturation |
| Particles write L1 sim or weather fields | **REJECT** | Architecture violation |
| Particles on minimap / planetary zoom | **REJECT** | Readability contract |

---

## Style bounds (numeric guidance)

| Token | Bound | Rationale |
|:---|:---|:---|
| **Max instances / event** | ≤ 32 local burst | Avoid fill-rate spikes |
| **Lifetime** | 0.2–1.2 s | Documentary, not persistent magic |
| **Alpha** | peak ≤ 0.45 | No opaque neon stacks |
| **Size** | sub-tile footprint | Tactical scale only |
| **Color** | desaturated warm/cool from domain palette | No pure #FF00FF arcade |
| **Motion** | gravity-biased drift; no orbit cameras | Industrial physics read |

---

## Allowed domains (post-spike)

| Domain | Allowed use | Authority |
|:---|:---|:---|
| Fire edge embers | Local L3 embellishment | Fire sim + extract owns heat/smoke |
| Water splash mist | Local L3 | Hydrology/water extract owns surface |
| Construction spark (micro) | One-shot on commit | Construction ghost tokens unchanged |

---

## Forbidden (hard)

| Use | Why |
|:---|:---|
| World rain/snow | Weather field + composite |
| Smoke density / columns | Layer A sim |
| Minimap / strategic draws | D-F09 / D-W09 |
| Gameplay collision from particles | L3 must not write L1 |
| Main plugin before signoff + H-A report | Identity risk HIGH |

**Aesthetic:** particles read as **material kick-up** in archive footage — not fantasy combat VFX.

---

## Spike exit criteria (coder, not designer)

1. `experiments/hanabi_validation/` report documents instance counts vs bounds table.
2. No regression on `stage5_full_app_live.json` → `tactical_vfx_witness`.
3. `hanabi_spike_report_present` witness row when wired.
4. Designer re-review: **DESIGN-HANABI-SPIKE-REVIEW-001** → H-A2 **CLEARED (qualified)** (2026-05-27).
5. Re-open only if a future preset exceeds **TUNE** thresholds in production wiring.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS (qualified)** | 2026-05-27 |
