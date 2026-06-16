# Fire overlay debug tooling `v1` (TRIAGE-FIRE-OVERLAY-DBG-001)

| Field | Value |
|:---|:---|
| **Program** | **TRIAGE-FIRE-OVERLAY-DBG-001** |
| **Triage** | `TRIAGE-FIRE-OVERLAY-DBG` · [`stage5_triage_backlog.md`](stage5_triage_backlog.md) T3 |
| **Source** | [`base_finsh_5.md`](../prompts/guides/base_finsh_5.md) — `fire-overlay-debug` |
| **Owner** | `@designer` (spec) · `@coder` (optional F3 wire) |
| **Date** | 2026-06-03 |
| **Verdict** | **PASS** |
| **Prior** | [`fire_streaming_debug_overlay_names_v1.md`](../docs/archive/2026-06-src-dev/plans/fire_streaming_debug_overlay_names_v1.md) (F7-B — **wired**) |

**No Rust in this doc.** F3-only engineer surfaces — not player ops strip, not weather WX line.

---

## Mission

Engineers debugging fire **overlay** (per-view extract, LOD caps, visible chunk set) need terse F3 lines that do not duplicate Stage 5 readiness spam or F7-B streaming rows. This spec closes triage **fire-overlay-debug** with a named contract + optional wire slice.

---

## Authority model

```text
FireSimulationSnapshot     → sim truth (heat, fuel)
ActiveFireChunkSet         → streaming active union (F7-B)
VisibleFireChunkSet        → per-view draw set (extract)
FireVisualFramesByView     → filtered instances per ViewId
FireLodBand / WorldLodBand → visual tier caps (F7-C)
```

**Rule:** Overlay debug reads **extract + visible set** — never imply sim correctness from draw counts alone.

---

## Surface inventory

| Surface | Status | Doc / code |
|:---|:---:|:---|
| **F7-B chunk streaming** | **Wired** | F3 `Fire Phase 7 — chunk streaming (F7-B)` · [`diagnostics_ui.rs`](../gui/diagnostics_ui.rs) |
| **Map gizmo legend** | **Wired** | Gold focus · red fire active · green terrain · gray empty |
| **GPU weather / fire field** | **Wired** | Compute debug sprite toggle (orthogonal to chunk LOD) |
| **Stage 5 readiness** | **Wired** | `fire_extract=` · `particle_lod=` rollup — keep; do not duplicate |
| **Fire overlay & LOD (extract)** | **Spec (P2 wire)** | This doc §F3 extract section |

---

## F3 — new section (optional wire: `FIRE-OVERLAY-DBG-UI-001`)

| Element | Canonical string |
|:---|:---|
| **CollapsingHeader** | `Fire overlay & LOD (per-view extract)` |
| **Placement** | After **Fire Phase 7 — chunk streaming (F7-B)** · before **GPU weather / fire field** |
| **default_open** | `false` (Simulation + editor) |

### Telemetry lines (wire order)

| # | Display line (template) | Source |
|:---:|:---|:---|
| 1 | `F7O gate=TRIAGE-FIRE-OVERLAY-DBG green={green}` | witness rollup |
| 2 | `F7O view={view_id} lod_band={band} cap={cap}` | `ViewId` + `WorldLodBand` + `fire_cap_for_world_band` |
| 3 | `F7O visible_chunks={vis} instances={inst} band={lod_band}` | `VisibleFireChunkSet` + `FireVisualFrame` count for active view |
| 4 | `F7O minimap_instances={n} tactical_instances={n}` | `FireVisualFramesByView` WorldMain vs Minimap |
| 5 | `F7O extract_authority=ViewProjectionAuthority policy_ok={bool}` | No camera-global shortcut when true |

**Example:**

```text
F7O gate=TRIAGE-FIRE-OVERLAY-DBG green=true
F7O view=WorldMain lod_band=LocalTactical cap=512
F7O visible_chunks=12 instances=48 band=LowFlame
F7O minimap_instances=3 tactical_instances=48
F7O extract_authority=ViewProjectionAuthority policy_ok=true
```

### Tooltip glossary

| Key | Meaning |
|:---|:---|
| **visible_chunks** | Chunks in per-view visible set after frustum + LOD radius |
| **instances** | Drawable fire instances after LOD band clamp |
| **minimap_instances** | Force strategic policy — must stay ≤ strategic cap |
| **policy_ok** | `VisibleFireChunkSet` derived only from view authority (base_finsh_5) |

---

## Player / product boundary

| Do | Do not |
|:---|:---|
| F3 engineer panels | Fire stats on ops strip (use **ALERTS** / fire channel) |
| Minimap `fire_heat` default **off** in sim ([`design_sim_hud_minimap_v1.md`](design_sim_hud_minimap_v1.md)) | Pink full-map wash at strategic zoom |
| Warm smoke plumes vs cool WX fog ([`design_weather_player_read_v1.md`](design_weather_player_read_v1.md)) | Relabel smoke as weather |

---

## Relation to other triage rows

| Triage ID | Relationship |
|:---|:---|
| **TRIAGE-FIRE-EXTRACT** | Coder hardening — this spec is **read-only debug** for extract |
| **TRIAGE-FIRE-LOD-TIERS** | Caps in `fire_view_extract.rs` — F7O lines surface live counts |
| **TRIAGE-FIRE-STREAM** | F7-B section — already wired; do not merge into F7O |

---

## Witness (designer close)

**Spec witness:** [`fire_overlay_debug_spec_live.json`](../debug_runs/fire_overlay_debug_spec_live.json)

| Key | Target |
|:---|:---|
| `gate` | `TRIAGE-FIRE-OVERLAY-DBG-001` |
| `green` | `true` (spec on disk + F7-B wired) |
| `f7b_debug_wired` | `true` |
| `f7o_extract_section_spec` | `true` |
| `f7o_ui_wired` | `false` until optional coder slice |

---

## Coder handoff (optional P2)

```text
FIRE-OVERLAY-DBG-UI-001 (optional)
Read:  src/dev/design_fire_overlay_debug_v1.md
       docs/archive/2026-06-src-dev/plans/fire_lod_player_read_v1.md
Touch: diagnostics_ui.rs — CollapsingHeader + F7O muted_label rows
Res:   FireVisualFramesByView, VisibleFireChunkSet, ViewManager active view
Do NOT: second fire extract · minimap ECS fire query · player HUD
Verify: cargo test -p proc_A_dine01 --lib fire_view_extract
```

---

## Sign-off

```text
TRIAGE-FIRE-OVERLAY-DBG-001 complete
Verdict: PASS
Doc: src/dev/design_fire_overlay_debug_v1.md
F7-B wired; F7O extract section spec ready for optional @coder wire
```
