//! Road selection + info card (Bevy UI), 2D and voxel 3D alike: a click on a drawn road or bridge
//! selects that committed segment; the card shows its kind, endpoints, length, condition
//! (open / flooded / debris-blocked) and the trucks routed over it.
//!
//! Pick: the shared map cursor (`FrontMapCursor`, fed by the ortho pick in 2D and by the voxel
//! ray pick, which hits bridge decks, in 3D). Writes only [`HudRoadFocus`]. Read-only otherwise.

use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::construction::{ActiveBuildTool, CorridorNetwork};
use crate::economy::logistics::debris_block::DebrisRouteBlockBook;
use crate::entities::components::Waypoints;
use crate::gui::front_controls::{build_tool_blocks_front_orders, FrontMapCursor, SelectedFormation};
use crate::gui::hud::simulation_pointer_gate::{simulation_tool_pointer, SimulationMapPointerGate, SimulationToolPointer};
use crate::gui::logistics_focus::{HudLogisticsFocus, HudRoadFocus, HudRoadReport};
use crate::gui::style::{CmdUiMonoFont, UiPalette};
use crate::gui::WorldPresentationMode;
use crate::presentation::TruckPresentation;
use crate::render::pixel_world::PixelWorldRoadSurface;
use crate::strategic::BuildSiteTile;
use crate::systems::water::FloodRouteBlockBook;
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

/// Half the drawn ribbon width (tiles) added to the click radius.
const ROAD_RIBBON_HALF: f32 = 0.11;

fn centre(t: BuildSiteTile) -> Vec2 {
    Vec2::new(t.x as f32 + 0.5, t.z as f32 + 0.5)
}

fn dist_to_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let d = b - a;
    let t = if d.length_squared() < 1e-6 { 0.0 } else { ((p - a).dot(d) / d.length_squared()).clamp(0.0, 1.0) };
    p.distance(a + d * t)
}

/// Nearest committed road segment within `radius` tiles of `xy`.
#[must_use]
pub fn pick_road_segment(roads: &CorridorNetwork, xy: Vec2, radius: f32) -> Option<(IVec2, IVec2)> {
    roads
        .road_spans()
        .map(|(_, a, b)| (dist_to_segment(xy, centre(a), centre(b)), a, b))
        .filter(|(d, ..)| *d <= radius + ROAD_RIBBON_HALF)
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, a, b)| (IVec2::new(a.x as i32, a.z as i32), IVec2::new(b.x as i32, b.z as i32)))
}

/// Idle LMB on a road or bridge selects it. Yields to build tools, formation selection and sites
/// (a site hit keeps the site card).
pub fn map_pick_road_focus_system(
    buttons: Res<ButtonInput<MouseButton>>,
    cursor: Res<FrontMapCursor>,
    tool: Option<Res<ActiveBuildTool>>,
    selected: Res<SelectedFormation>,
    roads: Res<CorridorNetwork>,
    site_focus: Res<HudLogisticsFocus>,
    mut focus: ResMut<HudRoadFocus>,
    gate: Option<Res<SimulationMapPointerGate>>,
) {
    if !buttons.just_released(MouseButton::Left)
        || !cursor.free
        || simulation_tool_pointer(gate.as_deref()) != SimulationToolPointer::Free
        || build_tool_blocks_front_orders(tool.as_deref())
        || !selected.0.is_empty()
    {
        return;
    }
    let picked = match (cursor.xy, site_focus.tracked_entity) {
        (Some(xy), None) => {
            let radius = (7.0 / cursor.px_per_tile.max(0.05)).clamp(0.35, 2.0);
            pick_road_segment(&roads, xy, radius)
        }
        _ => None,
    };
    if focus.segment != picked {
        focus.segment = picked;
    }
}

/// Build the card lines for the selected road (empty when nothing is selected or it was removed).
#[allow(clippy::too_many_arguments)]
pub fn sync_road_report_system(
    focus: Res<HudRoadFocus>,
    roads: Res<CorridorNetwork>,
    mode: Option<Res<WorldPresentationMode>>,
    surface: Option<Res<PixelWorldRoadSurface>>,
    flood: Option<Res<FloodRouteBlockBook>>,
    debris: Option<Res<DebrisRouteBlockBook>>,
    trucks: Query<&Waypoints, With<TruckPresentation>>,
    mut report: ResMut<HudRoadReport>,
) {
    let as_iv = |t: crate::strategic::BuildSiteTile| IVec2::new(t.x as i32, t.z as i32);
    let lines = focus
        .segment
        .and_then(|(a, b)| {
            roads
                .road_spans()
                .find(|(_, p, q)| as_iv(*p) == a && as_iv(*q) == b)
                .and_then(|(seg, ..)| seg.kind.road_tier())
                .map(|tier| (a, b, tier))
        })
        .map(|(a, b, tier)| {
            let (ca, cb) = (a.as_vec2() + Vec2::splat(0.5), b.as_vec2() + Vec2::splat(0.5));
            let bridge = mode.as_deref().is_some_and(|m| m.is_pixel_world())
                && surface.as_deref().is_some_and(|s| {
                    s.0.iter().any(|g| g.deck.is_some() && g.a.distance(ca) < 0.01 && g.b.distance(cb) < 0.01)
                });
            let key = |t: IVec2| format!("t{}_{}", t.x, t.y);
            let (ka, kb) = (key(a), key(b));
            let flooded = flood.as_deref().is_some_and(|f| f.blocks_key(&ka) || f.blocks_key(&kb));
            let rubble = debris.as_deref().is_some_and(|d| d.blocks_key(&ka) || d.blocks_key(&kb));
            let condition = match (flooded, rubble) {
                (true, true) => "BLOCKED: flooded + debris",
                (true, false) => "BLOCKED: flooded",
                (false, true) => "BLOCKED: debris / crater",
                _ => "open",
            };
            let (ma, mb) = (ca * TILE_METRES_GAMEPLAY, cb * TILE_METRES_GAMEPLAY);
            let traffic = trucks
                .iter()
                .filter(|wp| {
                    wp.points.windows(2).any(|w| {
                        (w[0].distance(ma) < 1.0 && w[1].distance(mb) < 1.0) || (w[0].distance(mb) < 1.0 && w[1].distance(ma) < 1.0)
                    })
                })
                .count();
            vec![
                if bridge { format!("{} bridge", tier.label()) } else { tier.label().to_owned() },
                format!("Speed: {:.0} km/h, {} lane(s)", tier.spec().speed_kmh, tier.spec().lanes_default),
                format!("From ({},{}) to ({},{})", a.x, a.y, b.x, b.y),
                format!("Length: {:.0} m", ca.distance(cb) * TILE_METRES_GAMEPLAY),
                format!("Condition: {condition}"),
                format!("Traffic: {traffic} truck(s) routed"),
            ]
        })
        .unwrap_or_default();
    if report.0 != lines {
        report.0 = lines;
    }
}

#[derive(Component)]
pub struct RoadInfoCardRoot;

#[derive(Component)]
pub struct RoadInfoCardText;

/// Spawn / update / despawn the card from [`HudRoadReport`] (same look as the plant info card).
pub fn sync_road_info_card_system(
    mut commands: Commands,
    report: Res<HudRoadReport>,
    roots: Query<Entity, With<RoadInfoCardRoot>>,
    mut texts: Query<&mut Text, With<RoadInfoCardText>>,
    palette: Option<Res<UiPalette>>,
    mono: Option<Res<CmdUiMonoFont>>,
    placements: Res<crate::gui::hud::panel_drag::HudCardPlacements>,
) {
    if report.0.is_empty() {
        for e in &roots {
            commands.entity(e).despawn();
        }
        return;
    }
    let body = report.0.join("\n");
    if let Some(mut text) = texts.iter_mut().next() {
        if text.0 != body {
            text.0 = body;
        }
        return;
    }
    let (Some(palette), Some(mono)) = (palette, mono) else {
        return;
    };
    // Movable like every info card; reopens where the player left it.
    let (anchor, drag, right, top) = placements.card("road_card", Vec2::new(12.0, 120.0));
    commands
        .spawn((
            RoadInfoCardRoot,
            anchor,
            drag,
            Node {
                position_type: PositionType::Absolute,
                top,
                right,
                width: Val::Px(300.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::ZERO,
                ..default()
            },
            BackgroundColor(palette.bevy_hud_panel_fill()),
            BorderColor::all(palette.bevy_wire_magenta()),
            FocusPolicy::Block,
            ZIndex(950),
        ))
        .with_children(|root| {
            root.spawn((
                RoadInfoCardText,
                Text::new(body),
                TextFont::from_font_size(11.5).with_font(mono.0.clone()),
                TextColor(palette.bevy_primary_text()),
            ));
        });
}
