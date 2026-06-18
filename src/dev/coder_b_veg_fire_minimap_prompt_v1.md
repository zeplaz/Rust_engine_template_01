# @coder B — veg · fire · minimap tail prompt `v1`

Copy-paste into chat. Plan: [`plan_coder_b_veg_fire_minimap_tail_001_v1.md`](plan_coder_b_veg_fire_minimap_tail_001_v1.md)

---

```
@coder B — POST-PHASE-6 TAIL · pick seq 1→3 (one primary per session, ≤3 files)

Context: Phase 6 + CDR-B wave-0 rows are done on paper. Veg burn SIM + extract BUILD are green (@coder A).
Your lane: **consumers** — catalog loader, minimap compositor merge, fire/veg coherence, HUD legend.

Read first (authority):
  .cursor/skills/bevy-simulation-grade/07-repo-authority-map.md
  src/dev/plan_veg_burn_extract_001_v1.md §1 (single writers)
  src/dev/plan_veg_variant_key_naming_v1.md Q4a (burn wins on minimap tint)
  src/dev/design_minimap_veg_legend_v1.md (legend UI — seq 4)

Primary pick — VEG-CATALOG-LOADER-001:
  Mirror src/construction/procedural/tile_variant_resolver.rs VariantCatalog load pattern.
  Load assets/configs/landscape/_vegetation_variant_catalog.ron → Resource.
  Witness: debug_runs/vegetation_variant_catalog_load_live.json
  Exit: all 13 veg_* keys from veg_resolver_known_keys_v1.md present in catalog.entries

Then — VEG-MINIMAP-BURN-MERGE-001 (depends catalog optional, extract required):
  src/render/minimap_compositor/pass.rs — read Res<VegetationExtractFrame> (after FireVisualFrameSet::BuildProfiles).
  When row.burn_active && variant_key.starts_with("veg_burn_") → override ecology chunk tint.
  Do NOT ECS-scan fire/veg components in compositor.
  Witness: extend debug_runs/minimap_compositor_live.json
  Exit: veg_burn_rows >= 1, burn_overrides_topology: true (harness or sim refresh)

Then — FIRE-MINIMAP-COHERENCE-001:
  Align SharedOverlayFieldBuffers.revision with VegetationExtractFrame.revision in witness.
  Keep simulation_minimap_overlay_defaults: fire_heat off, ecology_heat on.
  Witness: debug_runs/minimap_fire_veg_coherence_live.json

Do NOT:
  · Touch landscape_grammar_burn.rs sim writers (A lane)
  · Re-pick BUILD-GRAMMAR / phase6 done rows
  · Mark done on lib test only — exit_predicate on witness JSON
  · Second fire extract or global Tree ECS

Verify:
  cargo test -p proc_A_dine01 --lib landscape_grammar minimap_compositor vegetation_visual_extract veg_resolver
  validate-report cargo --compress 3

Report: slice ID, files touched, witness path, green fields only (validation-first).
```

---

## Optional follow-ups (after seq 1–3 green)

| ID | One-liner |
|:---|:---|
| VEG-MINIMAP-LEGEND-UI-001 | HUD legend chips — not tint-only proof |
| VEG-LG5-STAMP-RUNTIME-001 | Runtime atlas UV stamp |
| S7B-M3-MINIMAP-001 | Stage 7 recon/logistics minimap readers |
