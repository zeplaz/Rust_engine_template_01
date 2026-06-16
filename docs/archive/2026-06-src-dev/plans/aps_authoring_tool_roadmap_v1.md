# APS-UX-AUTHORING-001 — Assembly Authoring Tool (not data editor) `v1`



| Field | Value |

|:---|:---|

| **Track** | **A — APS Product** |

| **Parent** | [`plan_three_track_execution_v1.md`](plan_three_track_execution_v1.md) |

| **Priority stack** | P0 ARCH-MAT-001 → P1 APS-PREVIEW-001 → P2 APS-MAT-002 → … see three-track plan |



---



## Gap (today)



| Field shown | Artist question unanswered |

|:---|:---|

| `Material: steel_panel_01` | What does that look like? |

| Placement tags | Why was this placement generated? |

| `Module: wall_steel_1u` | What mesh / GLB is this? |

| Validation line | Where am I in the pipeline? |



Target = **Assembly Authoring Tool** (not raw data editor).



---



## P1 — APS-PREVIEW-001 (selected slot) — **done** 2026-06-03



Spec: [`aps_preview_001_spec_v1.md`](aps_preview_001_spec_v1.md)



```text

Selected slot previews

----------------------

[Module isolated]  [Material wall+sphere]  [Combined]  [Placement context]



Module:    wall_industrial_a

Material:  steel_panel_01

Mesh:      assets/.../model.glb

Why:       Cell (2,4) · massing=yard_complex · facade=facade_v1 · seed=43

```



Implementation: [`slot_preview_panel.py`](../../tools/mcp/art_pipeline_suite/slot_preview_panel.py)



**Follow-up:** pipe assembly-level Bevy thumb into context panel after “Preview assembly”.



---



## P2 — APS-MAT-002 (material browser at scale)



Not `Combobox(material_profiles)` for 300 entries.



```text

Materials

  Industrial

    Steel

    Corrugated

    Concrete

  Residential

    Brick

    Plaster

```



Reuse [`material_library_widget.py`](../../tools/mcp/art_pipeline_suite/material_library_widget.py); elevate to Materials tab primary surface.



---



## P3 — Grammar inspector (Republic-style)



**Why was this generated?** Example header:



```text

Warehouse_Industrial_042

Archetype: Industrial Warehouse

Massing:   Long Hall

Roof:      Sawtooth

Facade:    Factory Window Grid

Material:  Weathered Steel (age + district)

Seed:      832991

```



Expanded in [`grammar_inspector.py`](../../tools/mcp/art_pipeline_suite/grammar_inspector.py); per-cell hint on slot select.



---



## P4+ — Track C grammar layers



[`plan_building_grammar_evolution_v1.md`](plan_building_grammar_evolution_v1.md) — archetype → massing → facade → roof → detail → **material strategy** → aging.



Weak grammar → 100 identical buildings regardless of preview quality.



---



## Done (do not redo)



- Footprint grid + grammar generate  

- Material library click-apply (assign mode)  

- Assembly-level Bevy/browser preview  

- P0 grammar gate before bake  

- ARCH-MAT-001 rule doc  


