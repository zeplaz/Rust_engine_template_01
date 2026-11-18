# PLAN-BLAST-PROFILE-v1 — data-driven blast execute thresholds

| Field | Value |
|:---|:---|
| **Id** | **PLAN-BLAST-PROFILE-v1** |
| **Status** | **ADOPTED** — RON + sole execute owner wired |
| **Date** | 2026-11-17 |
| **Owner** | `@coder` (construction) |
| **Parents** | PHYS-07 · [`ballistics_blast_contract_v1.md`](ballistics_blast_contract_v1.md) · DF-8 |
| **Books** | PHYS-07 (gea4v2 14.5.3.4 explosions — finite-support falloff, not continuum blast) |

## Objective

Move blast **execute thresholds / scales** out of magic numbers in `blast.rs` into a RON-backed [`BlastProfile`](../construction/blast.rs) resource, without changing authority:

- single writer: `execute_blast_intents_system`
- `BlastIntent` keeps instance `radius_m` / `damage` / `penetration_m`
- combat warheads stay the yield table (`WarheadCatalog`)
- finite-support linear falloff stays
- no spatial index, no FEA column smear, no multi-`ResMut` execute split

## Separation

| Surface | Owner | Contents |
|:---|:---|:---|
| `WarheadCatalog` | combat | per-warhead radius / peak damage / pen (instance yield) |
| `BlastProfile` | construction | demolish gate, pen multiplier curve, dam peak-loss / leak / breach flow scales |
| `BlastIntent` | enqueue → execute | one impact’s center + instance yield fields |

## Keep / adopt / reject

| Keep | Adopt | Reject |
|:---|:---|:---|
| Sole execute owner + intent queue | RON `assets/configs/construction/blast_profile_v1.ron` | Sedov / \(1/r^2\) as correctness |
| Linear finite-support falloff | Compiled-in `Default` fallback (warheads pattern) | FEA / adjacent column load smear |
| Direct Tier-2 `mark_tile` from blast | `BLAST_DEMOLISH_DAMAGE` = compiled default gate (tests / tripwires) | Combat-owned spatial insert/remove |
| Crest-precise `carve_dam_columns` | | Split crater/site/mask into parallel writers |

## Files

| Path | Change |
|:---|:---|
| `assets/configs/construction/blast_profile_v1.ron` | new — thresholds |
| `src/construction/blast.rs` | `BlastProfile` + load/parse; wire execute + carve |
| `src/construction/mod.rs` | re-export |
| `src/dev/plan_blast_profile_v1.md` | this plan |

## Accept

```text
cargo test -p proc_A_dine01 --lib construction::blast::
# or validate-report cargo (scoped)
```

Tripwires still pass: sole `BlastCraterBook` writer; ballistics enqueue-only; demolish gate matches profile default.
