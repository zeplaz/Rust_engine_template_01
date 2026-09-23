//! Save / load / bake transport + hybrid world + map snapshot IO (M5, G4, R8).

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use bevy::prelude::*;

use crate::gui::editor::editor_world_commit_bridge::{
    write_editor_world_grid_commit, EditorTileEditCommitted, EditorTileEditKind,
};
use crate::io::snapshot::{read_hybrid_world_snapshot_dev_v0, write_hybrid_world_snapshot_dev_v0};
use crate::strategic::{
    apply_corridor_book_from_transport_snapshot, transport_construction_records_from_book,
    CorridorConstructionBook,
};
use crate::systems::terrain::TerrainRegistriesHandles;
use crate::systems::transport::{
    bake_snapshot_from_ordered_markers_with_world_positions, hydrate_transport_from_snapshot,
    hydrate_transport_from_snapshot_text, transport_network_snapshot_from_world_with_construction,
    transport_network_snapshot_save_ron_path, transport_network_snapshot_to_ron_string,
    LoadTransportNetworkSnapshotFromDisk, TransportEdgeDirectory, TransportFieldStore,
    TransportLastHydratedSnapshot, TransportTopology,
};
use crate::terrain::editor::map_snapshot::{
    load_map_snapshot_bundle_from_ron, MapSnapshotCellV2, MapSnapshotV2,
    MAP_SNAPSHOT_SCHEMA_VERSION_V2,
};
use crate::terrain::family::{TerrainFamilyId, TerrainFamilyRegistry, DEFAULT_TERRAIN_FAMILY_ID};
use crate::terrain::generation::polygon_world_semantics::MacroStrategicKind;
use crate::terrain::generation::world_generator_enhanced::{
    despawn_generated_world_entities, Height, Moisture, Temperature, TerrainType, TileMarker,
    TileRegionIndex, WorldGenParams, WorldMarker,
};

use super::road_markers::{
    MapEditorRoadMarkerV1, MapEditorRoadPlacementSeq, MapEditorRoadUndoStack,
    RoadAuthoringGhostPreview,
};
use super::HEIGHT_WORLD_SCALE;

/// Request: build **W1** transport topology from current [`MapEditorRoadMarkerV1`] entities.
#[derive(Message)]
pub struct MapEditorBakeTransportRequest;

/// **G4** dev: write `TransportNetworkSnapshot` JSON under `assets/saves/` (crate root at compile time).
#[derive(Message)]
pub struct MapEditorSaveDevTransportRequest;

/// **G4** dev: load same path via [`LoadTransportNetworkSnapshotFromDisk`].
#[derive(Message)]
pub struct MapEditorLoadDevTransportRequest;

/// **M5 / wave S** stub: write hybrid-shaped dev snapshot (JSON header line + transport JSON body).
#[derive(Message)]
pub struct MapEditorSaveHybridWorldDevRequest;

/// Load transport body from [`dev_hybrid_world_save_path`] after validating header.
#[derive(Message)]
pub struct MapEditorLoadHybridWorldDevRequest;

/// **M5:** save or load terrain grid snapshot at `assets/saves/maps/last.ron`.
#[derive(Message, Clone, Copy)]
pub enum MapEditorMapSnapshotIoRequest {
    Save,
    Load,
}

pub(crate) fn dev_transport_network_save_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/saves/dev_transport_network.ron")
}

pub(crate) fn dev_hybrid_world_save_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/saves/dev_world_hybrid_v0.sav")
}

pub(crate) fn dev_map_snapshot_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/saves/maps/last.ron")
}

pub(crate) fn map_editor_bake_transport(
    mut events: MessageReader<MapEditorBakeTransportRequest>,
    markers: Query<(&MapEditorRoadMarkerV1, &Transform)>,
    mut topology: ResMut<TransportTopology>,
    mut fields: ResMut<TransportFieldStore>,
    mut directory: ResMut<TransportEdgeDirectory>,
    mut last_hydrated: ResMut<TransportLastHydratedSnapshot>,
    params: Res<WorldGenParams>,
    mut edit_commits: MessageWriter<EditorTileEditCommitted>,
) {
    for _ in events.read() {
        let mut rows: Vec<(u32, u32, u32, Vec3)> = markers
            .iter()
            .map(|(m, t)| (m.placement_seq, m.tile_x, m.tile_z, t.translation))
            .collect();
        rows.sort_by_key(|(seq, _, _, _)| *seq);
        let with_pos: Vec<(u32, u32, Vec3)> =
            rows.into_iter().map(|(_, x, z, p)| (x, z, p)).collect();
        let snap = bake_snapshot_from_ordered_markers_with_world_positions(&with_pos);
        if snap.edges.is_empty() {
            warn!("Bake transport: need ≥2 markers after removing consecutive duplicates on same tile.");
            continue;
        }
        match hydrate_transport_from_snapshot(&mut topology, &mut fields, &mut directory, &snap) {
            Ok(()) => {
                last_hydrated.snapshot = Some(snap);
                write_editor_world_grid_commit(
                    &mut edit_commits,
                    &params,
                    EditorTileEditKind::TransportTopology,
                );
            }
            Err(e) => warn!("Bake transport hydrate failed: {e:?}"),
        }
    }
}

pub(crate) fn map_editor_dev_save_transport(
    mut events: MessageReader<MapEditorSaveDevTransportRequest>,
    last: Res<TransportLastHydratedSnapshot>,
    topology: Res<TransportTopology>,
    directory: Res<TransportEdgeDirectory>,
    book: Res<CorridorConstructionBook>,
) {
    for _ in events.read() {
        let construction = transport_construction_records_from_book(&book, &topology);
        let snap = transport_network_snapshot_from_world_with_construction(
            &topology,
            &directory,
            construction,
        )
        .or_else(|| last.snapshot.clone());
        let Some(snap) = snap else {
            warn!("Save transport: bake or load a graph first (nothing to save).");
            continue;
        };
        let path = dev_transport_network_save_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match transport_network_snapshot_save_ron_path(&snap, &path) {
            Ok(()) => info!("Saved transport R8 RON to {}", path.display()),
            Err(e) => warn!("Save transport failed: {e:?}"),
        }
    }
}

pub(crate) fn map_editor_dev_load_transport(
    mut events: MessageReader<MapEditorLoadDevTransportRequest>,
    mut load_tx: MessageWriter<LoadTransportNetworkSnapshotFromDisk>,
) {
    for _ in events.read() {
        let path = dev_transport_network_save_path();
        let s: String = path.to_string_lossy().into_owned();
        load_tx.write(LoadTransportNetworkSnapshotFromDisk {
            path: Arc::from(s.into_boxed_str()),
        });
    }
}

pub(crate) fn map_editor_dev_save_hybrid_world(
    mut events: MessageReader<MapEditorSaveHybridWorldDevRequest>,
    last: Res<TransportLastHydratedSnapshot>,
    topology: Res<TransportTopology>,
    directory: Res<TransportEdgeDirectory>,
    book: Res<CorridorConstructionBook>,
) {
    for _ in events.read() {
        let construction = transport_construction_records_from_book(&book, &topology);
        let snap = transport_network_snapshot_from_world_with_construction(
            &topology,
            &directory,
            construction,
        )
        .or_else(|| last.snapshot.clone());
        let Some(snap) = snap else {
            warn!("Save hybrid world: bake or load a graph first (nothing to save).");
            continue;
        };
        let path = dev_hybrid_world_save_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let ron = match transport_network_snapshot_to_ron_string(&snap) {
            Ok(s) => s,
            Err(e) => {
                warn!("Save hybrid: RON error {e:?}");
                continue;
            }
        };
        match write_hybrid_world_snapshot_dev_v0(&path, ron.as_bytes()) {
            Ok(()) => info!("Saved hybrid dev snapshot to {}", path.display()),
            Err(e) => warn!("Save hybrid failed: {e:?}"),
        }
    }
}

pub(crate) fn map_editor_dev_load_hybrid_world(
    mut events: MessageReader<MapEditorLoadHybridWorldDevRequest>,
    mut topology: ResMut<TransportTopology>,
    mut fields: ResMut<TransportFieldStore>,
    mut directory: ResMut<TransportEdgeDirectory>,
    mut last: ResMut<TransportLastHydratedSnapshot>,
    mut book: ResMut<CorridorConstructionBook>,
) {
    for _ in events.read() {
        let path = dev_hybrid_world_save_path();
        let (header, body) = match read_hybrid_world_snapshot_dev_v0(&path) {
            Ok(x) => x,
            Err(e) => {
                warn!("Load hybrid failed for {}: {e:?}", path.display());
                continue;
            }
        };
        let text = match std::str::from_utf8(&body) {
            Ok(s) => s,
            Err(e) => {
                warn!("Load hybrid: body not UTF-8: {e:?}");
                continue;
            }
        };
        match hydrate_transport_from_snapshot_text(
            topology.as_mut(),
            fields.as_mut(),
            directory.as_mut(),
            text,
        ) {
            Ok(snap) => {
                apply_corridor_book_from_transport_snapshot(
                    book.as_mut(),
                    directory.as_ref(),
                    &snap,
                );
                last.snapshot = Some(snap);
                info!(
                    "Loaded hybrid dev transport ({} bytes, header v{})",
                    header.transport_byte_len, header.format_version
                );
            }
            Err(e) => warn!("Load hybrid hydrate failed: {e:?}"),
        }
    }
}

pub(crate) fn map_editor_map_snapshot_io(
    mut events: MessageReader<MapEditorMapSnapshotIoRequest>,
    mut commands: Commands,
    mut params: ResMut<WorldGenParams>,
    tiles: Query<(&Transform, &Height, &TerrainType), With<TileMarker>>,
    roads: Query<&MapEditorRoadMarkerV1>,
    fam_assets: Res<Assets<TerrainFamilyRegistry>>,
    handles: Res<TerrainRegistriesHandles>,
    mut road_placement: ResMut<MapEditorRoadPlacementSeq>,
    mut road_undo: ResMut<MapEditorRoadUndoStack>,
    mut ghost: ResMut<RoadAuthoringGhostPreview>,
    world_q: Query<Entity, With<WorldMarker>>,
    road_entities: Query<(Entity, &MapEditorRoadMarkerV1)>,
    mut edit_commits: MessageWriter<EditorTileEditCommitted>,
) {
    for req in events.read() {
        match *req {
            MapEditorMapSnapshotIoRequest::Save => {
                let Some(reg) = fam_assets.get(&handles.terrain_families) else {
                    warn!("Save map snapshot: terrain family registry not loaded.");
                    continue;
                };
                let w = params.width;
                let h = params.height;
                if w == 0 || h == 0 {
                    warn!("Save map snapshot: world dimensions are zero.");
                    continue;
                }
                let mut grid: Vec<Option<(f32, TerrainFamilyId)>> = vec![None; (w * h) as usize];
                for (tf, he, terr) in &tiles {
                    let x = tf.translation.x.round() as i32;
                    let z = tf.translation.z.round() as i32;
                    if x < 0 || z < 0 {
                        continue;
                    }
                    let x = x as u32;
                    let z = z as u32;
                    if x >= w || z >= h {
                        continue;
                    }
                    let i = (z * w + x) as usize;
                    grid[i] = Some((he.0, terr.0));
                }
                let mut road_tiles = HashSet::new();
                for m in &roads {
                    road_tiles.insert((m.tile_x, m.tile_z));
                }
                let mut cells = Vec::with_capacity((w * h) as usize);
                for z in 0..h {
                    for x in 0..w {
                        let i = (z * w + x) as usize;
                        let (height, tid) = grid[i].unwrap_or((0.0, DEFAULT_TERRAIN_FAMILY_ID));
                        let terrain_family = reg
                            .def(tid)
                            .map(|d| d.name.clone())
                            .unwrap_or_else(|| "Grassland".to_string());
                        cells.push(MapSnapshotCellV2 {
                            height,
                            terrain_family,
                        });
                    }
                }
                let road_marker_tiles: Vec<[u32; 2]> = road_tiles
                    .iter()
                    .map(|(x, z)| [*x, *z])
                    .collect();
                let snap_v2 = MapSnapshotV2 {
                    schema_version: MAP_SNAPSHOT_SCHEMA_VERSION_V2,
                    width: w,
                    height: h,
                    cells,
                    road_marker_tiles,
                };
                let path = dev_map_snapshot_path();
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                match snap_v2.to_ron_string() {
                    Ok(s) => match std::fs::write(&path, format!("{}\n", s.trim_end())) {
                        Ok(()) => info!("Saved map snapshot to {}", path.display()),
                        Err(e) => warn!("Save map snapshot failed: {e:?}"),
                    },
                    Err(e) => warn!("Save map snapshot RON: {e:?}"),
                }
            }
            MapEditorMapSnapshotIoRequest::Load => {
                let path = dev_map_snapshot_path();
                let bytes = match std::fs::read(&path) {
                    Ok(b) => b,
                    Err(e) => {
                        warn!("Load map snapshot: read {}: {e:?}", path.display());
                        continue;
                    }
                };
                let text = match std::str::from_utf8(&bytes) {
                    Ok(s) => s,
                    Err(e) => {
                        warn!("Load map snapshot: UTF-8: {e:?}");
                        continue;
                    }
                };
                let bundle = match load_map_snapshot_bundle_from_ron(text) {
                    Ok(s) => s,
                    Err(e) => {
                        warn!("Load map snapshot: RON: {e}");
                        continue;
                    }
                };
                let snap = bundle.terrain;
                if let Err(e) = snap.validate() {
                    warn!("Load map snapshot: {e}");
                    continue;
                }
                let Some(reg) = fam_assets.get(&handles.terrain_families) else {
                    warn!("Load map snapshot: terrain family registry not loaded.");
                    continue;
                };
                for (e, _) in road_entities.iter() {
                    commands.entity(e).despawn();
                }
                despawn_generated_world_entities(&mut commands, &world_q);
                params.width = snap.width;
                params.height = snap.height;
                *road_placement = MapEditorRoadPlacementSeq::default();
                road_undo.frames.clear();
                *ghost = RoadAuthoringGhostPreview::default();

                let world_root = commands
                    .spawn((
                        WorldMarker,
                        Transform::default(),
                        GlobalTransform::default(),
                        Name::new("Map snapshot world"),
                    ))
                    .id();

                let w = snap.width;
                let h = snap.height;
                let mut idx = 0usize;
                for z in 0..h {
                    for x in 0..w {
                        let cell = &snap.cells[idx];
                        idx += 1;
                        let tid = match reg.require_id(&cell.terrain_family) {
                            Ok(id) => id,
                            Err(_) => {
                                warn!(
                                    "Load map snapshot: unknown terrain family {:?}, using Grassland",
                                    cell.terrain_family
                                );
                                DEFAULT_TERRAIN_FAMILY_ID
                            }
                        };
                        let tile_e = commands
                            .spawn((
                                TileMarker,
                                TileRegionIndex(0),
                                Transform::from_translation(Vec3::new(
                                    x as f32,
                                    cell.height * HEIGHT_WORLD_SCALE,
                                    z as f32,
                                )),
                                Height(cell.height),
                                Moisture(0.5),
                                Temperature(0.5),
                                TerrainType(tid),
                                MacroStrategicKind::default(),
                                Name::new(format!("Tile ({x}, {z})")),
                            ))
                            .id();
                        commands.entity(world_root).add_child(tile_e);
                    }
                }

                for (x, z) in bundle.road_marker_tiles {
                    let i = (z * h + x) as usize;
                    let height = snap.cells.get(i).map(|c| c.height).unwrap_or(0.0);
                    let seq = road_placement.next;
                    road_placement.next = road_placement.next.saturating_add(1);
                    let y = height * HEIGHT_WORLD_SCALE + 0.25;
                    commands.entity(world_root).with_children(|parent| {
                        parent.spawn((
                            MapEditorRoadMarkerV1 {
                                tile_x: x,
                                tile_z: z,
                                placement_seq: seq,
                            },
                            Transform::from_translation(Vec3::new(x as f32, y, z as f32)),
                            Name::new(format!("Road marker v1 ({x},{z}) seq={seq}")),
                        ));
                    });
                }

                info!("Loaded map snapshot {}×{} from {}", w, h, path.display());
                write_editor_world_grid_commit(
                    &mut edit_commits,
                    &params,
                    EditorTileEditKind::MapSnapshotImport,
                );
            }
        }
    }
}
