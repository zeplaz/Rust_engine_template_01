# APS preview v2 — four-state contract `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-PREVIEW-V2-001** |
| **Program** | APS UI/UX phase 2 · Track A2 |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Authority** | [`design_aps_design_system_v11_delta_v1.md`](design_aps_design_system_v11_delta_v1.md) §4 N2 · [`aps_preview_state.py`](../../tools/mcp/art_pipeline_suite/aps_preview_state.py) |
| **Handoff** | OVR-P55-PREVIEW-002 · slot_preview · assembly_preview · catalog thumbs |
| **Verdict** | **PASS** |

```text
DES-APS-PREVIEW-V2-001 Q✓
4 visual states — clean / night / damaged / burning — async-on-select, labelled thumbs
```

---

## 0. Scope

Unifies **Catalog · Assembly slot · Variants · Atlas** preview surfaces under one **variant-state strip** artists can scrub without opening Variants tab.

**Not in v2:** ship-render fidelity · Bevy live viewport (separate `interactive` chip).

---

## 1. Four visual states (canonical)

| State id | Artist label | Variant axes | Thumb caption |
|:---|:---|:---|:---|
| `clean` | **Clean** | `lighting: day` · `damage.state: clean` | `Clean` |
| `night` | **Night** | `lighting: night_on` · `night_lights: true` | `Night` |
| `damaged` | **Damaged** | `damage.state: damaged` · `damage: 0.45` | `Damaged` |
| `burning` | **Burning** | `damage.state: damaged` + emissive overlay hook | `Burning` |

**Burning:** reuse damaged mesh + orange emissive rim in quick renderer — no separate GLB in v2.

---

## 2. State strip UI

```text
┌ Piece previews ─────────────────────────────────────┐
│ State  [ Clean ] [ Night ] [ Damaged ] [ Burning ]  │  ← single-select chips
│ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐       │
│ │ Module │ │Material│ │Combined│ │Context │       │
│ │ thumb  │ │ thumb  │ │ thumb  │ │ strip  │       │
│ │ Clean  │ │ Clean  │ │ Clean  │ │        │       │
│ └────────┘ └────────┘ └────────┘ └────────┘       │
│ ⟳ Rendering…  (inline — not log-only)              │
└────────────────────────────────────────────────────┘
```

| Rule | Spec |
|:---|:---|
| Default state | **Clean** on first select |
| Remember | `aps_preview_prefs.last_variant_state` per session |
| Debounce | **300ms** after select before re-render (smoothness charter) |
| Async | `⟳ Rendering…` on all four thumbs until job completes |
| Chip label | State name under each thumb (8px caption) — not color-alone |

---

## 3. Async-on-select flow

```text
1. User selects footprint cell OR catalog module
2. All thumbs → preview_surface_state("loading")
3. Job queue: module · material · combined · context (parallel OK)
4. On complete → apply_preview_photo + caption "{State}"
5. On fail → ◐ partial + hint adjacent (status_atom)
```

**Cancel:** new select cancels in-flight job — no stale thumb flash.

---

## 4. Empty-state copy (per thumb)

| Surface | Empty | Loading | Error |
|:---|:---|:---|:---|
| Module | `○ No piece selected` | `⟳ Rendering…` | `◐ No 3D file — validate or pick another module` |
| Material | `○ No material` | `⟳ Rendering…` | `◐ Material preview unavailable` |
| Combined | `○ Select a piece` | `⟳ Rendering…` | `◐ Combined preview unavailable — check module and material` |
| Context | `○ No placement` | `⟳ Rendering…` | `◐ Layout view still works` |

**Catalog list row:** module thumb shows `○ No preview` until first hover/select triggers async load.

---

## 5. Surfaces in scope

| Panel | State strip | Thumbs |
|:---|:---:|:---|
| `slot_preview_panel.py` | **yes** | 4-up grid |
| `assembly_preview_panel.py` | **yes** | assembly hero + state strip |
| `catalog.py` list row | scrub on row focus | 1 thumb |
| `atlas_preview_panel.py` | **yes** | selected tile cell |
| `material_preview_modes.py` | no (material-only) | sphere/wall/section unchanged |

---

## 6. Fidelity chips (unchanged)

| Chip | Meaning |
|:---|:---|
| `Quick preview` | slot/module thumbs |
| `Layout view` | placement context strip |
| `Interactive 3D` | browser / Bevy button — separate action |

---

## 7. Acceptance

| # | Check |
|:---:|:---|
| P1 | State chip change re-renders all in-scope thumbs within 300ms debounce |
| P2 | Four states visually distinct on warehouse kit002 pilot |
| P3 | No `GEN`/`ERR` placeholders — status_atom only |
| P4 | Empty copy matches §4 verbatim |
| P5 | Witness `debug_runs/aps_preview_v2_live.json` → `four_state_strip: true` |

---

## 8. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-18 |
