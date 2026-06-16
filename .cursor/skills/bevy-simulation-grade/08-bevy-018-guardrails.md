# Bevy 0.18 Guardrails — `Rust_engine_template_01`

This engine uses **Bevy 0.18**. Use [official migration guides](https://bevy.org/learn/migration-guides/) when bumping to 0.19+.

## In scope for this template

| Bevy 0.18 area | Engine use |
|----------------|------------|
| **PBR / atmosphere** | World 3D accents; tactical map is mostly 2D/tile — do not bypass snapshot extract for “prettier PBR” in sim spine |
| **FullscreenMaterial** | Post-FX only via render graph review — not ad-hoc UI hacks |
| **Cargo features `2d` / `3d` / `ui`** | Prefer feature collections when splitting crates |
| **Unified errors** | `Result` at IO boundaries; structured logs in authority systems |
| **remove_systems_in_set** | Editor vs sim plugin opt-out — not for per-frame toggles (use run conditions) |
| **EntityRef / EntityMut** | `world.query::<EntityRef>()` — `iter_entities()` deprecated (0.17+) |

## Out of scope / do not default

| Feature | Rule |
|---------|------|
| **FreeCamera / PanCamera** (`bevy_camera_controller`) | Dev tooling only — **never** replace `MapCameraSystemSet` / `ViewProjectionAuthority` on WorldMain |
| **Solari** raytracing | Experimental — not Stage 5 spine |
| **Stock logical UI widgets** | OK for HUD; must not write sim or viewport commit |
| **AI / diffusion assets** | Art pipeline — not Bevy ECS |

## Rendering trends (0.16–0.18) vs this repo

- **GPU-driven mesh rendering** — helps large 3D; your hot path is **tile/GPU raster + extraction graph**. Authority model unchanged: sim → snapshot → extract.
- **Seekable asset readers** — aligns with MCP staging; loaders stay deterministic.

## API habits for agents

```rust
// Prefer explicit sets (repo pattern)
app.configure_sets(Update, (
    ViewRepresentationSystemSet::UiCollect,
    ViewRepresentationSystemSet::ResolveViewport,
    // ...
).chain());

// Inspect entities (0.17+)
fn inspect(world: &World) {
    for entity in world.query::<EntityRef>().iter(world) {
        let _ = entity.get::<Transform>();
    }
}

// Parallel sim — never touch view authority
fn sim_only(mut q: Query<&mut SimCell>) {
    q.par_iter_mut().for_each(|mut c| { /* ... */ });
}
```

## Before using a new Bevy API

1. Confirm **0.18** docs / context7 — not 0.12 blog posts.
2. Check **schedule impact** against [07-repo-authority-map.md](07-repo-authority-map.md).
3. Run **`validate-report bevy`** after API-heavy edits (project **validation-first** skill).

## Version bump checklist

- [ ] Read 0.18 → 0.19 migration guide (when upgrading)
- [ ] `cargo check -p proc_A_dine01` + `validate-report cargo`
- [ ] Re-verify `ViewAuthoritySystemSet` ordering in `view_authority.rs`
- [ ] Stage 5 regression: `cargo test -p proc_A_dine01 --lib stage5`
