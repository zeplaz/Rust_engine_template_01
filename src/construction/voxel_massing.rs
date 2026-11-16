//! VoxelMassing — presentation-only column stacks derived from XY footprint cells.
//!
//! Contract: `src/dev/plan_voxel_massing_presentation_v1.md` · X2-P3 RO bridge in
//! `plan_voxel_occupation_v1.md`.
//! Sim occupation stays `FootprintMatrix` / weighted raster until cutover. Columns never
//! enter commit payloads.
//!
//! **Dual-view:** massing columns paint only under
//! [`crate::gui::WorldPresentationKind::PixelWorld3D`]. TacticalFlat2D keeps flat
//! footprint / icon tokens (see `plan_dual_view_pixel_world_v1.md`).
//!
//! **V4-ECS15 / ECS-15 (gea4v2 16.2.2) — tool-side vs runtime (Studio consumer):**
//! - **Tool:** ParamFrame / `massing_preview.json` with `sim_occupation: false` →
//!   [`StudioMassingOverride`]. Must not write voxel/tile occupation or commit payloads.
//! - **Runtime:** sole writer [`sync_voxel_massing_visual_requests`] → `massing_columns` /
//!   `massing_instances`. Studio columns are never sim occupation.
//! - Same book split as IF-TRG authoring vs SIM-TRN runtime in transport.
//!   Payload with `sim_occupation: true` is **refused** by [`parse_massing_preview_json`].
//!
//! **Lane B (PCG-06 · ECS-15 · RND-01/09):** sole writer of
//! [`ConstructionVisualRequests::massing_columns`] and the RND-09
//! [`ConstructionVisualRequests::massing_instances`] batch (packed when column count ≥
//! [`MASSING_INSTANCE_BATCH_THRESHOLD`]). Studio ingest is tool/runtime split (IO-09
//! export → engine hydrate); pixel_world / world_map consumers are **read-only** (RND-01 app paint).
//! Optional studio override via [`StudioMassingOverride`] (explicit set / one-shot env load) —
//! never auto-polls disk every frame; never writes occupation or [`FootprintMatrix`].
//!
//! **X2-P3 (PCG-06 · ECS-15):** when [`VoxelOccupationConfig`] commit or derived-mirror RO
//! is on, [`sync_voxel_massing_visual_requests`] may **read** [`VoxelOccupationBook`] for
//! per-tile `z_layers` floor intent. Massing never mutates the book.

use std::path::Path;

use bevy::prelude::*;
use serde::Deserialize;

use crate::construction::building_catalog::BuildingFamily;
use crate::construction::visual_authority::{
    ConstructionVisualRequests, FootprintTileColorKind, FootprintTileRequest,
};
use crate::strategic::{VoxelOccupationBook, VoxelOccupationConfig};

/// Disposable column stack for world-map silhouette (not sim occupation).
#[derive(Clone, Debug, PartialEq)]
pub struct VoxelColumnRequest {
    pub tile: IVec2,
    /// Stacked visual floors — clamped **1..=4** at operational zoom.
    pub floors: u8,
    pub color_kind: FootprintTileColorKind,
    pub weight: f32,
}

/// Max floors painted as stacked quads (readability / Ebbtide LOD band).
pub const MASSING_FLOORS_MAX: u8 = 4;

/// Below this screen px/tile, massing collapses to floors=1 (footprint grid read).
/// Contract: `plan_voxel_massing_presentation_v1.md` §4.
pub const MASSING_MIN_PX_PER_TILE: f32 = 24.0;

/// RND-09 (gea4v2 11.3.8.2): when live column count reaches this, the sole writer packs
/// [`ConstructionVisualRequests::massing_instances`] so consumers paint from one instance
/// buffer instead of expanding per-column geometry ad hoc. Below threshold the buffer is
/// cleared and consumers keep the direct `massing_columns` path.
pub const MASSING_INSTANCE_BATCH_THRESHOLD: usize = 64;

/// Packed column instance row — one logical mesh, many placements (RND-09).
/// Lane B packs; Lane C reads RO. Does not replace [`VoxelColumnRequest`] semantics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MassingColumnInstance {
    pub tile: IVec2,
    pub floors: u8,
    pub color_kind: FootprintTileColorKind,
    pub weight: f32,
}

impl From<&VoxelColumnRequest> for MassingColumnInstance {
    fn from(c: &VoxelColumnRequest) -> Self {
        Self {
            tile: c.tile,
            floors: c.floors,
            color_kind: c.color_kind,
            weight: c.weight,
        }
    }
}

/// True when consumers should prefer the RND-09 instance batch over per-column expand.
#[must_use]
pub fn massing_uses_instance_batch(column_count: usize) -> bool {
    column_count >= MASSING_INSTANCE_BATCH_THRESHOLD
}

/// Pack columns into instance rows when the count gate is met; otherwise empty.
#[must_use]
pub fn pack_massing_instances(columns: &[VoxelColumnRequest]) -> Vec<MassingColumnInstance> {
    if !massing_uses_instance_batch(columns.len()) {
        return Vec::new();
    }
    columns.iter().map(MassingColumnInstance::from).collect()
}

/// APS / studio_dash export path (repo-relative).
pub const STUDIO_MASSING_PREVIEW_REL: &str = "debug_runs/studio_dash/massing_preview.json";

/// Checked-in pilot-derived fixture (grammar lineage; presentation-only).
pub const STUDIO_MASSING_CHECKED_IN_FIXTURE_REL: &str =
    "assets/configs/buildings/examples/massing_preview_pilot_rail_warehouse_v1.json";

/// Env opt-in: when set to a non-empty value other than `0`/`false`/`off`, startup loads
/// [`STUDIO_MASSING_PREVIEW_REL`] once into [`StudioMassingOverride`]. No per-frame disk poll.
/// Set to a path to load that file instead; `fixture` loads [`STUDIO_MASSING_CHECKED_IN_FIXTURE_REL`].
pub const STUDIO_MASSING_ENV: &str = "RUST_ENGINE_STUDIO_MASSING";

/// Explicit presentation override from APS `massing_preview.json`.
/// Default empty → sync uses catalog derive. Set via [`StudioMassingOverride::set_columns`],
/// [`StudioMassingOverride::set_from_json`], [`StudioMassingOverride::load_from_path`], or
/// the one-shot env startup system — never polled every frame on the production path.
#[derive(Resource, Debug, Clone, Default)]
pub struct StudioMassingOverride {
    pub columns: Option<Vec<VoxelColumnRequest>>,
}

impl StudioMassingOverride {
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.columns.as_ref().is_some_and(|c| !c.is_empty())
    }

    pub fn clear(&mut self) {
        self.columns = None;
    }

    pub fn set_columns(&mut self, columns: Vec<VoxelColumnRequest>) {
        if columns.is_empty() {
            self.columns = None;
        } else {
            self.columns = Some(columns);
        }
    }

    /// Parse massing_preview-shaped JSON; ignore (clear) on missing/invalid.
    pub fn set_from_json(&mut self, text: &str) {
        match parse_massing_preview_json(text) {
            Some(cols) if !cols.is_empty() => self.columns = Some(cols),
            _ => self.columns = None,
        }
    }

    /// Load from disk once; ignore (clear) if missing/invalid.
    pub fn load_from_path(&mut self, path: impl AsRef<Path>) {
        match load_massing_preview_file(path) {
            Some(cols) if !cols.is_empty() => self.columns = Some(cols),
            _ => self.columns = None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct MassingPreviewFile {
    /// ECS-15: tool payloads must stay presentation-only. Default `false`; `true` → refuse load.
    #[serde(default)]
    sim_occupation: bool,
    #[serde(default)]
    massing_columns: Vec<MassingColumnJson>,
}

#[derive(Debug, Deserialize)]
struct MassingColumnJson {
    tile: [i32; 2],
    #[serde(default)]
    floors: u8,
    #[serde(default = "default_color_kind")]
    color_kind: String,
    #[serde(default = "default_weight")]
    weight: f32,
}

fn default_color_kind() -> String {
    "Valid".to_string()
}

fn default_weight() -> f32 {
    1.0
}

fn parse_color_kind(s: &str) -> FootprintTileColorKind {
    match s.trim().to_ascii_lowercase().as_str() {
        "risky" | "warn" => FootprintTileColorKind::Risky,
        "invalid" | "bad" => FootprintTileColorKind::Invalid,
        _ => FootprintTileColorKind::Valid,
    }
}

/// Pure loader: massing_preview-shaped JSON → columns (floors clamped 1..=4).
/// Returns `None` if JSON is invalid, claims `sim_occupation: true` (ECS-15), or has no
/// usable columns.
#[must_use]
pub fn parse_massing_preview_json(text: &str) -> Option<Vec<VoxelColumnRequest>> {
    let file: MassingPreviewFile = serde_json::from_str(text).ok()?;
    // Tool-side must not claim sim occupation — refuse rather than invent a second writer.
    if file.sim_occupation {
        return None;
    }
    let cols = columns_from_preview_entries(&file.massing_columns);
    if cols.is_empty() {
        None
    } else {
        Some(cols)
    }
}

/// Load + parse a massing_preview file. Missing/invalid → `None` (caller keeps catalog derive).
#[must_use]
pub fn load_massing_preview_file(path: impl AsRef<Path>) -> Option<Vec<VoxelColumnRequest>> {
    let text = std::fs::read_to_string(path).ok()?;
    parse_massing_preview_json(&text)
}

fn columns_from_preview_entries(entries: &[MassingColumnJson]) -> Vec<VoxelColumnRequest> {
    entries
        .iter()
        .filter(|e| e.weight > 0.001)
        .map(|e| VoxelColumnRequest {
            tile: IVec2::new(e.tile[0], e.tile[1]),
            floors: e.floors.clamp(1, MASSING_FLOORS_MAX),
            color_kind: parse_color_kind(&e.color_kind),
            weight: e.weight.clamp(0.05, 1.0),
        })
        .collect()
}

/// Apply operational-zoom readability collapse.
#[must_use]
pub fn massing_floors_for_zoom(floors: u8, px_per_tile: f32) -> u8 {
    let floors = floors.clamp(1, MASSING_FLOORS_MAX);
    if px_per_tile < MASSING_MIN_PX_PER_TILE {
        1
    } else {
        floors
    }
}

/// Family height bonus (0..2) — presentation heuristic only.
#[must_use]
pub fn family_height_bonus(family: Option<BuildingFamily>) -> u8 {
    match family {
        Some(BuildingFamily::Residential) => 1,
        Some(BuildingFamily::Retail) | Some(BuildingFamily::Civic) => 1,
        Some(BuildingFamily::Industry) | Some(BuildingFamily::Logistics) => 2,
        Some(BuildingFamily::Extraction) | Some(BuildingFamily::Fuel) => 1,
        Some(BuildingFamily::Power) => 2,
        Some(BuildingFamily::Research) | Some(BuildingFamily::Agriculture) => 1,
        Some(BuildingFamily::Rail) => 0,
        None => 0,
    }
}

/// Deterministic floors from occupied cell count + optional family bonus.
#[must_use]
pub fn derive_massing_floors(occupied_cells: u32, family: Option<BuildingFamily>) -> u8 {
    let size_band = if occupied_cells == 0 {
        0u32
    } else {
        // log2-ish band: 1→0, 2–3→1, 4–7→2, 8+→3
        let mut n = occupied_cells;
        let mut band = 0u32;
        while n > 1 && band < 3 {
            n >>= 1;
            band += 1;
        }
        band
    };
    let raw = 1u32 + size_band + u32::from(family_height_bonus(family));
    raw.min(u32::from(MASSING_FLOORS_MAX)) as u8
}

/// Build column requests from footprint tile requests (same cells, stacked height).
#[must_use]
pub fn columns_from_footprint_tiles(
    tiles: &[FootprintTileRequest],
    floors: u8,
) -> Vec<VoxelColumnRequest> {
    let floors = floors.clamp(1, MASSING_FLOORS_MAX);
    tiles
        .iter()
        .filter(|t| t.weight > 0.001)
        .map(|t| VoxelColumnRequest {
            tile: t.tile,
            floors,
            color_kind: t.color_kind,
            weight: t.weight,
        })
        .collect()
}

/// True when massing may RO-read voxel occupation for column height intent (X2-P3).
#[must_use]
pub fn massing_reads_voxel_column_intent(config: &VoxelOccupationConfig) -> bool {
    config.commit_authority_enabled || config.derived_mirror_enabled
}

/// Clamp voxel `z_layers` into presentation floor band.
#[must_use]
pub fn floors_from_voxel_layers(z_layers: u8) -> u8 {
    z_layers.clamp(1, MASSING_FLOORS_MAX)
}

/// Ghost footprint tiles → columns; occupied voxel cells supply per-tile floor intent.
/// Missing / near-zero weight cells keep `default_floors`. Never mutates `voxel`.
#[must_use]
pub fn columns_from_footprint_tiles_voxel_ro(
    tiles: &[FootprintTileRequest],
    default_floors: u8,
    voxel: &VoxelOccupationBook,
) -> Vec<VoxelColumnRequest> {
    let default_floors = default_floors.clamp(1, MASSING_FLOORS_MAX);
    tiles
        .iter()
        .filter(|t| t.weight > 0.001)
        .map(|t| {
            let floors = voxel
                .cells
                .get(&t.tile)
                .filter(|c| c.layers.weight > 0.001)
                .map(|c| floors_from_voxel_layers(c.layers.z_layers))
                .unwrap_or(default_floors);
            VoxelColumnRequest {
                tile: t.tile,
                floors,
                color_kind: t.color_kind,
                weight: t.weight,
            }
        })
        .collect()
}

fn apply_zoom_to_columns(cols: &[VoxelColumnRequest], px_per_tile: f32) -> Vec<VoxelColumnRequest> {
    cols.iter()
        .map(|c| VoxelColumnRequest {
            tile: c.tile,
            floors: massing_floors_for_zoom(c.floors, px_per_tile),
            color_kind: c.color_kind,
            weight: c.weight,
        })
        .collect()
}

/// One-shot Startup: if [`STUDIO_MASSING_ENV`] is truthy, load preview path once.
/// Never schedules a per-frame disk read.
///
/// | Value | Loads |
/// |---|---|
/// | `1` / `true` / `on` / `yes` | [`STUDIO_MASSING_PREVIEW_REL`] |
/// | `fixture` | [`STUDIO_MASSING_CHECKED_IN_FIXTURE_REL`] |
/// | other path | that path |
pub fn load_studio_massing_override_from_env_once(mut studio: ResMut<StudioMassingOverride>) {
    let Ok(raw) = std::env::var(STUDIO_MASSING_ENV) else {
        return;
    };
    let v = raw.trim().to_ascii_lowercase();
    if v.is_empty() || matches!(v.as_str(), "0" | "false" | "off" | "no") {
        return;
    }
    let path = if matches!(v.as_str(), "1" | "true" | "on" | "yes") {
        Path::new(STUDIO_MASSING_PREVIEW_REL)
    } else if v == "fixture" {
        Path::new(STUDIO_MASSING_CHECKED_IN_FIXTURE_REL)
    } else {
        Path::new(raw.trim())
    };
    studio.load_from_path(path);
}

/// After footprint sync — derive massing columns for the live ghost tiles only.
/// Staged book rows keep flat footprint (no extra height noise).
/// Sole writer of [`ConstructionVisualRequests::massing_columns`] and
/// [`ConstructionVisualRequests::massing_instances`] (RND-09 batch when gated).
///
/// Optional [`VoxelOccupationBook`] / [`VoxelOccupationConfig`]: when RO cutover flags are
/// on, reads per-tile `z_layers` for floor intent — never writes occupation.
pub fn sync_voxel_massing_visual_requests(
    strip: Res<crate::construction::build_strip::BuildStripState>,
    ghost: Res<crate::construction::build_state::BuildGhostState>,
    tool: Res<crate::construction::build_tool_authority::ActiveBuildTool>,
    settings: Res<crate::construction::tile_visual::ConstructionTileVisualSettings>,
    studio: Res<StudioMassingOverride>,
    world_mode: Option<Res<crate::gui::WorldPresentationMode>>,
    zoom: Option<Res<crate::gui::ZoomFrame>>,
    voxel_cfg: Option<Res<VoxelOccupationConfig>>,
    voxel_book: Option<Res<VoxelOccupationBook>>,
    mut requests: ResMut<ConstructionVisualRequests>,
) {
    use crate::construction::build_strip::ToolContext;
    // Tactical = flat occupation tokens only; PixelWorld owns massing silhouette.
    let pixel_world = world_mode
        .as_ref()
        .map(|m| m.is_pixel_world())
        .unwrap_or(false);
    if !pixel_world {
        return;
    }
    if strip.active == ToolContext::None || !settings.show_occupation_tiles {
        requests.massing_columns.clear();
        requests.massing_instances.clear();
        return;
    }
    let px = zoom.map(|z| z.px_per_tile).unwrap_or(MASSING_MIN_PX_PER_TILE);

    // Explicit studio override (Lane B ingest) — prefer when present.
    if studio.is_active() {
        if let Some(cols) = studio.columns.as_ref() {
            requests.massing_columns = apply_zoom_to_columns(cols, px);
            requests.massing_instances = pack_massing_instances(&requests.massing_columns);
            return;
        }
    }

    if ghost.origin.is_none() {
        requests.massing_columns.clear();
        requests.massing_instances.clear();
        return;
    }
    let occupied = requests.footprint_tiles.len() as u32;
    if occupied == 0 {
        requests.massing_columns.clear();
        requests.massing_instances.clear();
        return;
    }
    let family = tool.building_intent.as_ref().map(|i| i.family);
    let default_floors = derive_massing_floors(occupied, family);
    // Only the live ghost slice: footprint_lock_tiles marks Adjust cells; otherwise all
    // footprint_tiles that are not from staged book alone — use tiles pushed this frame
    // for the ghost by taking the lock set when Adjust, else first `occupied` from empty
    // massing: we re-derive from current footprint_tiles when lock empty (Preview).
    let ghost_tiles: Vec<FootprintTileRequest> = if !requests.footprint_lock_tiles.is_empty() {
        let lock: std::collections::HashSet<IVec2> =
            requests.footprint_lock_tiles.iter().copied().collect();
        requests
            .footprint_tiles
            .iter()
            .filter(|t| lock.contains(&t.tile))
            .cloned()
            .collect()
    } else {
        // Preview: massing on all footprint tiles (ghost + staged). Prefer ghost-sized
        // subset by matching occupied count from matrix when possible.
        requests.footprint_tiles.clone()
    };
    let use_voxel_ro = voxel_cfg
        .as_ref()
        .is_some_and(|c| massing_reads_voxel_column_intent(c));
    let cols = if use_voxel_ro {
        if let Some(book) = voxel_book.as_deref() {
            columns_from_footprint_tiles_voxel_ro(&ghost_tiles, default_floors, book)
        } else {
            columns_from_footprint_tiles(&ghost_tiles, default_floors)
        }
    } else {
        columns_from_footprint_tiles(&ghost_tiles, default_floors)
    };
    requests.massing_columns = apply_zoom_to_columns(&cols, px);
    requests.massing_instances = pack_massing_instances(&requests.massing_columns);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construction::build_state::BuildGhostState;
    use crate::construction::build_strip::{BuildStripState, ToolContext};
    use crate::construction::build_tool_authority::ActiveBuildTool;
    use crate::construction::tile_visual::ConstructionTileVisualSettings;
    use crate::construction::visual_authority::FootprintTileColorKind;
    use crate::gui::{WorldPresentationKind, WorldPresentationMode, ZoomFrame};
    use crate::strategic::{
        BuildSiteTile, OccupancyLayerStack, VoxelOccupationBook, VoxelOccupationCell,
        VoxelOccupationConfig,
    };
    use bevy::ecs::system::RunSystemOnce;

    const FIXTURE_JSON: &str = r#"{
        "schema_version": 1,
        "title": "massing_preview",
        "presentation_only": true,
        "sim_occupation": false,
        "seed": 42,
        "massing_columns": [
            {"tile": [1, 2], "floors": 3, "color_kind": "Valid", "weight": 1.0},
            {"tile": [0, 0], "floors": 9, "color_kind": "Risky", "weight": 0.5},
            {"tile": [-1, 0], "floors": 0, "color_kind": "Invalid", "weight": 1.0}
        ],
        "note": "VOXEL-MASSING-PRES-001 — never commit to occupation"
    }"#;

    #[test]
    fn floors_clamp_and_scale_with_occupied_count() {
        assert_eq!(derive_massing_floors(0, None), 1);
        assert_eq!(derive_massing_floors(1, None), 1);
        assert!(derive_massing_floors(8, Some(BuildingFamily::Industry)) <= MASSING_FLOORS_MAX);
        assert!(derive_massing_floors(16, Some(BuildingFamily::Industry)) >= 3);
    }

    #[test]
    fn massing_collapses_below_readability_px_per_tile() {
        assert_eq!(massing_floors_for_zoom(4, 23.9), 1);
        assert_eq!(massing_floors_for_zoom(4, MASSING_MIN_PX_PER_TILE), 4);
        assert_eq!(massing_floors_for_zoom(3, 48.0), 3);
    }

    #[test]
    fn columns_skip_near_zero_weight() {
        let tiles = vec![
            FootprintTileRequest {
                tile: IVec2::new(1, 2),
                color_kind: FootprintTileColorKind::Valid,
                weight: 1.0,
                fill_override: None,
            },
            FootprintTileRequest {
                tile: IVec2::new(2, 2),
                color_kind: FootprintTileColorKind::Valid,
                weight: 0.0001,
                fill_override: None,
            },
        ];
        let cols = columns_from_footprint_tiles(&tiles, 3);
        assert_eq!(cols.len(), 1);
        assert_eq!(cols[0].floors, 3);
        assert_eq!(cols[0].tile, IVec2::new(1, 2));
    }

    #[test]
    fn parse_massing_preview_fixture_clamps_floors() {
        let cols = parse_massing_preview_json(FIXTURE_JSON).expect("fixture parses");
        assert_eq!(cols.len(), 3);
        assert_eq!(cols[0].tile, IVec2::new(1, 2));
        assert_eq!(cols[0].floors, 3);
        assert_eq!(cols[0].color_kind, FootprintTileColorKind::Valid);
        // floors 9 → clamp 4
        assert_eq!(cols[1].floors, MASSING_FLOORS_MAX);
        assert_eq!(cols[1].color_kind, FootprintTileColorKind::Risky);
        // floors 0 → clamp 1
        assert_eq!(cols[2].floors, 1);
        assert_eq!(cols[2].color_kind, FootprintTileColorKind::Invalid);
    }

    #[test]
    fn parse_missing_or_invalid_returns_none() {
        assert!(parse_massing_preview_json("").is_none());
        assert!(parse_massing_preview_json("{not json").is_none());
        assert!(parse_massing_preview_json(r#"{"massing_columns":[]}"#).is_none());
        assert!(parse_massing_preview_json(r#"{"massing_columns":[{"tile":[0,0],"floors":1,"weight":0.0}]}"#).is_none());
        // V4-ECS15: tool payload claiming sim occupation is refused.
        assert!(parse_massing_preview_json(
            r#"{"sim_occupation":true,"massing_columns":[{"tile":[0,0],"floors":2,"weight":1.0}]}"#
        )
        .is_none());
    }

    /// V4-ECS15 — Studio / tool hydrate paints presentation only; never writes occupation.
    #[test]
    fn ecs15_studio_path_cannot_write_sim_occupation() {
        let mut app = massing_test_app();
        app.insert_resource(VoxelOccupationConfig {
            derived_mirror_enabled: true,
            commit_authority_enabled: true,
        });
        let mut voxel = VoxelOccupationBook::default();
        voxel.cells.insert(
            IVec2::new(8, 8),
            VoxelOccupationCell {
                xy: IVec2::new(8, 8),
                layers: OccupancyLayerStack {
                    weight: 0.5,
                    z_layers: 2,
                },
                site_id: None,
                corridor_reserve: false,
            },
        );
        let rev_before = voxel.revision;
        let cells_before = voxel.cells.len();
        app.insert_resource(voxel);
        app.world_mut()
            .resource_mut::<StudioMassingOverride>()
            .set_from_json(FIXTURE_JSON);
        assert!(
            app.world().resource::<StudioMassingOverride>().is_active(),
            "fixture with sim_occupation:false must hydrate"
        );

        app.world_mut()
            .run_system_once(sync_voxel_massing_visual_requests)
            .expect("sync runs");

        let book = app.world().resource::<VoxelOccupationBook>();
        assert_eq!(book.revision, rev_before);
        assert_eq!(book.cells.len(), cells_before);
        assert!(
            (book.weight_at(IVec2::new(8, 8)) - 0.5).abs() < 1e-5,
            "studio path must not mutate existing occupation cells"
        );
        let req = app.world().resource::<ConstructionVisualRequests>();
        assert_eq!(req.massing_columns.len(), 3);
        assert!(
            !req.massing_columns.iter().any(|c| c.tile == IVec2::new(8, 8)),
            "studio override must not invent occupation-backed columns"
        );
    }

    #[test]
    fn studio_override_set_from_json_and_clear() {
        let mut studio = StudioMassingOverride::default();
        assert!(!studio.is_active());
        studio.set_from_json(FIXTURE_JSON);
        assert!(studio.is_active());
        assert_eq!(studio.columns.as_ref().unwrap().len(), 3);
        studio.clear();
        assert!(!studio.is_active());
    }

    fn massing_test_app() -> App {
        let mut app = App::new();
        app.init_resource::<BuildStripState>()
            .init_resource::<BuildGhostState>()
            .init_resource::<ActiveBuildTool>()
            .init_resource::<ConstructionTileVisualSettings>()
            .init_resource::<StudioMassingOverride>()
            .init_resource::<ConstructionVisualRequests>()
            .insert_resource(WorldPresentationMode {
                kind: WorldPresentationKind::PixelWorld3D,
            })
            .insert_resource(ZoomFrame {
                px_per_tile: 48.0,
                ..Default::default()
            });
        app.world_mut().resource_mut::<BuildStripState>().active = ToolContext::Civil;
        app.world_mut().resource_mut::<BuildGhostState>().origin = Some(BuildSiteTile { x: 0, z: 0 });
        app
    }

    /// Checked-in grammar/pilot fixture → override without env (Phase 3 demo harness).
    #[test]
    fn checked_in_pilot_fixture_loads_override() {
        const CHECKED: &str = include_str!(
            "../../assets/configs/buildings/examples/massing_preview_pilot_rail_warehouse_v1.json"
        );
        let cols = parse_massing_preview_json(CHECKED).expect("checked-in fixture parses");
        assert!(
            cols.len() >= 4,
            "pilot L footprint should yield multiple columns, got {}",
            cols.len()
        );
        assert!(cols.iter().all(|c| (1..=MASSING_FLOORS_MAX).contains(&c.floors)));
        let mut studio = StudioMassingOverride::default();
        studio.set_from_json(CHECKED);
        assert!(studio.is_active());

        let mut app = massing_test_app();
        app.world_mut()
            .resource_mut::<StudioMassingOverride>()
            .set_from_json(CHECKED);
        app.world_mut()
            .run_system_once(sync_voxel_massing_visual_requests)
            .expect("sync runs");
        let n = app
            .world()
            .resource::<ConstructionVisualRequests>()
            .massing_columns
            .len();
        assert_eq!(n, cols.len());
    }

    #[test]
    fn sync_sole_writer_prefers_studio_override() {
        let mut app = massing_test_app();
        {
            let mut req = app.world_mut().resource_mut::<ConstructionVisualRequests>();
            req.massing_columns.push(VoxelColumnRequest {
                tile: IVec2::new(99, 99),
                floors: 2,
                color_kind: FootprintTileColorKind::Valid,
                weight: 1.0,
            });
            req.footprint_tiles.push(FootprintTileRequest {
                tile: IVec2::new(5, 5),
                color_kind: FootprintTileColorKind::Valid,
                weight: 1.0,
                fill_override: None,
            });
        }
        app.world_mut()
            .resource_mut::<StudioMassingOverride>()
            .set_from_json(FIXTURE_JSON);

        app.world_mut()
            .run_system_once(sync_voxel_massing_visual_requests)
            .expect("sync runs");

        let req = app.world().resource::<ConstructionVisualRequests>();
        assert_eq!(req.massing_columns.len(), 3);
        assert_eq!(req.massing_columns[0].tile, IVec2::new(1, 2));
        assert_eq!(req.massing_columns[0].floors, 3);
        // Stale (99,99) cleared — only this system writes massing_columns.
        assert!(!req.massing_columns.iter().any(|c| c.tile == IVec2::new(99, 99)));
        // Catalog footprint tile not used when studio active.
        assert!(!req.massing_columns.iter().any(|c| c.tile == IVec2::new(5, 5)));
    }

    #[test]
    fn sync_catalog_derive_when_studio_empty() {
        let mut app = massing_test_app();
        {
            let mut req = app.world_mut().resource_mut::<ConstructionVisualRequests>();
            req.footprint_tiles.push(FootprintTileRequest {
                tile: IVec2::new(3, 4),
                color_kind: FootprintTileColorKind::Valid,
                weight: 1.0,
                fill_override: None,
            });
        }
        assert!(!app.world().resource::<StudioMassingOverride>().is_active());

        app.world_mut()
            .run_system_once(sync_voxel_massing_visual_requests)
            .expect("sync runs");

        let req = app.world().resource::<ConstructionVisualRequests>();
        assert_eq!(req.massing_columns.len(), 1);
        assert_eq!(req.massing_columns[0].tile, IVec2::new(3, 4));
        assert_eq!(req.massing_columns[0].floors, 1); // occupied=1, no family
    }

    /// X0 / PQ-R2 / X2-P3 / V4-ECS15 — no sim VFX / atmosphere; no FootprintMatrix.
    /// RO `VoxelOccupationBook` import is allowed (never written from this module).
    #[test]
    fn lane_b_source_has_no_sim_vfx_imports() {
        let src = include_str!("voxel_massing.rs");
        for line in src.lines() {
            let t = line.trim();
            if !t.starts_with("use ") {
                continue;
            }
            for forbidden in ["weather", "fire", "atmosphere", "FootprintMatrix"] {
                assert!(
                    !t.contains(forbidden),
                    "voxel_massing must stay presentation-only; forbidden use: {line}"
                );
            }
        }
        let prod = src.split("#[cfg(test)]").next().unwrap_or(src);
        assert!(
            !prod.contains("ResMut<VoxelOccupationBook>")
                && !prod.contains("ResMut<TileOccupationBook>")
                && !prod.contains(".apply_site_commit")
                && !prod.contains(".remove_site_commit")
                && !prod.contains(".replace_from_tile_mirror")
                && !prod.contains(".apply_corridor_reserve_mask")
                && !prod.contains("TileOccupationBook"),
            "massing / Studio path must never write sim occupation books"
        );
    }

    #[test]
    fn voxel_ro_column_intent_uses_z_layers() {
        let tiles = vec![
            FootprintTileRequest {
                tile: IVec2::new(1, 1),
                color_kind: FootprintTileColorKind::Valid,
                weight: 1.0,
                fill_override: None,
            },
            FootprintTileRequest {
                tile: IVec2::new(2, 1),
                color_kind: FootprintTileColorKind::Valid,
                weight: 1.0,
                fill_override: None,
            },
        ];
        let mut voxel = VoxelOccupationBook::default();
        voxel.cells.insert(
            IVec2::new(1, 1),
            VoxelOccupationCell {
                xy: IVec2::new(1, 1),
                layers: OccupancyLayerStack {
                    weight: 1.0,
                    z_layers: 3,
                },
                site_id: None,
                corridor_reserve: false,
            },
        );
        let cols = columns_from_footprint_tiles_voxel_ro(&tiles, 1, &voxel);
        assert_eq!(cols.len(), 2);
        assert_eq!(cols[0].floors, 3);
        assert_eq!(cols[1].floors, 1); // no voxel cell → default
    }

    #[test]
    fn sync_reads_voxel_ro_when_commit_authority_on() {
        let mut app = massing_test_app();
        app.insert_resource(VoxelOccupationConfig {
            derived_mirror_enabled: false,
            commit_authority_enabled: true,
        });
        let mut voxel = VoxelOccupationBook::default();
        voxel.cells.insert(
            IVec2::new(3, 4),
            VoxelOccupationCell {
                xy: IVec2::new(3, 4),
                layers: OccupancyLayerStack {
                    weight: 1.0,
                    z_layers: 4,
                },
                site_id: None,
                corridor_reserve: false,
            },
        );
        let rev_before = voxel.revision;
        app.insert_resource(voxel);
        {
            let mut req = app.world_mut().resource_mut::<ConstructionVisualRequests>();
            req.footprint_tiles.push(FootprintTileRequest {
                tile: IVec2::new(3, 4),
                color_kind: FootprintTileColorKind::Valid,
                weight: 1.0,
                fill_override: None,
            });
        }

        app.world_mut()
            .run_system_once(sync_voxel_massing_visual_requests)
            .expect("sync runs");

        let req = app.world().resource::<ConstructionVisualRequests>();
        assert_eq!(req.massing_columns.len(), 1);
        assert_eq!(req.massing_columns[0].floors, 4);
        assert_eq!(
            app.world().resource::<VoxelOccupationBook>().revision,
            rev_before,
            "massing must not bump voxel revision"
        );
    }

    #[test]
    fn sync_ignores_voxel_when_flags_off() {
        let mut app = massing_test_app();
        app.insert_resource(VoxelOccupationConfig::default());
        let mut voxel = VoxelOccupationBook::default();
        voxel.cells.insert(
            IVec2::new(3, 4),
            VoxelOccupationCell {
                xy: IVec2::new(3, 4),
                layers: OccupancyLayerStack {
                    weight: 1.0,
                    z_layers: 4,
                },
                site_id: None,
                corridor_reserve: false,
            },
        );
        app.insert_resource(voxel);
        {
            let mut req = app.world_mut().resource_mut::<ConstructionVisualRequests>();
            req.footprint_tiles.push(FootprintTileRequest {
                tile: IVec2::new(3, 4),
                color_kind: FootprintTileColorKind::Valid,
                weight: 1.0,
                fill_override: None,
            });
        }

        app.world_mut()
            .run_system_once(sync_voxel_massing_visual_requests)
            .expect("sync runs");

        let req = app.world().resource::<ConstructionVisualRequests>();
        assert_eq!(req.massing_columns.len(), 1);
        assert_eq!(
            req.massing_columns[0].floors, 1,
            "flag off → catalog derive (occupied=1), ignore voxel z_layers"
        );
    }

    #[test]
    fn sync_clears_when_not_pixel_world() {
        let mut app = massing_test_app();
        app.insert_resource(WorldPresentationMode {
            kind: WorldPresentationKind::TacticalFlat2D,
        });
        app.world_mut()
            .resource_mut::<StudioMassingOverride>()
            .set_from_json(FIXTURE_JSON);
        {
            let mut req = app.world_mut().resource_mut::<ConstructionVisualRequests>();
            req.massing_columns.push(VoxelColumnRequest {
                tile: IVec2::ZERO,
                floors: 2,
                color_kind: FootprintTileColorKind::Valid,
                weight: 1.0,
            });
        }

        app.world_mut()
            .run_system_once(sync_voxel_massing_visual_requests)
            .expect("sync runs");

        let req = app.world().resource::<ConstructionVisualRequests>();
        assert!(req.massing_columns.is_empty());
        assert!(req.massing_instances.is_empty());
    }

    /// X5 / RND-09 — count gate + pack API (gea4v2 11.3.8.2 · LEGACY geometry_visual_effects).
    #[test]
    fn rnd09_instance_batch_gate_and_pack() {
        assert!(!massing_uses_instance_batch(0));
        assert!(!massing_uses_instance_batch(MASSING_INSTANCE_BATCH_THRESHOLD - 1));
        assert!(massing_uses_instance_batch(MASSING_INSTANCE_BATCH_THRESHOLD));
        assert!(massing_uses_instance_batch(MASSING_INSTANCE_BATCH_THRESHOLD + 8));

        let below: Vec<VoxelColumnRequest> = (0..MASSING_INSTANCE_BATCH_THRESHOLD - 1)
            .map(|i| VoxelColumnRequest {
                tile: IVec2::new(i as i32, 0),
                floors: 2,
                color_kind: FootprintTileColorKind::Valid,
                weight: 1.0,
            })
            .collect();
        assert!(pack_massing_instances(&below).is_empty());

        let at_gate: Vec<VoxelColumnRequest> = (0..MASSING_INSTANCE_BATCH_THRESHOLD)
            .map(|i| VoxelColumnRequest {
                tile: IVec2::new(i as i32, 1),
                floors: 3,
                color_kind: FootprintTileColorKind::Risky,
                weight: 0.8,
            })
            .collect();
        let packed = pack_massing_instances(&at_gate);
        assert_eq!(packed.len(), MASSING_INSTANCE_BATCH_THRESHOLD);
        assert_eq!(packed[0].tile, IVec2::new(0, 1));
        assert_eq!(packed[0].floors, 3);
        assert_eq!(packed[0].color_kind, FootprintTileColorKind::Risky);
    }

    #[test]
    fn sync_packs_instances_at_threshold() {
        let mut app = massing_test_app();
        let cols: Vec<VoxelColumnRequest> = (0..MASSING_INSTANCE_BATCH_THRESHOLD)
            .map(|i| VoxelColumnRequest {
                tile: IVec2::new(i as i32, 2),
                floors: 2,
                color_kind: FootprintTileColorKind::Valid,
                weight: 1.0,
            })
            .collect();
        app.world_mut()
            .resource_mut::<StudioMassingOverride>()
            .set_columns(cols);

        app.world_mut()
            .run_system_once(sync_voxel_massing_visual_requests)
            .expect("sync runs");

        let req = app.world().resource::<ConstructionVisualRequests>();
        assert_eq!(req.massing_columns.len(), MASSING_INSTANCE_BATCH_THRESHOLD);
        assert_eq!(req.massing_instances.len(), MASSING_INSTANCE_BATCH_THRESHOLD);
        assert_eq!(req.massing_instances[0].tile, req.massing_columns[0].tile);
    }

    #[test]
    fn sync_clears_instances_below_threshold() {
        let mut app = massing_test_app();
        app.world_mut()
            .resource_mut::<StudioMassingOverride>()
            .set_from_json(FIXTURE_JSON); // 3 columns ≪ 64
        {
            let mut req = app.world_mut().resource_mut::<ConstructionVisualRequests>();
            req.massing_instances.push(MassingColumnInstance {
                tile: IVec2::new(7, 7),
                floors: 2,
                color_kind: FootprintTileColorKind::Valid,
                weight: 1.0,
            });
        }

        app.world_mut()
            .run_system_once(sync_voxel_massing_visual_requests)
            .expect("sync runs");

        let req = app.world().resource::<ConstructionVisualRequests>();
        assert_eq!(req.massing_columns.len(), 3);
        assert!(
            req.massing_instances.is_empty(),
            "below gate sole writer must clear stale instance rows"
        );
    }
}
