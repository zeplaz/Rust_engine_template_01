//! FC-8/9 — fronts, theaters, pressure points and formations on the TacticalFlat2D world map.
//!
//! Reads the front lane's books and nothing else about them ([`FrontlineState`], [`TheaterBook`],
//! [`PressurePointBook`], [`Formation`]); it writes no sim state except through
//! [`DeploymentOrderQueue::enqueue`] (the one order funnel the AI also uses).
//!
//! * **Fronts** — one ribbon per boundary edge of every hostile pair's polylines, coloured by the
//!   control margin there (contested = hot and thick, firm = the left faction's colour, thin).
//! * **Theaters** — outline of the area of responsibility (front cells bbox plus the depth band) and a
//!   `T<id>` label at the centroid.
//! * **Pressure points** — triangles for Threatened, diamonds for Opportunity; rank 1 is largest and
//!   brightest; the label carries the rank.
//! * **Formations** — APP-6-style symbols: frame by affiliation to the viewer (friend rectangle,
//!   hostile diamond) filled with the faction colour, icon by unit kind, strength + morale bars, posture
//!   arrow (attack toward the target, retreat back, move chevron), dig-in underline.
//!
//! All geometry goes into one merged vertex-coloured mesh on one entity (like the vegetation layer);
//! labels are `Text2d`. The mesh and labels rebuild only when a content hash (books, formations,
//! selection, zoom band) changes. Symbol size is a screen-pixel size converted to tiles, so symbols stay
//! readable at every zoom. The layer is never mesh-pickable (`Pickable::IGNORE`); formation symbols are
//! selected by the cursor-tile test in [`select_and_order_formations_system`], which runs
//! `.after(PointerGateSet::Publish)` and honours the pointer gate.
//!
//! **Presentation modes:** TacticalFlat2D only. PixelWorld3D has no flat map plane for these tile-space
//! primitives (its projection is the LG-5 atlas path), so the layer is hidden there and selection /
//! ordering is disabled; a billboard/decal path for the same primitives is the graphics lane's ask.
//!
//! Controls (v1): left click selects one of the player's formations (click empty ground to deselect);
//! right click orders the selection: an enemy formation or enemy-held front cell is an Attack, a
//! Threatened pressure point of ours is a Reinforce, anything else a Move.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::combat::{
    DeploymentKind, DeploymentOrder, DeploymentOrderQueue, Formation, FrontBalance, Posture, PressureKind,
    PressurePointBook, TheaterBook, UnitKind,
};
use crate::construction::{cursor_world_on_map, ActiveBuildTool, BuildTool};
use crate::engine::states::BaseState;
use crate::gui::hud::simulation_pointer_gate::{
    simulation_tool_pointer, PointerGateSet, SimulationMapPointerGate, SimulationToolPointer,
};
use crate::gui::{MainWorldCameraOrthoTrace, MapCameraDesiredRes, SimulationMapViewport, WorldPresentationMode};
use crate::render::view_runtime::ViewProjectionAuthority;
use crate::strategic::FrontlineState;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

/// Above the network ribbons (1.04); below nothing the player needs to click.
pub const FRONT_Z: f32 = 1.10;
const THEATER_Z: f32 = 1.08;
const PRESSURE_Z: f32 = 1.12;
const SYMBOL_Z: f32 = 1.15;
const LABEL_Z: f32 = 1.2;

/// The slot whose formations the local player commands.
pub const PLAYER_SLOT: u8 = 0;

/// Symbol edge in screen pixels (converted to tiles per zoom).
const SYMBOL_PX: f32 = 20.0;
const SYMBOL_MIN_TILES: f32 = 0.9;
const SYMBOL_MAX_TILES: f32 = 12.0;

#[derive(Component)]
pub(crate) struct WorldMapFrontLayer;

#[derive(Component)]
pub(crate) struct WorldMapFrontLabel;

/// What the layer draws. All on by default.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrontOverlaySettings {
    pub fronts: bool,
    pub theaters: bool,
    pub pressure: bool,
    pub formations: bool,
    pub labels: bool,
}

impl Default for FrontOverlaySettings {
    fn default() -> Self {
        Self { fronts: true, theaters: true, pressure: true, formations: true, labels: true }
    }
}

/// The player's selected formation.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SelectedFormation(pub Option<Entity>);

/// Cursor on the map plane, published each frame after the pointer gate.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct FrontMapCursor {
    /// Map-plane position in tile units (`x`, row).
    pub xy: Option<Vec2>,
    /// The gate lets the map see the pointer (not over chrome / panels).
    pub free: bool,
    /// Screen pixels per tile (symbol hit radius is derived from it).
    pub px_per_tile: f32,
}

// ---------------------------------------------------------------------------------------------
// Geometry
// ---------------------------------------------------------------------------------------------

#[derive(Default)]
pub(crate) struct Geo {
    pub positions: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}

type Rgba = [f32; 4];

impl Geo {
    fn vertex(&mut self, p: Vec2, z: f32, c: Rgba) -> u32 {
        let i = self.positions.len() as u32;
        self.positions.push([p.x, p.y, z]);
        self.colors.push(c);
        i
    }

    fn tri(&mut self, a: Vec2, b: Vec2, c: Vec2, z: f32, col: Rgba) {
        let (i, j, k) = (self.vertex(a, z, col), self.vertex(b, z, col), self.vertex(c, z, col));
        self.indices.extend([i, j, k]);
    }

    fn quad(&mut self, a: Vec2, b: Vec2, c: Vec2, d: Vec2, z: f32, col: Rgba) {
        self.tri(a, b, c, z, col);
        self.tri(a, c, d, z, col);
    }

    /// Ribbon of `width` (tiles) from `a` to `b`.
    fn line(&mut self, a: Vec2, b: Vec2, width: f32, z: f32, col: Rgba) {
        let d = b - a;
        if d.length_squared() < 1e-10 {
            return;
        }
        let n = Vec2::new(-d.y, d.x).normalize() * (width * 0.5);
        self.quad(a - n, a + n, b + n, b - n, z, col);
    }

    fn polyline(&mut self, pts: &[Vec2], closed: bool, width: f32, z: f32, col: Rgba) {
        for w in pts.windows(2) {
            self.line(w[0], w[1], width, z, col);
        }
        if closed && pts.len() > 2 {
            self.line(pts[pts.len() - 1], pts[0], width, z, col);
        }
    }

    fn fan(&mut self, centre: Vec2, ring: &[Vec2], z: f32, col: Rgba) {
        for i in 0..ring.len() {
            self.tri(centre, ring[i], ring[(i + 1) % ring.len()], z, col);
        }
    }

    fn into_mesh(self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colors);
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }
}

/// Distinct faction colours (slot-indexed, 16 slots).
#[must_use]
pub fn faction_color(slot: u8) -> Rgba {
    const PALETTE: [[f32; 3]; 16] = [
        [0.20, 0.45, 0.95],
        [0.90, 0.25, 0.25],
        [0.25, 0.75, 0.35],
        [0.95, 0.75, 0.20],
        [0.65, 0.35, 0.85],
        [0.20, 0.80, 0.80],
        [0.95, 0.50, 0.15],
        [0.80, 0.80, 0.80],
        [0.55, 0.30, 0.15],
        [0.90, 0.45, 0.70],
        [0.50, 0.75, 0.15],
        [0.15, 0.30, 0.55],
        [0.70, 0.15, 0.30],
        [0.35, 0.60, 0.55],
        [0.60, 0.60, 0.25],
        [0.45, 0.45, 0.55],
    ];
    let c = PALETTE[slot as usize % 16];
    [c[0], c[1], c[2], 1.0]
}

fn with_alpha(c: Rgba, a: f32) -> Rgba {
    [c[0], c[1], c[2], a]
}

fn lerp_col(a: Rgba, b: Rgba, t: f32) -> Rgba {
    let t = t.clamp(0.0, 1.0);
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t, a[3] + (b[3] - a[3]) * t]
}

/// Minimum visible stroke: `px` screen pixels in tiles.
fn px_to_tiles(px: f32, px_per_tile: f32) -> f32 {
    px / px_per_tile.max(0.05)
}

/// Mean `|margin|` of the front cells touching a boundary edge (`None` when the edge has none).
fn edge_margin(cells: &std::collections::HashMap<(i32, i32), f32>, a: IVec2, b: IVec2) -> Option<f32> {
    let (lo, hi) = (a.min(b), a.max(b));
    let tiles: [(i32, i32); 2] = if lo.y == hi.y {
        // Horizontal edge: tiles above and below, per unit step along it.
        [(lo.x, lo.y - 1), (lo.x, lo.y)]
    } else {
        [(lo.x - 1, lo.y), (lo.x, lo.y)]
    };
    let (mut sum, mut n) = (0.0, 0.0);
    let steps = ((hi - lo).abs().max_element()).max(1);
    for s in 0..steps {
        for t in tiles {
            let t = if lo.y == hi.y { (t.0 + s, t.1) } else { (t.0, t.1 + s) };
            if let Some(m) = cells.get(&t) {
                sum += m.abs();
                n += 1.0;
            }
        }
    }
    (n > 0.0).then(|| sum / n)
}

/// Front ribbons: colour by margin (contested hot and thick), thickness floored to a few pixels.
pub(crate) fn build_front_lines(geo: &mut Geo, front: &FrontlineState, px_per_tile: f32, contest_band: f32) {
    for seg in &front.segments {
        let margins: std::collections::HashMap<(i32, i32), f32> =
            seg.cells.iter().map(|c| ((c.tile.x, c.tile.y), c.margin)).collect();
        for pl in &seg.polylines {
            let base = faction_color(pl.left_slot);
            let other = faction_color(pl.right_slot);
            let mut edges: Vec<(IVec2, IVec2)> = pl.points.windows(2).map(|w| (w[0], w[1])).collect();
            if pl.closed && pl.points.len() > 2 {
                edges.push((pl.points[pl.points.len() - 1], pl.points[0]));
            }
            for (a, b) in edges {
                let m = edge_margin(&margins, a, b).unwrap_or(1.0);
                // 0 = dead even, 1 = firm.
                let firm = (m / contest_band.max(1e-3)).clamp(0.0, 1.0);
                let hot = [1.0, 0.35, 0.1, 0.95];
                let col = lerp_col(hot, with_alpha(lerp_col(base, other, 0.35), 0.9), firm);
                let width = (0.34 - 0.18 * firm).max(px_to_tiles(2.5, px_per_tile));
                geo.line(a.as_vec2(), b.as_vec2(), width, FRONT_Z, col);
            }
        }
    }
}

/// Outline of each theater's area of responsibility.
pub(crate) fn build_theater_outlines(geo: &mut Geo, book: &TheaterBook, px_per_tile: f32) {
    let w = px_to_tiles(1.5, px_per_tile).max(0.06);
    for t in &book.theaters {
        let (mut lo, mut hi) = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
        for c in &t.cells {
            lo = lo.min(c.tile.as_vec2());
            hi = hi.max(c.tile.as_vec2() + Vec2::ONE);
        }
        if lo.x > hi.x {
            continue;
        }
        let (lo, hi) = (lo - Vec2::splat(book.depth_tiles), hi + Vec2::splat(book.depth_tiles));
        let col = lerp_col(faction_color(t.pair.0), faction_color(t.pair.1), 0.5);
        let ring = [lo, Vec2::new(hi.x, lo.y), hi, Vec2::new(lo.x, hi.y)];
        geo.polyline(&ring, true, w, THEATER_Z, with_alpha(col, 0.55));
    }
}

/// Triangles (Threatened) and diamonds (Opportunity); rank 1 is the largest and brightest.
pub(crate) fn build_pressure_markers(geo: &mut Geo, book: &PressurePointBook, px_per_tile: f32) {
    let unit = px_to_tiles(14.0, px_per_tile).clamp(0.7, 8.0);
    for (rank, p) in pressure_ranks(book).into_iter().map(|(r, p)| (r, *p)) {
        let k = 1.0 - 0.18 * rank as f32;
        let size = unit * k.max(0.55);
        let c = p.tile.as_vec2() + Vec2::splat(0.5);
        match p.kind {
            PressureKind::Threatened => {
                let col = [1.0, 0.2, 0.2, 0.55 + 0.4 * k.max(0.0)];
                geo.tri(c + Vec2::new(0.0, size * 0.6), c + Vec2::new(-size * 0.5, -size * 0.4), c + Vec2::new(size * 0.5, -size * 0.4), PRESSURE_Z, col);
            }
            PressureKind::Opportunity => {
                let col = [0.2, 0.95, 0.45, 0.55 + 0.4 * k.max(0.0)];
                geo.quad(
                    c + Vec2::new(0.0, size * 0.55),
                    c + Vec2::new(-size * 0.5, 0.0),
                    c + Vec2::new(0.0, -size * 0.55),
                    c + Vec2::new(size * 0.5, 0.0),
                    PRESSURE_Z,
                    col,
                );
            }
        }
        // A ring in the owning faction's colour says whose point it is.
        let ring: Vec<Vec2> = (0..8)
            .map(|i| c + Vec2::from_angle(i as f32 * std::f32::consts::TAU / 8.0) * (size * 0.62))
            .collect();
        geo.polyline(&ring, true, px_to_tiles(1.5, px_per_tile), PRESSURE_Z + 0.001, faction_color(p.side));
    }
}

/// `(rank within theater/side/kind starting at 0, point)`; the book keeps each group strongest first.
pub(crate) fn pressure_ranks(book: &PressurePointBook) -> Vec<(usize, &crate::combat::PressurePoint)> {
    let mut out = Vec::with_capacity(book.points.len());
    let mut last: Option<(u32, u8, PressureKind)> = None;
    let mut rank = 0;
    for p in &book.points {
        let key = (p.theater.0, p.side, p.kind);
        rank = if last == Some(key) { rank + 1 } else { 0 };
        last = Some(key);
        out.push((rank, p));
    }
    out
}

fn unit_icon(geo: &mut Geo, kind: UnitKind, c: Vec2, h: f32, w: f32, z: f32, ink: Rgba) {
    let ring = |rx: f32, ry: f32| -> Vec<Vec2> {
        (0..12).map(|i| c + Vec2::from_angle(i as f32 * std::f32::consts::TAU / 12.0) * Vec2::new(rx, ry)).collect()
    };
    let x = |geo: &mut Geo| {
        geo.line(c + Vec2::new(-h, -h), c + Vec2::new(h, h), w, z, ink);
        geo.line(c + Vec2::new(-h, h), c + Vec2::new(h, -h), w, z, ink);
    };
    match kind {
        UnitKind::Infantry => x(geo),
        UnitKind::Armor => geo.polyline(&ring(h * 1.2, h * 0.7), true, w, z, ink),
        UnitKind::Mechanized => {
            x(geo);
            geo.polyline(&ring(h * 1.2, h * 0.7), true, w, z, ink);
        }
        UnitKind::Artillery => geo.fan(c, &ring(h * 0.35, h * 0.35), z, ink),
        UnitKind::Recon => geo.line(c + Vec2::new(-h, -h), c + Vec2::new(h, h), w, z, ink),
        UnitKind::Engineer => {
            geo.line(c + Vec2::new(-h * 0.6, -h), c + Vec2::new(-h * 0.6, h), w, z, ink);
            for dy in [-h, 0.0, h] {
                geo.line(c + Vec2::new(-h * 0.6, dy), c + Vec2::new(h * 0.7, dy), w, z, ink);
            }
        }
        UnitKind::AirDefense => {
            let arc: Vec<Vec2> = (0..=6)
                .map(|i| c + Vec2::new(0.0, -h * 0.5) + Vec2::from_angle(i as f32 * std::f32::consts::PI / 6.0) * h * 1.1)
                .collect();
            geo.polyline(&arc, false, w, z, ink);
            geo.line(c + Vec2::new(-h * 1.1, -h * 0.5), c + Vec2::new(h * 1.1, -h * 0.5), w, z, ink);
        }
    }
}

/// Edge length (tiles) of a formation symbol at this zoom.
#[must_use]
pub fn symbol_size_tiles(px_per_tile: f32) -> f32 {
    px_to_tiles(SYMBOL_PX, px_per_tile).clamp(SYMBOL_MIN_TILES, SYMBOL_MAX_TILES)
}

/// One APP-6-style symbol. `friend`: rectangle frame; otherwise diamond.
pub(crate) fn build_formation_symbol(geo: &mut Geo, f: &Formation, friend: bool, selected: bool, px_per_tile: f32) {
    let s = symbol_size_tiles(px_per_tile);
    let c = f.tile.as_vec2() + Vec2::splat(0.5);
    let half = s * 0.5;
    let line_w = px_to_tiles(1.6, px_per_tile).min(s * 0.2);
    let fill = with_alpha(faction_color(f.faction_slot), 0.62);
    let edge = [0.05, 0.05, 0.08, 1.0];
    let ink = [0.97, 0.97, 0.97, 1.0];
    let z = SYMBOL_Z;
    if friend {
        let (a, b, d, e) = (c + Vec2::new(-half, -half * 0.75), c + Vec2::new(half, -half * 0.75), c + Vec2::new(half, half * 0.75), c + Vec2::new(-half, half * 0.75));
        geo.quad(a, b, d, e, z, fill);
        geo.polyline(&[a, b, d, e], true, line_w, z + 0.001, edge);
    } else {
        let r = half * 1.1;
        let pts = [c + Vec2::new(0.0, -r), c + Vec2::new(r, 0.0), c + Vec2::new(0.0, r), c + Vec2::new(-r, 0.0)];
        geo.quad(pts[0], pts[1], pts[2], pts[3], z, fill);
        geo.polyline(&pts, true, line_w, z + 0.001, edge);
    }
    unit_icon(geo, f.kind, c, half * 0.45, line_w, z + 0.002, ink);

    // Strength (green to red) and morale (blue) bars under the frame.
    let bar_h = (s * 0.09).max(px_to_tiles(2.0, px_per_tile));
    let base_y = c.y - half * 0.75 - bar_h * 1.6;
    for (i, (v, col)) in [
        (f.strength, lerp_col([0.9, 0.15, 0.1, 1.0], [0.2, 0.85, 0.25, 1.0], f.strength)),
        (f.morale, [0.35, 0.6, 1.0, 1.0]),
    ]
    .into_iter()
    .enumerate()
    {
        let y = base_y - i as f32 * bar_h * 1.4;
        let (x0, x1) = (c.x - half, c.x + half);
        geo.quad(Vec2::new(x0, y - bar_h * 0.5), Vec2::new(x1, y - bar_h * 0.5), Vec2::new(x1, y + bar_h * 0.5), Vec2::new(x0, y + bar_h * 0.5), z, [0.0, 0.0, 0.0, 0.7]);
        let xv = x0 + (x1 - x0) * v.clamp(0.0, 1.0);
        geo.quad(Vec2::new(x0, y - bar_h * 0.5), Vec2::new(xv, y - bar_h * 0.5), Vec2::new(xv, y + bar_h * 0.5), Vec2::new(x0, y + bar_h * 0.5), z + 0.001, col);
    }
    // Dig-in: a brown underline growing with the works.
    if f.dug_in > 0.02 {
        let y = c.y + half * 0.75 + bar_h;
        geo.line(Vec2::new(c.x - half, y), Vec2::new(c.x - half + s * f.dug_in.clamp(0.0, 1.0), y), bar_h * 1.2, z, [0.55, 0.38, 0.18, 1.0]);
    }
    // Posture arrow.
    let toward = f.target.map(|t| (t.as_vec2() + Vec2::splat(0.5) - c).normalize_or_zero()).filter(|d| *d != Vec2::ZERO);
    let arrow = |geo: &mut Geo, dir: Vec2, len: f32, col: Rgba| {
        let tip = c + dir * (half + len);
        let from = c + dir * half;
        geo.line(from, tip, line_w * 1.6, z + 0.003, col);
        let side = Vec2::new(-dir.y, dir.x);
        geo.tri(tip + dir * (len * 0.3), tip + side * (len * 0.3), tip - side * (len * 0.3), z + 0.003, col);
    };
    match f.posture {
        Posture::Attack => arrow(geo, toward.unwrap_or(Vec2::X), s * 0.9, [1.0, 0.25, 0.15, 1.0]),
        Posture::Move => arrow(geo, toward.unwrap_or(Vec2::X), s * 0.5, [0.95, 0.9, 0.3, 1.0]),
        Posture::Retreat => arrow(geo, -toward.unwrap_or(Vec2::X), s * 0.7, [1.0, 0.6, 0.1, 1.0]),
        Posture::Hold => {}
    }
    if selected {
        let r = half * 1.35;
        let ring = [c + Vec2::new(-r, -r), c + Vec2::new(r, -r), c + Vec2::new(r, r), c + Vec2::new(-r, r)];
        geo.polyline(&ring, true, line_w * 1.8, z + 0.004, [1.0, 0.92, 0.2, 1.0]);
    }
}

// ---------------------------------------------------------------------------------------------
// Sync
// ---------------------------------------------------------------------------------------------

fn mix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

fn px_per_tile(trace: &MainWorldCameraOrthoTrace) -> f32 {
    if trace.fixed_width > 0.0 && trace.view_pixels.x > 0.0 {
        trace.view_pixels.x / trace.fixed_width
    } else {
        1.0
    }
}

/// Zoom band: strokes are pixel-sized, so rebuild when px/tile moves by roughly 12%.
fn zoom_band(px: f32) -> i64 {
    (px.max(0.05).log2() * 6.0).round() as i64
}

#[derive(Default)]
struct LayerState {
    built_key: Option<u64>,
    mesh: Option<Handle<Mesh>>,
    material: Option<Handle<ColorMaterial>>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn sync_world_map_front_overlay(
    mut commands: Commands,
    base: Res<State<BaseState>>,
    mode: Option<Res<WorldPresentationMode>>,
    settings: Option<Res<FrontOverlaySettings>>,
    front: Option<Res<FrontlineState>>,
    theaters: Option<Res<TheaterBook>>,
    pressure: Option<Res<PressurePointBook>>,
    balance: Option<Res<FrontBalance>>,
    selected: Option<Res<SelectedFormation>>,
    trace: Option<Res<MainWorldCameraOrthoTrace>>,
    formations: Query<(Entity, &Formation)>,
    mut layer: Query<(Entity, &mut Visibility), With<WorldMapFrontLayer>>,
    labels: Query<Entity, With<WorldMapFrontLabel>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut state: Local<LayerState>,
) {
    let tactical = mode.as_deref().is_none_or(|m| m.is_tactical());
    if !matches!(base.get(), BaseState::Simulation) || !tactical {
        // PixelWorld3D (or leaving the sim): hide the layer and drop labels; the mesh is kept for the way back.
        for (_, mut v) in &mut layer {
            v.set_if_neq(Visibility::Hidden);
        }
        for e in &labels {
            commands.entity(e).despawn();
        }
        state.built_key = None;
        return;
    }
    let settings = settings.as_deref().copied().unwrap_or_default();
    let default_trace = MainWorldCameraOrthoTrace::default();
    let px = px_per_tile(trace.as_deref().unwrap_or(&default_trace));
    let selected = selected.as_deref().and_then(|s| s.0);
    let contest_band = balance.as_deref().map_or(0.3, |b| b.front.contest_band);

    let mut key = mix64(zoom_band(px) as u64 ^ (u64::from(settings.fronts) | u64::from(settings.theaters) << 1 | u64::from(settings.pressure) << 2 | u64::from(settings.formations) << 3 | u64::from(settings.labels) << 4) << 40);
    if let Some(f) = front.as_deref() {
        key = mix64(key ^ f.revision);
    }
    if let Some(t) = theaters.as_deref() {
        key = mix64(key ^ t.source_revision ^ (t.theaters.len() as u64) << 32);
    }
    if let Some(p) = pressure.as_deref() {
        key = mix64(key ^ p.source_tick ^ (p.source_theater_revision << 20) ^ p.points.len() as u64);
        for pt in &p.points {
            key = mix64(key ^ (pt.tile.x as u32 as u64) ^ (pt.tile.y as u32 as u64) << 32 ^ (pt.score * 64.0) as u64);
        }
    }
    let mut fs: Vec<(Entity, &Formation)> = formations.iter().collect();
    fs.sort_by_key(|(e, _)| e.to_bits());
    for (e, f) in &fs {
        key = mix64(
            key ^ e.to_bits()
                ^ (f.tile.x as u32 as u64) << 8
                ^ (f.tile.y as u32 as u64) << 24
                ^ ((f.strength * 32.0) as u64) << 40
                ^ ((f.morale * 32.0) as u64) << 46
                ^ (f.posture as u64) << 52
                ^ ((f.dug_in * 8.0) as u64) << 56
                ^ f.target.map_or(0, |t| (t.x as u32 as u64) ^ (t.y as u32 as u64) << 16),
        );
    }
    key = mix64(key ^ selected.map_or(0, |e| e.to_bits()));

    let mesh_handle = state.mesh.get_or_insert_with(|| meshes.add(Geo::default().into_mesh())).clone();
    let material = state.material.get_or_insert_with(|| materials.add(ColorMaterial::from_color(Color::WHITE))).clone();

    if state.built_key != Some(key) {
        let mut geo = Geo::default();
        if settings.theaters {
            if let Some(t) = theaters.as_deref() {
                build_theater_outlines(&mut geo, t, px);
            }
        }
        if settings.fronts {
            if let Some(f) = front.as_deref() {
                build_front_lines(&mut geo, f, px, contest_band);
            }
        }
        if settings.pressure {
            if let Some(p) = pressure.as_deref() {
                build_pressure_markers(&mut geo, p, px);
            }
        }
        if settings.formations {
            for (e, f) in &fs {
                build_formation_symbol(&mut geo, f, f.faction_slot == PLAYER_SLOT, selected == Some(*e), px);
            }
        }
        meshes.insert(&mesh_handle, geo.into_mesh());

        for e in &labels {
            commands.entity(e).despawn();
        }
        if settings.labels && px >= 1.5 {
            let scale = px_to_tiles(13.0, px) / 32.0;
            let mut spawn_label = |text: String, at: Vec2, color: Color| {
                commands.spawn((
                    WorldMapFrontLabel,
                    Text2d::new(text),
                    TextFont { font_size: FontSize::Px(32.0), ..default() },
                    TextColor(color),
                    bevy::sprite::Anchor::CENTER,
                    Transform::from_translation(at.extend(LABEL_Z)).with_scale(Vec3::splat(scale)),
                    Pickable::IGNORE,
                ));
            };
            if settings.theaters {
                if let Some(t) = theaters.as_deref() {
                    for th in &t.theaters {
                        spawn_label(
                            format!("T{} ({}v{})", th.id.0, th.pair.0, th.pair.1),
                            th.centroid + Vec2::splat(0.5),
                            Color::srgba(0.92, 0.95, 1.0, 0.9),
                        );
                    }
                }
            }
            if settings.pressure {
                if let Some(p) = pressure.as_deref() {
                    let unit = px_to_tiles(14.0, px).clamp(0.7, 8.0);
                    for (rank, pt) in pressure_ranks(p) {
                        spawn_label(
                            format!("{}", rank + 1),
                            pt.tile.as_vec2() + Vec2::new(0.5, 0.5 - unit * 0.75),
                            Color::srgba(1.0, 1.0, 1.0, 0.85),
                        );
                    }
                }
            }
        }
        state.built_key = Some(key);
    }

    if layer.is_empty() {
        commands.spawn((
            WorldMapFrontLayer,
            Mesh2d(mesh_handle),
            MeshMaterial2d(material),
            Transform::default(),
            Visibility::Visible,
            Pickable::IGNORE,
        ));
    } else {
        for (_, mut v) in &mut layer {
            v.set_if_neq(Visibility::Visible);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Selection and orders
// ---------------------------------------------------------------------------------------------

/// Publish the map-plane cursor (after the pointer gate has decided who owns the pointer).
#[allow(clippy::too_many_arguments)]
pub(crate) fn update_front_map_cursor_system(
    window: Query<&Window, With<PrimaryWindow>>,
    authority: Option<Res<ViewProjectionAuthority>>,
    desired: Option<Res<MapCameraDesiredRes>>,
    map_vp: Option<Res<SimulationMapViewport>>,
    params: Option<Res<WorldGenParams>>,
    gate: Option<Res<SimulationMapPointerGate>>,
    trace: Option<Res<MainWorldCameraOrthoTrace>>,
    mode: Option<Res<WorldPresentationMode>>,
    mut cursor: ResMut<FrontMapCursor>,
) {
    let mut next = FrontMapCursor::default();
    let tactical = mode.as_deref().is_none_or(|m| m.is_tactical());
    if tactical {
        next.px_per_tile = trace.as_deref().map_or(1.0, px_per_tile);
        next.free = gate.as_deref().is_some_and(|g| g.in_play_area)
            && simulation_tool_pointer(gate.as_deref()) == SimulationToolPointer::Free;
        if let (Ok(w), Some(desired), Some(map_vp), Some(params)) = (window.single(), desired, map_vp, params) {
            next.xy = cursor_world_on_map(w, authority.as_deref(), desired.as_ref(), map_vp.as_ref(), params.as_ref())
                .map(|v| Vec2::new(v.x, v.z));
        }
    }
    cursor.set_if_neq(next);
}

/// What a right click on `target` means for a formation of `slot`.
#[must_use]
pub fn order_kind_for_target(
    slot: u8,
    target: IVec2,
    theaters: &TheaterBook,
    pressure: Option<&PressurePointBook>,
    hostile_formation_tiles: &[IVec2],
) -> DeploymentKind {
    let near = |a: IVec2, b: IVec2| (a - b).as_vec2().length() <= 1.5;
    if hostile_formation_tiles.iter().any(|t| near(*t, target)) {
        return DeploymentKind::Attack(target);
    }
    let enemy_front = theaters.theaters.iter().flat_map(|t| t.cells.iter()).any(|c| c.tile == target && c.owner != slot);
    if enemy_front {
        return DeploymentKind::Attack(target);
    }
    if let Some(p) = pressure.and_then(|b| {
        b.points
            .iter()
            .filter(|p| p.side == slot && p.kind == PressureKind::Threatened && near(p.tile, target))
            .min_by(|a, b| {
                (a.tile - target).as_vec2().length().total_cmp(&(b.tile - target).as_vec2().length())
            })
    }) {
        return DeploymentKind::Reinforce { theater: p.theater, tile: p.tile };
    }
    DeploymentKind::Move(target)
}

/// Left click selects the player's formation under the cursor (empty ground deselects); right click
/// orders the selection through [`DeploymentOrderQueue::enqueue`].
#[allow(clippy::too_many_arguments)]
pub fn select_and_order_formations_system(
    buttons: Res<ButtonInput<MouseButton>>,
    cursor: Res<FrontMapCursor>,
    tool: Option<Res<ActiveBuildTool>>,
    theaters: Option<Res<TheaterBook>>,
    pressure: Option<Res<PressurePointBook>>,
    formations: Query<(Entity, &Formation)>,
    mut selected: ResMut<SelectedFormation>,
    mut queue: ResMut<DeploymentOrderQueue>,
) {
    // A formation that died or changed hands is no longer selected.
    if let Some(e) = selected.0 {
        if formations.get(e).map_or(true, |(_, f)| f.faction_slot != PLAYER_SLOT) {
            selected.0 = None;
        }
    }
    if !cursor.free || tool.as_deref().is_some_and(|t| t.tool != BuildTool::None) {
        return;
    }
    let Some(at) = cursor.xy else { return };
    let tile = IVec2::new(at.x.floor() as i32, at.y.floor() as i32);

    if buttons.just_pressed(MouseButton::Left) {
        let reach = (symbol_size_tiles(cursor.px_per_tile.max(0.05)) * 0.7).max(0.8);
        let hit = formations
            .iter()
            .filter(|(_, f)| f.faction_slot == PLAYER_SLOT)
            .map(|(e, f)| (e, (f.tile.as_vec2() + Vec2::splat(0.5) - at).length()))
            .filter(|(_, d)| *d <= reach)
            .min_by(|a, b| a.1.total_cmp(&b.1).then(a.0.to_bits().cmp(&b.0.to_bits())));
        let next = hit.map(|(e, _)| e);
        if selected.0 != next {
            selected.0 = next;
        }
        return;
    }
    if buttons.just_pressed(MouseButton::Right) {
        let Some(sel) = selected.0 else { return };
        let Ok((_, own)) = formations.get(sel) else { return };
        let hostile: Vec<IVec2> =
            formations.iter().filter(|(_, f)| f.faction_slot != own.faction_slot).map(|(_, f)| f.tile).collect();
        let empty = TheaterBook::default();
        let kind = order_kind_for_target(
            own.faction_slot,
            tile,
            theaters.as_deref().unwrap_or(&empty),
            pressure.as_deref(),
            &hostile,
        );
        // A refused enqueue (full queue) just drops the click; the player can repeat it.
        let _ = queue.enqueue(DeploymentOrder { formation: sel, kind, issued_by: own.faction_slot });
    }
}

pub struct WorldMapFrontMeshPlugin;

impl Plugin for WorldMapFrontMeshPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrontOverlaySettings>()
            .init_resource::<SelectedFormation>()
            .init_resource::<FrontMapCursor>()
            .add_systems(
                Update,
                (
                    update_front_map_cursor_system,
                    select_and_order_formations_system,
                    sync_world_map_front_overlay,
                )
                    .chain()
                    .after(PointerGateSet::Publish),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::front::{FrontCell, FrontPolyline, FrontSegment};

    fn front_with_margin(margin: f32) -> FrontlineState {
        // Vertical boundary at x = 3 between slot 0 (x = 2) and slot 1 (x = 3), 4 rows.
        let mut cells = Vec::new();
        for y in 0..4 {
            cells.push(FrontCell { tile: IVec2::new(2, y), owner: 0, facing: 1, contested: false, margin });
            cells.push(FrontCell { tile: IVec2::new(3, y), owner: 1, facing: 0, contested: false, margin });
        }
        FrontlineState {
            segments: vec![FrontSegment {
                pair: (0, 1),
                cells,
                polylines: vec![FrontPolyline {
                    points: vec![IVec2::new(3, 0), IVec2::new(3, 4)],
                    left_slot: 0,
                    right_slot: 1,
                    closed: false,
                    edge_count: 4,
                }],
                contested_cells: 0,
                length_m: 100.0,
            }],
            revision: 1,
            ..default()
        }
    }

    #[test]
    fn contested_front_is_hotter_and_thicker_than_a_firm_one() {
        let mut hot = Geo::default();
        let mut firm = Geo::default();
        build_front_lines(&mut hot, &front_with_margin(0.02), 20.0, 0.3);
        build_front_lines(&mut firm, &front_with_margin(0.9), 20.0, 0.3);
        assert!(!hot.positions.is_empty() && !firm.positions.is_empty());
        // Hot is red-dominant; firm blends the faction colours (blue-ish).
        assert!(hot.colors[0][0] > 0.9 && firm.colors[0][0] < 0.6, "{:?} vs {:?}", hot.colors[0], firm.colors[0]);
        let width = |g: &Geo| (g.positions[1][0] - g.positions[0][0]).abs().max((g.positions[1][1] - g.positions[0][1]).abs());
        assert!(width(&hot) > width(&firm));
    }

    #[test]
    fn friend_and_hostile_frames_differ_and_every_kind_has_an_icon() {
        let mut f = Formation::new(0, UnitKind::Infantry, IVec2::new(5, 5));
        let (mut a, mut b) = (Geo::default(), Geo::default());
        build_formation_symbol(&mut a, &f, true, false, 10.0);
        f.faction_slot = 1;
        build_formation_symbol(&mut b, &f, false, false, 10.0);
        assert_ne!(a.positions, b.positions);
        for kind in UnitKind::ALL {
            let mut with = Geo::default();
            unit_icon(&mut with, kind, Vec2::new(5.0, 5.0), 0.3, 0.05, 1.0, [1.0; 4]);
            assert!(!with.positions.is_empty(), "{kind:?}");
        }
    }

    fn app_with_formation() -> (App, Entity) {
        let mut app = App::new();
        app.init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<DeploymentOrderQueue>()
            .init_resource::<SelectedFormation>()
            .init_resource::<FrontMapCursor>()
            .add_systems(Update, select_and_order_formations_system);
        let e = app.world_mut().spawn(Formation::new(0, UnitKind::Infantry, IVec2::new(10, 10))).id();
        app.world_mut().spawn(Formation::new(1, UnitKind::Armor, IVec2::new(30, 10)));
        (app, e)
    }

    fn click(app: &mut App, button: MouseButton, at: Vec2) {
        *app.world_mut().resource_mut::<FrontMapCursor>() = FrontMapCursor { xy: Some(at), free: true, px_per_tile: 20.0 };
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(button);
        app.update();
        let mut input = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
        input.release(button);
        input.clear();
    }

    #[test]
    fn select_then_right_click_enqueues_move_and_attack_orders() {
        let (mut app, e) = app_with_formation();
        click(&mut app, MouseButton::Left, Vec2::new(10.5, 10.5));
        assert_eq!(app.world().resource::<SelectedFormation>().0, Some(e));
        click(&mut app, MouseButton::Right, Vec2::new(18.5, 12.5));
        click(&mut app, MouseButton::Right, Vec2::new(30.5, 10.5));
        let orders: Vec<_> = app.world().resource::<DeploymentOrderQueue>().iter().map(|(_, o)| *o).collect();
        assert_eq!(orders.len(), 2);
        assert!(orders.iter().all(|o| o.formation == e && o.issued_by == 0));
        assert_eq!(orders[0].kind, DeploymentKind::Move(IVec2::new(18, 12)));
        assert_eq!(orders[1].kind, DeploymentKind::Attack(IVec2::new(30, 10)), "enemy formation = attack");
        // Click on empty ground deselects; the gate being closed ignores clicks.
        click(&mut app, MouseButton::Left, Vec2::new(50.5, 50.5));
        assert_eq!(app.world().resource::<SelectedFormation>().0, None);
        click(&mut app, MouseButton::Left, Vec2::new(10.5, 10.5));
        app.world_mut().resource_mut::<FrontMapCursor>().free = false;
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Right);
        app.update();
        assert_eq!(app.world().resource::<DeploymentOrderQueue>().len(), 2, "no order while chrome owns the pointer");
    }

    #[test]
    fn enemy_only_formations_are_not_selectable() {
        let (mut app, _) = app_with_formation();
        click(&mut app, MouseButton::Left, Vec2::new(30.5, 10.5));
        assert_eq!(app.world().resource::<SelectedFormation>().0, None);
    }
}
