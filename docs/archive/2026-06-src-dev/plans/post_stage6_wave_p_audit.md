# Wave P consumer audit (WP-B03)

**Date:** 2026-05-23  
**Scope:** Preview composite path must not mutate gameplay ECS.

## Findings

| Consumer | Mutates gameplay? | Notes |
|----------|-------------------|-------|
| `composite_preview_graph.rs` | No | Builds RGBA from layer flags + chunk data reads |
| `render_raster.rs` | No | Writes swap buffers / preview texture only |
| `gpu_preview.rs` | No | Offscreen camera + GPU RT for preview surface |
| `wave_p_readiness.rs` | No | Readiness DTO only |
| `wave_p_live_proof.rs` | No | JSON witness writer |

## Authority

- `PreviewPathAuthority` selects CPU vs GPU surface; does not replace `WorldRepresentationFrame`.
- Material/tag edits belong in asset tools or sim systems — not preview raster.

## Open

- Inspector table (BQ / Wave P non-goals) remains external tool or deferred egui F8.
