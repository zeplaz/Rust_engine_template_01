//! **CITY-G1-C3-001** — deterministic BlockRecipe evaluator (RON → lots / edge / scatter).

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::block::BlockRecord;
use super::block_archetype::BlockArchetype;
use super::block_frame::{scatter_interior_tiles, street_edge_tiles, BlockFrame, StreetSide};
use super::seed_chain::lot_seed;

pub const BLOCK_RECIPES_DIR: &str = "assets/configs/settlement/block_recipes";
pub const CITY_G1_C3_LIVE_JSON: &str = "debug_runs/city_g1_c3_001_live.json";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LotFacing {
    Street,
    Interior,
    Alley,
}

impl LotFacing {
    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "street" => Some(Self::Street),
            "interior" => Some(Self::Interior),
            "alley" => Some(Self::Alley),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BlockRecipeMeta {
    #[serde(default)]
    pub teaches: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BlockRecipeStep {
    LotRow {
        count: u8,
        depth: u8,
        facing: String,
        #[serde(default)]
        setback: u8,
        building_archetype: String,
        #[serde(default)]
        district_style: Option<String>,
        #[serde(default = "default_lot_width")]
        lot_width: u8,
    },
    Edge {
        asset: String,
        #[serde(default = "default_spacing")]
        spacing: f32,
        #[serde(default)]
        offset: u8,
        #[serde(default)]
        sim_authority: bool,
    },
    Scatter {
        asset: String,
        density: f32,
        #[serde(default)]
        jitter: f32,
        #[serde(default = "default_scatter_zone")]
        zone: String,
    },
    ParkFill {
        coverage: f32,
        #[serde(default = "default_park_surface")]
        surface: String,
    },
    Plaza {
        extent: [u8; 2],
        anchor: String,
        #[serde(default)]
        furniture: Vec<String>,
    },
}

fn default_lot_width() -> u8 {
    1
}

fn default_spacing() -> f32 {
    1.0
}

fn default_scatter_zone() -> String {
    "interior".into()
}

fn default_park_surface() -> String {
    "grass".into()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlockRecipe {
    pub schema: String,
    pub version: String,
    pub recipe_id: String,
    pub block_archetype: String,
    pub label: String,
    pub default_district_style: String,
    #[serde(default)]
    pub _meta: BlockRecipeMeta,
    pub steps: Vec<BlockRecipeStep>,
}

impl BlockRecipe {
    #[must_use]
    pub fn archetype(&self) -> Option<BlockArchetype> {
        parse_block_archetype(&self.block_archetype)
    }
}

#[must_use]
fn parse_block_archetype(raw: &str) -> Option<BlockArchetype> {
    match raw {
        "ForestPark" => Some(BlockArchetype::ForestPark),
        "LowDensityRes" => Some(BlockArchetype::LowDensityRes),
        "MediumDensityRes" => Some(BlockArchetype::MediumDensityRes),
        "HighDensityCommercial" => Some(BlockArchetype::HighDensityCommercial),
        "Industrial" => Some(BlockArchetype::Industrial),
        "Civic" => Some(BlockArchetype::Civic),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlockLotPlacement {
    pub lot_idx: u32,
    pub origin: IVec2,
    pub width: u8,
    pub depth: u8,
    pub tiles: Vec<IVec2>,
    pub building_archetype: String,
    pub district_style: String,
    pub lot_seed: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlockEdgePlacement {
    pub asset: String,
    pub street_tiles: Vec<IVec2>,
    pub sim_authority: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlockScatterPlacement {
    pub asset: String,
    pub tiles: Vec<IVec2>,
    pub density: f32,
    pub sim_authority: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlockParkFillPlacement {
    pub surface: String,
    pub tiles: Vec<IVec2>,
    pub coverage: f32,
    pub sim_authority: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlockPlazaPlacement {
    pub anchor: String,
    pub extent: [u8; 2],
    pub tiles: Vec<IVec2>,
    #[serde(default)]
    pub furniture: Vec<String>,
    pub sim_authority: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BlockRecipeEvaluation {
    pub recipe_id: String,
    pub lots: Vec<BlockLotPlacement>,
    pub edges: Vec<BlockEdgePlacement>,
    pub scatters: Vec<BlockScatterPlacement>,
    pub park_fills: Vec<BlockParkFillPlacement>,
    pub plazas: Vec<BlockPlazaPlacement>,
    pub errors: Vec<String>,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct BlockRecipeRegistry {
    pub recipes: HashMap<String, BlockRecipe>,
    pub by_archetype: HashMap<BlockArchetype, String>,
    pub load_errors: Vec<String>,
}

impl BlockRecipeRegistry {
    #[must_use]
    pub fn get(&self, recipe_id: &str) -> Option<&BlockRecipe> {
        self.recipes.get(recipe_id)
    }

    #[must_use]
    pub fn for_archetype(&self, archetype: BlockArchetype) -> Option<&BlockRecipe> {
        self.by_archetype
            .get(&archetype)
            .and_then(|id| self.recipes.get(id.as_str()))
    }
}

#[must_use]
fn repo_asset_path(rel: &str) -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(rel))
        .unwrap_or_else(|| PathBuf::from(rel))
}

pub fn load_block_recipe_from_path(path: &Path) -> Result<BlockRecipe, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let recipe: BlockRecipe = ron::from_str(&text).map_err(|e| format!("RON parse {}: {e}", path.display()))?;
    if recipe.schema != "block_recipe_v1" {
        return Err(format!(
            "unsupported schema {} in {}",
            recipe.schema,
            path.display()
        ));
    }
    Ok(recipe)
}

pub fn load_block_recipes_from_dir(dir: &Path) -> BlockRecipeRegistry {
    let mut registry = BlockRecipeRegistry::default();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(err) => {
            registry
                .load_errors
                .push(format!("read_dir {}: {err}", dir.display()));
            return registry;
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "ron") {
            continue;
        }
        match load_block_recipe_from_path(&path) {
            Ok(recipe) => {
                if let Some(archetype) = recipe.archetype() {
                    registry
                        .by_archetype
                        .insert(archetype, recipe.recipe_id.clone());
                }
                registry.recipes.insert(recipe.recipe_id.clone(), recipe);
            }
            Err(err) => registry.load_errors.push(err),
        }
    }
    registry
}

#[must_use]
pub fn load_block_recipe_registry() -> BlockRecipeRegistry {
    load_block_recipes_from_dir(&repo_asset_path(BLOCK_RECIPES_DIR))
}

struct StreetAxis {
    parallel_min: i32,
    parallel_max: i32,
    street_line: i32,
    inward_sign: i32,
    parallel_axis_is_z: bool,
}

#[must_use]
fn street_axis(frame: &BlockFrame, facing: LotFacing) -> StreetAxis {
    let w = frame.extent.x as i32;
    let d = frame.extent.y as i32;
    let side = match facing {
        LotFacing::Street => frame.street_side,
        LotFacing::Alley => match frame.street_side {
            StreetSide::PosX => StreetSide::NegX,
            StreetSide::NegX => StreetSide::PosX,
            StreetSide::PosZ => StreetSide::NegZ,
            StreetSide::NegZ => StreetSide::PosZ,
        },
        LotFacing::Interior => frame.street_side,
    };
    match side {
        StreetSide::PosX => StreetAxis {
            parallel_min: frame.anchor.y,
            parallel_max: frame.anchor.y + d - 1,
            street_line: frame.anchor.x + w - 1,
            inward_sign: -1,
            parallel_axis_is_z: true,
        },
        StreetSide::NegX => StreetAxis {
            parallel_min: frame.anchor.y,
            parallel_max: frame.anchor.y + d - 1,
            street_line: frame.anchor.x,
            inward_sign: 1,
            parallel_axis_is_z: true,
        },
        StreetSide::PosZ => StreetAxis {
            parallel_min: frame.anchor.x,
            parallel_max: frame.anchor.x + w - 1,
            street_line: frame.anchor.y + d - 1,
            inward_sign: -1,
            parallel_axis_is_z: false,
        },
        StreetSide::NegZ => StreetAxis {
            parallel_min: frame.anchor.x,
            parallel_max: frame.anchor.x + w - 1,
            street_line: frame.anchor.y,
            inward_sign: 1,
            parallel_axis_is_z: false,
        },
    }
}

#[must_use]
fn lot_row_tiles(
    frame: &BlockFrame,
    facing: LotFacing,
    setback: u8,
    count: u8,
    depth: u8,
    lot_width: u8,
    lot_idx: u8,
    block_tiles: &HashSet<IVec2>,
) -> Option<HashSet<IVec2>> {
    if count == 0 || depth == 0 || lot_width == 0 || lot_idx >= count {
        return None;
    }
    let axis = street_axis(frame, facing);
    let parallel_span = axis.parallel_max - axis.parallel_min + 1;
    if (count as i32) * (lot_width as i32) > parallel_span {
        return None;
    }
    let front_line = if facing == LotFacing::Interior {
        axis.street_line
            + axis.inward_sign
                * (if axis.parallel_axis_is_z {
                    frame.extent.x as i32 / 2
                } else {
                    frame.extent.y as i32 / 2
                })
    } else {
        axis.street_line + axis.inward_sign * setback as i32
    };
    let parallel_start = axis.parallel_min + lot_idx as i32 * lot_width as i32;
    let mut out = HashSet::new();
    for p in 0..lot_width {
        for depth_step in 0..depth {
            let tile = if axis.parallel_axis_is_z {
                IVec2::new(
                    front_line + axis.inward_sign * depth_step as i32,
                    parallel_start + p as i32,
                )
            } else {
                IVec2::new(
                    parallel_start + p as i32,
                    front_line + axis.inward_sign * depth_step as i32,
                )
            };
            if !block_tiles.contains(&tile) {
                return None;
            }
            out.insert(tile);
        }
    }
    Some(out)
}

#[must_use]
fn scatter_zone_tiles(
    frame: &BlockFrame,
    block_tiles: &HashSet<IVec2>,
    zone: &str,
) -> Vec<IVec2> {
    let interior = scatter_interior_tiles(frame, block_tiles);
    if zone == "rear" {
        let edge = street_edge_tiles(frame);
        let w = frame.extent.x as i32;
        let rear_x = match frame.street_side {
            StreetSide::PosX => frame.anchor.x,
            StreetSide::NegX => frame.anchor.x + w - 1,
            _ => frame.anchor.x,
        };
        let mut rear: Vec<IVec2> = interior
            .iter()
            .copied()
            .filter(|t| match frame.street_side {
                StreetSide::PosX | StreetSide::NegX => t.x == rear_x,
                StreetSide::PosZ => t.y == frame.anchor.y,
                StreetSide::NegZ => t.y == frame.anchor.y + frame.extent.y as i32 - 1,
            })
            .collect();
        if rear.is_empty() {
            rear = edge.iter().copied().collect();
        }
        rear.sort_by_key(|t| (t.x, t.y));
        return rear;
    }
    let mut tiles: Vec<IVec2> = interior.into_iter().collect();
    tiles.sort_by_key(|t| (t.x, t.y));
    tiles
}

#[must_use]
fn plaza_origin(frame: &BlockFrame, anchor: &str) -> IVec2 {
    let w = frame.extent.x as i32;
    let d = frame.extent.y as i32;
    match anchor {
        "corner" => match frame.street_side {
            StreetSide::PosX => IVec2::new(frame.anchor.x + w - 1, frame.anchor.y),
            StreetSide::NegX => frame.anchor,
            StreetSide::PosZ => IVec2::new(frame.anchor.x, frame.anchor.y + d - 1),
            StreetSide::NegZ => frame.anchor,
        },
        "mid_block" => IVec2::new(frame.anchor.x + w / 2, frame.anchor.y + d / 2),
        "street_mouth" => {
            let mut edge: Vec<IVec2> = street_edge_tiles(frame).into_iter().collect();
            edge.sort_by_key(|t| (t.x, t.y));
            let mid = edge[edge.len() / 2];
            match frame.street_side {
                StreetSide::PosX => IVec2::new(mid.x - 1, mid.y),
                StreetSide::NegX => IVec2::new(mid.x + 1, mid.y),
                StreetSide::PosZ => IVec2::new(mid.x, mid.y - 1),
                StreetSide::NegZ => IVec2::new(mid.x, mid.y + 1),
            }
        }
        _ => frame.anchor,
    }
}

#[must_use]
fn plaza_extent_tiles(
    frame: &BlockFrame,
    origin: IVec2,
    extent: [u8; 2],
    block_tiles: &HashSet<IVec2>,
) -> Vec<IVec2> {
    let ew = extent[0].max(1) as i32;
    let ed = extent[1].max(1) as i32;
    let mut out = Vec::new();
    for dx in 0..ew {
        for dz in 0..ed {
            let tile = match frame.street_side {
                StreetSide::PosX | StreetSide::NegX => {
                    let inward_x = if frame.street_side == StreetSide::PosX {
                        -1
                    } else {
                        1
                    };
                    IVec2::new(origin.x + inward_x * dx, origin.y + dz)
                }
                StreetSide::PosZ | StreetSide::NegZ => {
                    let inward_z = if frame.street_side == StreetSide::PosZ {
                        -1
                    } else {
                        1
                    };
                    IVec2::new(origin.x + dx, origin.y + inward_z * dz)
                }
            };
            if block_tiles.contains(&tile) {
                out.push(tile);
            }
        }
    }
    out.sort_by_key(|t| (t.x, t.y));
    out
}

#[must_use]
pub fn evaluate_block_recipe(
    recipe: &BlockRecipe,
    frame: &BlockFrame,
    block: &BlockRecord,
    block_seed: u64,
) -> BlockRecipeEvaluation {
    let mut eval = BlockRecipeEvaluation {
        recipe_id: recipe.recipe_id.clone(),
        ..Default::default()
    };
    let mut occupied = HashSet::new();
    let mut lot_idx = 0u32;

    for step in &recipe.steps {
        match step {
            BlockRecipeStep::LotRow {
                count,
                depth,
                facing,
                setback,
                building_archetype,
                district_style,
                lot_width,
            } => {
                let Some(facing) = LotFacing::parse(facing) else {
                    eval.errors
                        .push(format!("lot_row unknown facing: {facing}"));
                    continue;
                };
                for i in 0..*count {
                    let Some(tiles) = lot_row_tiles(
                        frame,
                        facing,
                        *setback,
                        *count,
                        *depth,
                        *lot_width,
                        i,
                        &block.tiles,
                    ) else {
                        eval.errors.push(format!(
                            "lot_row {i} does not fit block extent for {}",
                            recipe.recipe_id
                        ));
                        continue;
                    };
                    if tiles.iter().any(|t| occupied.contains(t)) {
                        eval.errors.push(format!(
                            "lot_row {i} overlaps prior footprint in {}",
                            recipe.recipe_id
                        ));
                        continue;
                    }
                    occupied.extend(tiles.iter().copied());
                    let style = district_style
                        .clone()
                        .unwrap_or_else(|| recipe.default_district_style.clone());
                    let ls = lot_seed(block_seed, lot_idx);
                    let origin = tiles.iter().copied().min_by_key(|t| (t.x, t.y)).unwrap_or(frame.anchor);
                    eval.lots.push(BlockLotPlacement {
                        lot_idx,
                        origin,
                        width: *lot_width,
                        depth: *depth,
                        tiles: {
                            let mut v: Vec<_> = tiles.into_iter().collect();
                            v.sort_by_key(|t| (t.x, t.y));
                            v
                        },
                        building_archetype: building_archetype.clone(),
                        district_style: style,
                        lot_seed: ls,
                    });
                    lot_idx += 1;
                }
            }
            BlockRecipeStep::Edge {
                asset,
                spacing: _,
                offset: _,
                sim_authority,
            } => {
                let mut street = street_edge_tiles(frame).into_iter().collect::<Vec<_>>();
                street.sort_by_key(|t| (t.x, t.y));
                eval.edges.push(BlockEdgePlacement {
                    asset: asset.clone(),
                    street_tiles: street,
                    sim_authority: *sim_authority,
                });
            }
            BlockRecipeStep::Scatter {
                asset,
                density,
                jitter,
                zone,
            } => {
                let zone_tiles = scatter_zone_tiles(frame, &block.tiles, zone);
                let want = ((*density).clamp(0.0, 1.0) * zone_tiles.len() as f32).round() as usize;
                let jitter_u = (*jitter).clamp(0.0, 1.0) as f64;
                let mut picks = Vec::new();
                for (n, tile) in zone_tiles.iter().enumerate() {
                    if occupied.contains(tile) {
                        continue;
                    }
                    let gate = super::seed_chain::mix_u64(
                        block_seed,
                        "scatter",
                        &format!("{}:{n}:{asset}", recipe.recipe_id),
                    );
                    let threshold = ((*density).clamp(0.0, 1.0) as f64
                        * u64::MAX as f64
                        * (1.0 - jitter_u * 0.5)) as u64;
                    if gate <= threshold || picks.len() < want {
                        picks.push(*tile);
                    }
                    if picks.len() >= want {
                        break;
                    }
                }
                eval.scatters.push(BlockScatterPlacement {
                    asset: asset.clone(),
                    tiles: picks,
                    density: *density,
                    sim_authority: false,
                });
            }
            BlockRecipeStep::ParkFill { coverage, surface } => {
                let interior = scatter_interior_tiles(frame, &block.tiles);
                let mut tiles: Vec<IVec2> = interior.into_iter().collect();
                tiles.sort_by_key(|t| (t.x, t.y));
                let take = ((*coverage).clamp(0.0, 1.0) * tiles.len() as f32).ceil() as usize;
                eval.park_fills.push(BlockParkFillPlacement {
                    surface: surface.clone(),
                    tiles: tiles.into_iter().take(take).collect(),
                    coverage: *coverage,
                    sim_authority: false,
                });
            }
            BlockRecipeStep::Plaza {
                extent,
                anchor,
                furniture,
            } => {
                let origin = plaza_origin(frame, anchor);
                let mut tiles = plaza_extent_tiles(frame, origin, *extent, &block.tiles);
                tiles.retain(|t| !occupied.contains(t));
                if tiles.is_empty() {
                    eval.errors.push(format!(
                        "plaza {anchor} {:?} does not fit block {}",
                        extent, recipe.recipe_id
                    ));
                    continue;
                }
                occupied.extend(tiles.iter().copied());
                eval.plazas.push(BlockPlazaPlacement {
                    anchor: anchor.clone(),
                    extent: *extent,
                    tiles,
                    furniture: furniture.clone(),
                    sim_authority: false,
                });
            }
        }
    }

    eval
}

#[must_use]
pub fn block_recipe_lot_list_stable_hash(eval: &BlockRecipeEvaluation) -> String {
    let mut rows: Vec<String> = eval
        .lots
        .iter()
        .map(|lot| {
            format!(
                "{}:{}/{}:{}:{:#018x}",
                lot.lot_idx,
                lot.origin.x,
                lot.origin.y,
                lot.building_archetype,
                lot.lot_seed
            )
        })
        .collect();
    rows.sort();
    let digest = Sha256::digest(rows.join("|").as_bytes());
    format!("{:016x}", u64::from_le_bytes(digest[..8].try_into().unwrap()))
}

#[must_use]
pub fn evaluate_block_recipe_for_archetype(
    registry: &BlockRecipeRegistry,
    archetype: BlockArchetype,
    frame: &BlockFrame,
    block: &BlockRecord,
    block_seed: u64,
) -> Option<BlockRecipeEvaluation> {
    let recipe = registry.for_archetype(archetype)?;
    Some(evaluate_block_recipe(recipe, frame, block, block_seed))
}

#[must_use]
pub fn build_city_g1_c3_001_witness_body() -> serde_json::Value {
    use super::block_frame::{
        build_block_frame, fixture_block_record_with_tiles, fixture_transport_graph_for_block_frame,
    };
    use super::seed_chain::{block_seed, town_seed, DEFAULT_TOWN_ID, DEFAULT_WORLD_SEED};

    let registry = load_block_recipe_registry();
    let recipes_ok = registry.load_errors.is_empty()
        && registry.recipes.len() >= 3
        && registry.for_archetype(BlockArchetype::Industrial).is_some()
        && registry
            .for_archetype(BlockArchetype::LowDensityRes)
            .is_some()
        && registry
            .for_archetype(BlockArchetype::MediumDensityRes)
            .is_some();

    let graph = fixture_transport_graph_for_block_frame();
    let block = fixture_block_record_with_tiles();
    let frame = build_block_frame(&block, Some(&graph)).expect("fixture frame");
    let bs = block_seed(
        town_seed(DEFAULT_WORLD_SEED, &super::ids::TownId(DEFAULT_TOWN_ID.into())),
        &block.id,
    );

    let bands = [
        (BlockArchetype::Industrial, 2u32),
        (BlockArchetype::LowDensityRes, 2u32),
        (BlockArchetype::MediumDensityRes, 4u32),
    ];

    let mut eval_rows = Vec::new();
    let mut all_errors_empty = true;
    let mut lot_counts_ok = true;
    let mut lot_seeds_wired = true;
    let mut mut_hashes = Vec::new();

    for (archetype, expected_lots) in bands {
        let Some(eval) = evaluate_block_recipe_for_archetype(&registry, archetype, &frame, &block, bs)
        else {
            lot_counts_ok = false;
            continue;
        };
        if !eval.errors.is_empty() {
            all_errors_empty = false;
        }
        if eval.lots.len() as u32 != expected_lots {
            lot_counts_ok = false;
        }
        if eval
            .lots
            .iter()
            .any(|lot| lot.lot_seed != lot_seed(bs, lot.lot_idx))
        {
            lot_seeds_wired = false;
        }
        let hash = block_recipe_lot_list_stable_hash(&eval);
        mut_hashes.push(hash.clone());
        eval_rows.push(serde_json::json!({
            "block_archetype": format!("{archetype:?}"),
            "recipe_id": eval.recipe_id,
            "lot_count": eval.lots.len(),
            "edge_count": eval.edges.len(),
            "scatter_count": eval.scatters.iter().map(|s| s.tiles.len()).sum::<usize>(),
            "park_fill_tiles": eval.park_fills.iter().map(|p| p.tiles.len()).sum::<usize>(),
            "lot_list_hash": hash,
            "errors": eval.errors,
        }));
    }

    let mut run_hashes = Vec::new();
    for _ in 0..3 {
        if let Some(eval) =
            evaluate_block_recipe_for_archetype(&registry, BlockArchetype::Industrial, &frame, &block, bs)
        {
            run_hashes.push(block_recipe_lot_list_stable_hash(&eval));
        }
    }
    let three_run_stable =
        run_hashes.len() == 3 && run_hashes.windows(2).all(|w| w[0] == w[1]);

    let g0 = crate::construction::procedural::city_g0_wit_001_determinism_witness_green();
    let g1_c1 = super::block_archetype::city_g1_c1_001_block_archetype_witness_green();
    let g1_c2 = super::block_frame::city_g1_c2_001_block_frame_witness_green();
    let g1_c4 = super::seed_chain::city_g1_c4_001_seed_chain_witness_green();
    let charter = {
        let path = repo_asset_path("debug_runs/city_block_recipe_charter_live.json");
        fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
            .and_then(|v| {
                v.get("green")
                    .or_else(|| v.get("payload").and_then(|p| p.get("green")))
                    .and_then(|g| g.as_bool())
            })
            .unwrap_or(false)
    };

    let green = recipes_ok
        && all_errors_empty
        && lot_counts_ok
        && lot_seeds_wired
        && three_run_stable
        && g1_c2
        && charter;

    serde_json::json!({
        "gate": "CITY-G1-C3-001",
        "issue": "CITY-C3",
        "green": green,
        "recipes_loaded": registry.recipes.len(),
        "recipes_ok": recipes_ok,
        "evaluations": eval_rows,
        "three_run_stable": three_run_stable,
        "stable_lot_hash": run_hashes.first(),
        "run_hashes": run_hashes,
        "city_g0_wit_still_green": g0,
        "city_g1_c1_still_green": g1_c1,
        "city_g1_c2_still_green": g1_c2,
        "city_g1_c4_still_green": g1_c4,
        "block_recipe_charter_green": charter,
    })
}

#[must_use]
pub fn city_g1_c3_001_block_recipe_witness_green() -> bool {
    build_city_g1_c3_001_witness_body()
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

#[must_use]
pub fn refresh_city_g1_c3_001_block_recipe_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_city_g1_c3_001_witness_body();
    let green = body
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let wrapped = wrap_debug_run(
        "CITY-G1-C3-001",
        "refresh_city_g1_c3_001_block_recipe_witness",
        CITY_G1_C3_LIVE_JSON,
        body,
    );
    write_debug_run_json(CITY_G1_C3_LIVE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_recipes_load_from_disk() {
        let reg = load_block_recipe_registry();
        assert!(reg.load_errors.is_empty(), "{:?}", reg.load_errors);
        assert!(reg.recipes.len() >= 3);
    }

    #[test]
    fn industrial_recipe_emits_two_lots_on_fixture() {
        use super::super::block_frame::{
            build_block_frame, fixture_block_record_with_tiles, fixture_transport_graph_for_block_frame,
        };
        use super::super::seed_chain::{block_seed, town_seed, DEFAULT_TOWN_ID, DEFAULT_WORLD_SEED};

        let reg = load_block_recipe_registry();
        let graph = fixture_transport_graph_for_block_frame();
        let block = fixture_block_record_with_tiles();
        let frame = build_block_frame(&block, Some(&graph)).expect("frame");
        let bs = block_seed(
            town_seed(DEFAULT_WORLD_SEED, &super::super::ids::TownId(DEFAULT_TOWN_ID.into())),
            &block.id,
        );
        let eval = evaluate_block_recipe_for_archetype(
            &reg,
            BlockArchetype::Industrial,
            &frame,
            &block,
            bs,
        )
        .expect("eval");
        assert!(eval.errors.is_empty(), "{:?}", eval.errors);
        assert_eq!(eval.lots.len(), 2);
        assert_eq!(eval.lots[0].building_archetype, "IndustrialWarehouse");
        assert_eq!(eval.lots[0].lot_seed, lot_seed(bs, 0));
    }

    #[test]
    fn lot_list_hash_stable_across_runs() {
        use super::super::block_frame::{
            build_block_frame, fixture_block_record_with_tiles, fixture_transport_graph_for_block_frame,
        };
        use super::super::seed_chain::{block_seed, town_seed, DEFAULT_TOWN_ID, DEFAULT_WORLD_SEED};

        let reg = load_block_recipe_registry();
        let graph = fixture_transport_graph_for_block_frame();
        let block = fixture_block_record_with_tiles();
        let frame = build_block_frame(&block, Some(&graph)).expect("frame");
        let bs = block_seed(
            town_seed(DEFAULT_WORLD_SEED, &super::super::ids::TownId(DEFAULT_TOWN_ID.into())),
            &block.id,
        );
        let a = evaluate_block_recipe_for_archetype(
            &reg,
            BlockArchetype::MediumDensityRes,
            &frame,
            &block,
            bs,
        )
        .expect("eval");
        let b = evaluate_block_recipe_for_archetype(
            &reg,
            BlockArchetype::MediumDensityRes,
            &frame,
            &block,
            bs,
        )
        .expect("eval");
        assert_eq!(
            block_recipe_lot_list_stable_hash(&a),
            block_recipe_lot_list_stable_hash(&b)
        );
        assert_eq!(a.lots.len(), 4);
    }

    #[test]
    fn plaza_step_places_tiles_without_error() {
        use super::super::block_frame::{
            build_block_frame, fixture_block_record_with_tiles, fixture_transport_graph_for_block_frame,
        };
        use super::super::seed_chain::{block_seed, town_seed, DEFAULT_TOWN_ID, DEFAULT_WORLD_SEED};

        let graph = fixture_transport_graph_for_block_frame();
        let block = fixture_block_record_with_tiles();
        let frame = build_block_frame(&block, Some(&graph)).expect("frame");
        let bs = block_seed(
            town_seed(DEFAULT_WORLD_SEED, &super::super::ids::TownId(DEFAULT_TOWN_ID.into())),
            &block.id,
        );
        let recipe = BlockRecipe {
            schema: "block_recipe_v1".into(),
            version: "1.0.0".into(),
            recipe_id: "test_plaza".into(),
            block_archetype: "Civic".into(),
            label: "test".into(),
            default_district_style: "civic".into(),
            _meta: BlockRecipeMeta::default(),
            steps: vec![BlockRecipeStep::Plaza {
                extent: [2, 2],
                anchor: "corner".into(),
                furniture: vec!["bench".into()],
            }],
        };
        let eval = evaluate_block_recipe(&recipe, &frame, &block, bs);
        assert!(eval.errors.is_empty(), "{:?}", eval.errors);
        assert_eq!(eval.plazas.len(), 1);
        assert!(!eval.plazas[0].tiles.is_empty());
    }

    #[test]
    fn city_g1_c3_001_witness_green_lib() {
        assert!(city_g1_c3_001_block_recipe_witness_green());
    }
}
