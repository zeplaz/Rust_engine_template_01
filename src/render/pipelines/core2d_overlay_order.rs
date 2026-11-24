//! Bevy Core2d overlay pass ordering (replaces `RenderGraph` sub-graph edges).
//!
//! # RND-03 — geometry sorting for Core2d overlay layers (V4-RND03)
//!
//! Dual-cite: NEW **RND-03** (gea4v2 11.5.2 *Geometry Sorting* p137-138;
//! 11.5.3.1 *Quadtrees and Octrees* p138-140; 11.5.3.3 *Bounding Volume Hierarchies*
//! p141-142) · LEGACY keyword `geometry_visual_effects` (gea_v2_e4 §11.6 p141; no
//! dedicated `draw_order` / `bvh` / `quadtree` keyword hit). Pair: `pipeline_app_layer`
//! (gea_v2_e4 §11.5 p133) for the Core2d app-layer chain.
//!
//! The book's *sorted submission* of geometry (painter / front-to-back) maps here to a
//! **fixed layer table** — not a runtime BVH. Map overlays are few, opaque-or-additive
//! passes with a known composite contract: water surface → flood field → particles → tile debug →
//! weather precip → **fire sparks last** (additive sparks vanish if buried). (The per-tile terrain
//! layer left with GS-03: terrain draws from the world bake texture, not a Core2d overlay.)
//!
//! Contract:
//! - [`CORE2D_OVERLAY_DRAW_ORDER`] is the sole authority for relative draw order.
//! - [`Core2dOverlayOrderPlugin`] chains those sets after `Core2dSystems::MainPass` and
//!   before `EarlyPostProcess` — sole `configure_sets` writer for this chain.
//! - Passes join via `.in_set(Core2dOverlaySet::…)` only; they never invent sibling order.
//!
//! Debt not taken: dynamic BVH/quadtree culling for overlays; voxel/massing presentation
//! slots stay on world_map / pixel_world (RND-09), not this Core2d chain; transparency /
//! OIT layering is RND-04 (`fire_vfx` / `weather_vfx`).

use bevy::core_pipeline::{Core2d, Core2dSystems};
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::RenderApp;

/// SDR Core2d overlay pipelines are compiled for this format (must match RTT targets).
pub const CORE2D_OVERLAY_SDR_FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;

/// Locked Core2d overlay layer count (must match [`CORE2D_OVERLAY_DRAW_ORDER`]).
#[allow(dead_code)] // RND-03 order contract; read by the tripwire tests
pub const CORE2D_OVERLAY_LAYER_COUNT: usize = 6;

/// Index into `[sdr, hdr]` overlay pipeline tables (`0` = SDR, `1` = HDR float).
#[inline]
#[must_use]
pub fn core2d_overlay_pipeline_hdr_index(target_format: TextureFormat) -> usize {
    usize::from(!matches!(
        target_format,
        TextureFormat::Rgba8UnormSrgb | TextureFormat::Bgra8UnormSrgb
    ))
}

/// Ordered overlay raster passes chained after stock Core2d main pass.
///
/// Painter's order: lower [`Core2dOverlaySet::draw_rank`] draws first (under).
/// **Fire sparks must be last** among map overlays — terrain/weather opaque or
/// heavy draws after fire bury the additive spark pass (tint survives; sparks vanish).
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Core2dOverlaySet {
    WaterSurface,
    /// DF-3 — live water field (sim depth / flow) over the static water surface.
    WaterField,
    WaterParticleRaster,
    TileDebug,
    /// EFFECTS-SYSTEM ES-5-002 — precip streaks under sparks.
    WeatherPrecipRaster,
    /// P2-FIRE-SPARK-010 — additive sparks end the overlay chain (visible on RTT).
    FireParticleRaster,
}

impl Core2dOverlaySet {
    /// Painter's rank: `0` draws first (under); max draws last (on top).
    #[inline]
    #[must_use]
    #[allow(dead_code)] // RND-03 order contract; read by the tripwire tests
    pub const fn draw_rank(self) -> u8 {
        match self {
            Self::WaterSurface => 0,
            Self::WaterField => 1,
            Self::WaterParticleRaster => 2,
            Self::TileDebug => 3,
            Self::WeatherPrecipRaster => 4,
            Self::FireParticleRaster => 5,
        }
    }

    /// Stable debug / tripwire name (matches enum variant spelling).
    #[inline]
    #[must_use]
    #[allow(dead_code)] // RND-03 order contract; read by the tripwire tests
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WaterSurface => "WaterSurface",
            Self::WaterField => "WaterField",
            Self::WaterParticleRaster => "WaterParticleRaster",
            Self::TileDebug => "TileDebug",
            Self::WeatherPrecipRaster => "WeatherPrecipRaster",
            Self::FireParticleRaster => "FireParticleRaster",
        }
    }
}

/// Sole draw-order authority: back → front (index 0 under, last on top).
///
/// RND-03: geometry sorting without a runtime spatial tree — fixed submission order.
#[allow(dead_code)] // RND-03 order contract; read by the tripwire tests
pub const CORE2D_OVERLAY_DRAW_ORDER: [Core2dOverlaySet; CORE2D_OVERLAY_LAYER_COUNT] = [
    Core2dOverlaySet::WaterSurface,
    Core2dOverlaySet::WaterField,
    Core2dOverlaySet::WaterParticleRaster,
    Core2dOverlaySet::TileDebug,
    Core2dOverlaySet::WeatherPrecipRaster,
    Core2dOverlaySet::FireParticleRaster,
];

pub struct Core2dOverlayOrderPlugin;

impl Plugin for Core2dOverlayOrderPlugin {
    fn build(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        // Order must match `CORE2D_OVERLAY_DRAW_ORDER` (rnd03_* tripwires scan this site).
        render_app.configure_sets(
            Core2d,
            (
                Core2dOverlaySet::WaterSurface,
                Core2dOverlaySet::WaterField,
                Core2dOverlaySet::WaterParticleRaster,
                Core2dOverlaySet::TileDebug,
                Core2dOverlaySet::WeatherPrecipRaster,
                Core2dOverlaySet::FireParticleRaster,
            )
                .chain()
                .after(Core2dSystems::MainPass)
                .before(Core2dSystems::EarlyPostProcess),
        );
    }
}

#[cfg(test)]
mod rnd03_overlay_order_tests {
    //! RND-03 tripwires: locked painter's order + pass membership (no GPU).
    use super::*;
    use std::path::PathBuf;

    fn this_source() -> String {
        include_str!("core2d_overlay_order.rs").to_string()
    }

    fn production_source(text: &str) -> String {
        let mut out = String::new();
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let t = line.trim();
            if t == "#[cfg(test)]"
                && lines
                    .get(i + 1)
                    .is_some_and(|n| n.trim_start().starts_with("mod "))
            {
                break;
            }
            if t.starts_with("//") {
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
        out
    }

    fn crate_src() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
    }

    /// `(relative path, production source)` for every overlay pass that joins the chain.
    fn overlay_pass_sources() -> Vec<(String, String)> {
        let roots = [
            crate_src().join("render/pipelines"),
            crate_src().join("render/weather_vfx"),
        ];
        let mut v = Vec::new();
        for root in roots {
            for entry in std::fs::read_dir(&root).expect("read overlay pass dir") {
                let path = entry.expect("dir entry").path();
                if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                    continue;
                }
                let text = std::fs::read_to_string(&path).expect("read source");
                if !text.contains("Core2dOverlaySet::") {
                    continue;
                }
                let name = path
                    .strip_prefix(crate_src())
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                v.push((name, production_source(&text)));
            }
        }
        v.sort_by(|a, b| a.0.cmp(&b.0));
        v
    }

    #[test]
    fn rnd03_draw_order_table_is_dense_and_ranked() {
        assert_eq!(CORE2D_OVERLAY_DRAW_ORDER.len(), CORE2D_OVERLAY_LAYER_COUNT);
        for (i, layer) in CORE2D_OVERLAY_DRAW_ORDER.iter().enumerate() {
            assert_eq!(
                layer.draw_rank() as usize,
                i,
                "{} rank {} != table index {i}",
                layer.as_str(),
                layer.draw_rank()
            );
        }
        assert_eq!(
            CORE2D_OVERLAY_DRAW_ORDER.last().copied(),
            Some(Core2dOverlaySet::FireParticleRaster),
            "fire sparks must remain last (P2-FIRE-SPARK-010 / RND-03)"
        );
    }

    #[test]
    fn rnd03_plugin_configure_sets_matches_draw_order_table() {
        let src = production_source(&this_source());
        // Extract the configure_sets tuple body between the outer parens after Core2d,
        let marker = "configure_sets(\n            Core2d,\n            (";
        let start = src
            .find(marker)
            .expect("plugin must configure_sets(Core2d, (…))");
        let after = &src[start + marker.len()..];
        let end = after
            .find(')')
            .expect("configure_sets tuple must close");
        let tuple = &after[..end];
        let mut found = Vec::new();
        for layer in CORE2D_OVERLAY_DRAW_ORDER {
            let needle = format!("Core2dOverlaySet::{}", layer.as_str());
            let pos = tuple
                .find(&needle)
                .unwrap_or_else(|| panic!("configure_sets missing {needle}"));
            found.push((pos, layer.as_str()));
        }
        found.sort_by_key(|(pos, _)| *pos);
        let ordered: Vec<&str> = found.into_iter().map(|(_, n)| n).collect();
        let expected: Vec<&str> = CORE2D_OVERLAY_DRAW_ORDER
            .iter()
            .map(|l| l.as_str())
            .collect();
        assert_eq!(
            ordered, expected,
            "configure_sets chain order drifted from CORE2D_OVERLAY_DRAW_ORDER"
        );
        assert!(
            src.contains(".chain()")
                && src.contains("after(Core2dSystems::MainPass)")
                && src.contains("before(Core2dSystems::EarlyPostProcess)"),
            "overlay chain must stay MainPass → overlays → EarlyPostProcess"
        );
    }

    #[test]
    fn rnd03_each_overlay_set_has_exactly_one_pass_membership() {
        // Every locked layer is joined by exactly one production `in_set(...)` site.
        let sources = overlay_pass_sources();
        assert!(
            !sources.is_empty(),
            "expected overlay pass sources under pipelines/ + weather_vfx/"
        );
        for layer in CORE2D_OVERLAY_DRAW_ORDER {
            let needle = format!("in_set(Core2dOverlaySet::{})", layer.as_str());
            let mut hits = Vec::new();
            for (file, src) in &sources {
                if file.contains("core2d_overlay_order") {
                    continue;
                }
                let count = src.matches(&needle).count();
                if count > 0 {
                    hits.push((file.clone(), count));
                }
            }
            assert_eq!(
                hits.len(),
                1,
                "{}: expected exactly one pass file with `{needle}`, got {:?}",
                layer.as_str(),
                hits
            );
            assert_eq!(
                hits[0].1, 1,
                "{}: `{}` referenced {}× in {}",
                layer.as_str(),
                needle,
                hits[0].1,
                hits[0].0
            );
        }
    }

    #[test]
    fn rnd03_no_ad_hoc_overlay_set_outside_draw_order_table() {
        // Any `Core2dOverlaySet::Foo` in pass sources must be a table member.
        let allowed: Vec<&str> = CORE2D_OVERLAY_DRAW_ORDER
            .iter()
            .map(|l| l.as_str())
            .collect();
        for (file, src) in overlay_pass_sources() {
            if file.contains("core2d_overlay_order") {
                continue;
            }
            let mut rest = src.as_str();
            while let Some(at) = rest.find("Core2dOverlaySet::") {
                let after = &rest[at + "Core2dOverlaySet::".len()..];
                let name: String = after
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect();
                assert!(
                    allowed.contains(&name.as_str()),
                    "{file}: uses Core2dOverlaySet::{name} which is not in CORE2D_OVERLAY_DRAW_ORDER"
                );
                rest = &after[name.len()..];
            }
        }
    }

    #[test]
    fn rnd03_fire_rank_is_strictly_above_weather_and_water() {
        assert!(
            Core2dOverlaySet::FireParticleRaster.draw_rank()
                > Core2dOverlaySet::WeatherPrecipRaster.draw_rank()
        );
        assert!(
            Core2dOverlaySet::WeatherPrecipRaster.draw_rank()
                > Core2dOverlaySet::WaterField.draw_rank()
        );
        // The live flood field draws over the static water surface.
        assert!(
            Core2dOverlaySet::WaterField.draw_rank() > Core2dOverlaySet::WaterSurface.draw_rank()
        );
    }

    #[test]
    fn rnd03_hdr_index_contract() {
        assert_eq!(
            core2d_overlay_pipeline_hdr_index(CORE2D_OVERLAY_SDR_FORMAT),
            0
        );
        assert_eq!(
            core2d_overlay_pipeline_hdr_index(TextureFormat::Bgra8UnormSrgb),
            0
        );
        assert_eq!(
            core2d_overlay_pipeline_hdr_index(TextureFormat::Rgba16Float),
            1
        );
    }
}
