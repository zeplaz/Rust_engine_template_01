# Fire pinpoint sparks — design plan (master)

| Field | Value |
|:---|:---|
| **Version** | `1.1.0` |
| **Date** | 2026-05-24 |
| **Owner** | `@designer` → `@coder` **FX-FIRE-SPARK-001** |
| **Status** | **SIGNED** (2026-05-24) — track **CLOSED** [`fire_spark_track_closure_plan_v1.md`](fire_spark_track_closure_plan_v1.md) |
| **Brief** | [`fire_particle_spark_designer_brief_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/fire_particle_spark_designer_brief_v1.md) |
| **Worksheet** | [`fire_particle_spark_decision_worksheet_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/fire_particle_spark_decision_worksheet_v1.md) |
| **References** | [`assets/vfx/reference/elemental_sparks/`](../../assets/vfx/reference/elemental_sparks/) |

---

## Executive summary

Replace **soft billboard blobs** with **pinpoint sparks** inspired by legacy **razerz elemental** compute particles. **SIGNED** — coder refactors `fire_particle_draw.wgsl` / `fire_particle.wgsl` against §5 — **no new fire extract**.

---

## Legacy ↔ engine mapping

| Legacy (GLSL) | Mechanism | Engine target |
|:---|:---|:---|
| `compute_expanse_BASE_A.glsl` | pos/vel buffers, 24 attractors, `pos.w` lifetime | Phase B compute (D-F02 B) |
| `vertex_partical_BASE_A.glsl` | `age_intensity`, `pos_intensity` varyings | Instance channels → fragment |
| `frag_partical_BASE_A.glsl` | Age + position color mix, point sprite | `fire_particle_draw.wgsl` |
| Attractor array `[24]` | Pull toward fire cores | `FireVisualGpuInstance` centers (D-F03 A) |
| `pos.w` lifetime | Respawn at origin | Per-particle life channel |

**Local GLSL path:** `C:\dev\razerz-master\shaderzglsl\elemental\compute_partical\`  
**Skim notes:** [`fire_particle_legacy_elemental_reference_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/fire_particle_legacy_elemental_reference_v1.md)

---

## Gap — today vs target

| Aspect | Today (`fire_particle_draw.wgsl`) | Target (signed) |
|:---|:---|:---|
| Primitive | 6-vert quad, `smoothstep(0.5, 0.08, d)` blob | **D-F01 A** — point / ≤2px sharp core |
| Size | 2.5–10px half-edge expand | 0.5–2px spark |
| Color | Single orange blob mix | **D-F04 A** ash→orange + **D-F05 A** twinkle |
| Motion | Global sine on expand | Phase A twinkle; Phase B advection |
| Density | Few large instances | **D-F07 A** many low-α points |
| Zoom | Partial via `zoom_alpha` | **D-F09 A** fade when zoomed out |
| Blend | Alpha blob | **D-F08 A** additive cores + α embers |

**Mock:** [`fire_spark_target_v1.png`](../../assets/vfx/reference/elemental_sparks/fire_spark_target_v1.png)

---

## §5 — Signed decisions (2026-05-24)

| ID | Choice | Summary |
|:---|:---:|:---|
| D-F01 | **A** | Point sprites / sharp ≤2px |
| D-F02 | **A→B** | Twinkle Phase A; compute Phase B |
| D-F03 | **A** | Fire instance attractors |
| D-F04 | **A** | Ash→orange age mix |
| D-F05 | **A** | Legacy sin/cos twinkle |
| D-F06 | **B** | Palette tokens |
| D-F07 | **A** | Many / low α |
| D-F08 | **A** | Additive + alpha embers |
| D-F09 | **A** | Zoom fade |
| D-F10 | **A** | Sparks above smoke |

---

## Phases (execution)

| Phase | Owner | Deliverable | Status |
|:---|:---|:---|:---:|
| **FX-L0** | Designer | Worksheet + stills + **SIGNED** | ☑ |
| **FX-L1** | Coder | Phase A — `fire_particle_draw.wgsl` + sizing | **done** (2026-05-24) |
| **FX-L2** | Coder A | Phase B — `fire_spark_compute.wgsl` advection | **queued** |
| **FX-L3** | Designer | LOD / zoom storyboard | ☑ (D-F09 A) |
| **FX-L4** | Coder B | Witness JSON + scatter/zoom caps | **queued** |
| **FX-L5** | Coder A | Smoke draw order (D-F10 A) | **queued** |
| **FX-L6** | Coder B | Spark/Ember class split | **queued** |

**Coder queue:** [`fire_particle_spark_coder_queue_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/fire_particle_spark_coder_queue_v1.md)

---

## Authority (Stage 5 — do not break)

| Layer | Owner |
|:---|:---|
| Fire sim + extract | `FireVisualFrameSet` → projection |
| Particle instances | `WorldFireParticleFrame` (single upload) |
| Spark **look** | Shaders + designer tokens |
| Spark **motion** (Phase B) | Optional compute — same attractors |
| UI | Read-only — no particle spawn from HUD |

---

## §11 Designer sign-off checklist

| # | Item | Done |
|:---|:---|:---:|
| 1 | §5 **D-F01…D-F10** on worksheet | ☑ |
| 2 | Legacy stills in `assets/vfx/reference/elemental_sparks/` | ☑ |
| 3 | `fire_spark_target_v1.png` (blob vs pinpoint) | ☑ |
| 4 | Color key §6 committed | ☑ |
| 5 | Zoom/LOD (D-F09 A) documented | ☑ |
| 6 | Blend (D-F08 A) confirmed | ☑ |
| 7 | Phase A vs B scope agreed | ☑ |

**Verdict:** ☑ **SIGNED**

| Role | Date | Verdict | Notes |
|:---|:---|:---|:---|
| Designer | 2026-05-24 | **SIGNED** | Recommended defaults; GLSL skim + reference stills |
| Coder | 2026-05-24 | Acknowledged | **FX-FIRE-SPARK-001** — no duplicate extract |

---

## §6 Color key (signed)

| Role | Legacy (frag) | Palette map (D-F06 B) |
|:---|:---|:---|
| Cooling ash | `vec4(0.112, 0.115, 0.12, 0.8)` | `fg_muted` / ash token |
| Hot spark | `vec4(0.902, 0.27, 0.0, 0.8)` | `accent_hot` / fire orange |
| Twinkle peak | `0.902, 0.515, 0.082` | `dirty_amber` |
| Core additive | — | `accent_gold` @ additive |

---

## Coder handoff — **dual @coder active**

**Queue:** [`fire_particle_spark_coder_queue_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/fire_particle_spark_coder_queue_v1.md)

| Coder | Primary | First file |
|:---|:---|:---|
| **A** | FX-FIRE-SPARK-002 | `assets/shaders/fire/fire_spark_compute.wgsl` |
| **B** | FX-FIRE-SPARK-003 | `src/render/gpu_particles.rs` + `stage5_full_app_harness.rs` |

```
Lane: FX-FIRE-SPARK-002 (A) or FX-FIRE-SPARK-003 (B)
Read: fire_particle_spark_coder_queue_v1.md
Do NOT: second fire extract; cross-lane file edits
Verify: cargo test -p proc_A_dine01 --lib gpu_particles stage5
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.1.0 | 2026-05-24 | **SIGNED**; stills + target mock; FX-FIRE-SPARK-001 unblocked |
| v1.0.0 | 2026-05-24 | Initial plan — elemental legacy reference |
