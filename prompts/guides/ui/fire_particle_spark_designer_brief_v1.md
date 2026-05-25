# Fire particle / pinpoint sparks — designer brief `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.0.0` |
| **Date** | 2026-05-24 |
| **Owner** | `@designer` (VFX read model) · `@coder` after **SIGNED** |
| **Status** | **SIGNED** — see master plan §11 · **FX-FIRE-SPARK-001** unblocked |
| **Master plan** | [`fire_particle_spark_design_plan_v1.md`](../../../src/dev/fire_particle_spark_design_plan_v1.md) |
| **Worksheet** | [`fire_particle_spark_decision_worksheet_v1.md`](fire_particle_spark_decision_worksheet_v1.md) |
| **Legacy reference** | `C:\dev\razerz-master\shaderzglsl\elemental\compute_partical\` *(local — not in repo)* |

---

## 0. Problem (plain English)

Today’s world fire reads as **soft orange blobs** on billboards. It does not match the **pinpoint spark field** from the old **elemental compute particle** stack — tight points, age-driven fade, position-twinkle, attractor-driven motion.

**Goal:** Restore **spark identity** (point-like, crisp, ember-hot) while keeping Stage 5 spine rules — **one fire extract**, projection graph, no parallel particle sim in UI.

---

## 1. Legacy reference — what to steal (not copy blindly)

### Source files (razerz `elemental`)

| File | Role |
|:---|:---|
| `compute_partical/compute_expanse_BASE_A.glsl` | GPU compute: pos/velocity buffers, **24 attractors**, lifetime in `pos.w`, respawn at origin |
| `compute_partical/vertex_partical_BASE_A.glsl` | Passes **`age_intensity`** (`vert.w`) + **`pos_intensity`** (`vert.xy`) to fragment |
| `compute_partical/frag_partical_BASE_A.glsl` | **Pinpoint look:** mixes age gray→orange + sin/cos position twinkle; intended for **point sprites** (`gl_PointCoord` path in comments) |

### Legacy visual vocabulary (designer)

| Trait | Legacy behavior | Target in new engine |
|:---|:---|:---|
| **Silhouette** | Point / spark, not quad blob | Point sprite or sub-pixel quad ≤2px screen |
| **Age** | `age_intensity` drives dark ash → hot orange | Normalized lifetime channel in fragment |
| **Twinkle** | `pos_intensity` sin/cos modulates color | Per-particle hash + time — no full-screen pulse |
| **Motion** | Compute advection + attractors toward fire cores | Optional Phase B — sim reads fire instances as attractors |
| **Density** | Many small points | Higher count, lower individual alpha |
| **Color** | `#E67345` hot, `#1D1D1E` cooling ash | Map to [`design_theme.md`](design_theme.md) fire tokens |

### Reference capture (designer task)

Export **3 stills** from legacy build or shader sandbox (if runnable):

1. Dense crown fire — many pinpoint sparks  
2. Single source — falloff field  
3. Cooling / respawn frame — gray→orange transition  

Save under: `assets/vfx/reference/elemental_sparks/` *(create when ready)*.

---

## 2. Current engine (coder context — do not redesign sim here)

| Layer | Path | Today |
|:---|:---|:---|
| Sim / extract | `FireVisualFrame` → projection → `WorldFireParticleFrame` | Instance rows from fire extract |
| Expand | `assets/shaders/fire/fire_particle.wgsl` | **Billboard quads**, sine wobble, 2.5–10px half-edge |
| Draw | `assets/shaders/fire/fire_particle_draw.wgsl` | **Radial soft blob** `smoothstep(0.5, 0.08, d)` |
| Policy | `gpu_particles.rs`, `PerViewRepresentationPolicy` | Budget + zoom-stable sizing |

**Diagnosis:** Fragment + expand stages optimize for **readable heat markers**, not **spark shower**.

---

## 3. Target look — three particle classes (designer owns names)

| Class | ID | Read | Size (screen) | Alpha | Motion |
|:---|:---|:---|:---|:---|:---|
| **Pinpoint spark** | `Spark` | Primary ask | **0.5–2 px** core | High peak, fast decay | Upward bias + flicker |
| **Ember** | `Ember` | Secondary | 2–6 px | Medium | Slow drift |
| **Heat haze** | `Haze` | Optional macro | 8–24 px soft | Low | Existing blob OK at LOD far |

**Rule:** Tactical zoom → mostly **Spark + Ember**; strategic zoom → **Haze + chunk heat** only (no spark soup).

---

## 4. §5 decisions — designer must resolve (D-F01…D-F10)

Mark **A / B / C** on [`fire_particle_spark_decision_worksheet_v1.md`](fire_particle_spark_decision_worksheet_v1.md).

| ID | Question | A | B | C |
|:---|:---|:---|:---|:---|
| **D-F01** | **Primary raster** | **Point sprites** (legacy-like) | Sub-pixel quads (1–2px) | Keep quads, sharp falloff |
| **D-F02** | **Motion model** | Static twinkle only (Phase A) | **Compute advection** (legacy attractors) Phase B | CPU trail on extract rows |
| **D-F03** | **Attractor source** | Fire instance centers | Chunk heat peaks | Manual emitters only |
| **D-F04** | **Lifetime visual** | Age gradient ash→orange (legacy `mix`) | Binary on/off | Heat-linked only |
| **D-F05** | **Twinkle function** | Legacy sin/cos on position | Hash noise | None (steady points) |
| **D-F06** | **Color key** | Legacy orange `#E67345` / ash `#1D1D1E` | [`design_theme.md`](design_theme.md) tokens | Custom sheet (attach) |
| **D-F07** | **Density** | High count / low alpha sparks | Medium | Low count / bright cores |
| **D-F08** | **Blend mode** | Additive sparks + alpha embers | Alpha only | Additive only |
| **D-F09** | **Zoom behavior** | Sparks fade out < zoom 0.4 | Constant screen px size | Sparks only when zoom > 0.7 |
| **D-F10** | **Smoke coupling** | Sparks above smoke field | Independent | Sparks replace smoke v1 |

**Recommended (smooth restore of legacy feel):** D-F01 **A**, D-F02 **A then B**, D-F03 **A**, D-F04 **A**, D-F05 **A**, D-F06 **B**, D-F07 **A**, D-F08 **A**, D-F09 **A**, D-F10 **A**.

---

## 5. Motion & timing (functional spec — implement after SIGNED)

| Interaction | Target | Reference |
|:---|:---|:---|
| Spark birth | 0–40ms flash to peak | legacy respawn `pos.w += 3` |
| Spark life | 0.4–1.2s visible | tune vs performance |
| Twinkle freq | 8–18 Hz per particle | legacy frag sin terms |
| Advection (Phase B) | Toward attractor + upward bias | `compute_expanse_BASE_A.glsl` |
| Respawn | When life ≤ 0 → fire core origin | legacy lines 60–65 |
| Zoom out | Spark count × LOD cap | `PerViewRepresentationPolicy` |

**Forbidden:** Full-screen sine pulse on all particles (current expand wobble reads as “jelly blob”).

---

## 6. Palette tokens (preview-only extension)

Add to designer sheet; coder maps to `palette.rs` / shader constants when signed.

| Token | Suggested hex | Role |
|:---|:---|:---|
| `spark_core` | `#FFE8A8` | Hottest center |
| `spark_body` | `#E67345` | Legacy orange body |
| `spark_ash` | `#1C1C1E` | Cooling trail |
| `spark_twinkle` | `#FF9040` | Twinkle peak |
| `ember_glow` | `#C44A12` | Larger ember class |

**No bloom / HDR glow v1** — pinpoint comes from **size + alpha**, not post.

---

## 7. Designer deliverables (before @coder)

| # | Deliverable | Path |
|:---|:---|:---|
| 1 | Completed worksheet **D-F01…D-F10** | [`fire_particle_spark_decision_worksheet_v1.md`](fire_particle_spark_decision_worksheet_v1.md) |
| 2 | Reference stills or GIF (legacy or paint-over) | `assets/vfx/reference/elemental_sparks/` |
| 3 | **Before/after** mock: blob (today) vs pinpoint (target) | `assets/vfx/reference/fire_spark_target_v1.png` |
| 4 | Color key PNG (5 swatches §6) | same folder |
| 5 | §11 checklist **SIGNED** on master plan | [`fire_particle_spark_design_plan_v1.md`](../../../src/dev/fire_particle_spark_design_plan_v1.md) |

---

## 8. @coder handoff (deferred — FX-FIRE-SPARK-001)

**Blocked until §11 SIGNED.**

```
Lane: FX-FIRE-SPARK-001 Phase A (look)
Read: fire_particle_spark_design_plan_v1.md signed §5
Legacy ref: razerz elemental compute_partical/*.glsl (local)
First: fire_particle_draw.wgsl — point sprite OR 1px falloff (D-F01)
Do NOT: second fire extract; break WorldFireParticleFrame spine
Verify: cargo test -p proc_A_dine01 --lib stage5 + visual fire scene
```

**Phase A files (look only):**

| File | Change |
|:---|:---|
| `assets/shaders/fire/fire_particle_draw.wgsl` | Pinpoint fragment |
| `assets/shaders/fire/fire_particle.wgsl` | Reduce blob wobble; spark sizing |
| `src/render/gpu_particles.rs` | Class split Spark vs Ember sizing |

**Phase B (motion — if D-F02 B signed):**

| File | Change |
|:---|:---|
| New `assets/shaders/fire/fire_spark_compute.wgsl` | Port attractor advection from legacy compute |
| `src/render/gpu_particles.rs` | Buffer lifecycle |

---

## 9. Cross-links

| Doc | Role |
|:---|:---|
| [`fire_ecology_f1_todos.md`](../../../src/dev/fire_ecology_f1_todos.md) | Sim fuel — orthogonal |
| [`stage5_triage_backlog.md`](../../../src/dev/stage5_triage_backlog.md) TRIAGE-PHASE-F-CULL | LOD caps |
| [`world_preview_runbook_v1.md`](../world_preview_runbook_v1.md) | Separate lane |
| [`design_theme.md`](design_theme.md) | Global motion + color |

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | Initial brief + legacy elemental reference |
