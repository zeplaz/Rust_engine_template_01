# Building Visual Concept v2 — “reads as a building” (not collage)

| Field | Value |
|:---|:---|
| **ID** | **DES-BUILDING-LOOK-V2** |
| **Date** | 2026-09-24 |
| **Owner** | `@designer-mcp` (charter) · `@coder-mcp` / `@coder` (enforce) |
| **Supersedes** | “random module collage / lod0 stand-in” as the ship look |
| **Rules** | mcp-production-rules unchanged (no AI art · deterministic · batch/atlas · grid-aligned) |
| **Done bar** | New assemblies pass **machine look-gates** below — **not** operator golden-seed theater |

```text
REJECT  random module collage · lod0/smoke as ship look · zero-yaw perimeter · 1m spacing of 4m modules
REQUIRE closed envelope · outward faces · continuous roof seat · street openings · ground contact · production meshes
```

---

## 0. Visual diagnosis (BQ-Q2 screens, 2026-09-24)

Inspected `debug_runs/bq_q2_screens/*.png` (victorian / industrial_west / rural).

Ruthless read failures:

- **Not a volume** — reads as intersecting thin slabs / L-brackets / floating wedges, not an enclosed box.
- **Massing collapse** — no base → wall → roof hierarchy; silhouette is debris.
- **Roof collage** — oversized / detached roof pieces; sawtooth/gable fans floating above spindly bases.
- **No openings** — doors/windows absent or lost in overlap; no human scale.
- **Ground contact missing** — bottoms hang in void; no plinth read.
- **Greybox / lod0 residue** — grainy placeholder materials; Q2 path forced `source_tier=lod0`.
- **Style blindness** — victorian / industrial / rural are indistinguishable abstract piles.
- **Root mechanical cause (code)** — placements used **1 m cell spacing** while modules are **`GRID_UNIT_M = 4 m`**, so every bay overlaps into a collage. Compounded by **all `rotation_euler = 0`** (walls never face outward) and **perimeter-only roof cells**.

SILH continuity at 100% while pixels fail = **metric theater**. Look gates must match the eye.

---

## 1. What a building MUST look like (player read distance)

At typical iso / city-sim distance, a building is a **readable envelope**, not a kitbash pile.

| Layer | Must-have |
|:---|:---|
| **Silhouette** | One continuous mass (rect / L / U). No floating detached primaries. |
| **Base** | Continuous ground contact along the footprint; no hovering stubs. |
| **Wall** | Closed perimeter; each bay **faces outward**; thickness reads as wall, not a card stack. |
| **Opening** | Street face has a door (floor 0) and windows (upper floors) at human scale. |
| **Roof** | Continuous seat covering the footprint; ridge/pitch consistent; sits on wall tops. |
| **Material** | Style-pack coherent (brick ≠ steel ≠ wood) from **production** modules only. |

Forbidden as ship output: lod0/smoke stand-ins, greybox kits, zero-yaw walls, overlapping 4 m meshes on 1 m grid, per-cell roof perimeter rings without filled plan.

---

## 2. Generation rules (hard — not optional QC)

These are **composition contracts**. Violation = fail assemble / fail look witness.

1. **Grid spacing** — `position.xz = grid_xy * GRID_UNIT_M` (4.0). Floor `y = floor * FLOOR_HEIGHT_M` (3.0). Same in Python + Rust.
2. **Outward yaw** — every W/D/C/O placement gets `rotation_euler` yaw so the module face points off-footprint.
3. **Roof plan** — roof tokens form a **ridge row** (`y = depth // 2`, one bay per `x`) — not a full carpet of independent pitched/sawtooth modules (reads as plank collage) and not perimeter-only rings.
4. **Openings** — floor 0 street bay = door; upper street bays = window slot when pack has `window_1u`.
5. **Corners as walls** — do not instance style-blind `corner_L` kits for envelope; corner cells use wall/door/opening with outward yaw.
6. **Production-only ship path** — default `source_tier=production`. Visual QC / Q2 screens must use production. lod0 is pilot/greybox only.
7. **No silent collage** — if resolvable production placements < envelope minimum, **fail** — do not emit a junk assembly.

---

## 3. Concept rejection

| Old concept | v2 |
|:---|:---|
| Module collage until “operator likes a seed” | Envelope grammar + machine look gates |
| lod0 screens as quality proof | Production meshes + composition contracts |
| SILH % as “looks like a building” | SILH ≠ look; look = envelope + openings + roof seat |
| BQ-Q3 golden approve theater | Deferred — fix look in code/art first |

---

## 4. Implementation slices

| ID | Owner | Goal |
|:---|:---|:---|
| **BUILDING-LOOK-V2-GRID** | `@coder` + `@coder-mcp` | 4 m grid spacing parity Python/Rust + scale_chain assert |
| **BUILDING-LOOK-V2-FACE** | `@coder-mcp` | Outward yaw on W/D/C/O |
| **BUILDING-LOOK-V2-ROOF** | `@coder-mcp` | Full-footprint roof tokens |
| **BUILDING-LOOK-V2-OPEN** | `@coder-mcp` | Street window/door tokens |
| **BUILDING-LOOK-V2-QC** | `@coder-mcp` | Q2 screens force production; BEFORE/AFTER witness |
| **BUILDING-LOOK-V2-ART** | `@designer-mcp` | Rebake style-true roofs/walls/openings where mesh still fails read |

**BQ-Q3-OPS-APPROVE-001** — **deferred / cancelled for now** (operator theater). Revisit only after look-v2 screens pass machine envelope gates.

---

## 5. Witness

- Diagnosis + AFTER captures: `debug_runs/building_look_v2/`
- Live gate: `debug_runs/building_look_v2_live.json`
- Rules: no `operator_pass: true` without real stills; no fake green over collage.

---

## 6. order_critique (this charter)

```yaml
order_critique:
  request_summary: "Reinvention of building look; stop BQ-Q3 operator theater; implement composition fixes"
  concerns:
    - "Machine SILH/green hid collage failures"
    - "1m placement of 4m modules is structural, not taste"
  rules_audit:
    no_ai_generated_images: pass
    deterministic_output: pass
    batch_processing: pass
    grid_alignment: pass
  blocked: false
  foresight_flags:
    - "Style-pack slot holes still need art bake (BQ-K)"
    - "Corner L mesh yaw may need kit audit after wall yaw lands"
  proceed: yes
```
