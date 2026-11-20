//! Debug-build cursor readout: the world tile under the cursor, its ground height and water
//! depth, so playtest reports can name exact coordinates ("black line at (162, 151)").
//!
//! **Debug builds only** (`cfg(debug_assertions)` at registration): AGENTS.md keeps dev panels
//! out of the shipped play UI; release builds compile this out. Read-only: it reads the map
//! cursor ([`FrontMapCursor`], same pick the build tools use) and the voxel / water fields.

use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::gui::front_controls::FrontMapCursor;

#[derive(Component)]
struct DebugCursorReadout;

/// Text for a cursor position (pure, testable).
#[must_use]
pub fn cursor_readout_text(xy: Option<Vec2>, ground_m: Option<f32>, water_m: Option<f32>) -> String {
    match xy {
        None => "cursor -".to_string(),
        Some(p) => {
            let mut s = format!("tile ({}, {})", p.x.floor() as i32, p.y.floor() as i32);
            if let Some(g) = ground_m {
                s.push_str(&format!("  ground {g:.0} m"));
            }
            if let Some(w) = water_m.filter(|w| *w > 0.01) {
                s.push_str(&format!("  water {w:.1} m"));
            }
            s
        }
    }
}

fn debug_cursor_readout_system(
    mut commands: Commands,
    cursor: Res<FrontMapCursor>,
    field: Option<Res<crate::render::pixel_world::PixelWorldTerrainField>>,
    water: crate::systems::water::WaterTileProbe,
    mut q: Query<&mut Text, With<DebugCursorReadout>>,
) {
    let ground = cursor.xy.zip(field.as_deref()).and_then(|(p, f)| {
        let i = f.index(p.x.floor() as i32, p.y.floor() as i32)?;
        Some(f.ground[i] / f.units_per_metre.max(1e-6))
    });
    let depth = cursor
        .xy
        .filter(|_| water.seeded())
        .map(|p| water.depth_m(IVec2::new(p.x.floor() as i32, p.y.floor() as i32)));
    let text = cursor_readout_text(cursor.xy, ground, depth);
    if let Ok(mut t) = q.single_mut() {
        if t.0 != text {
            t.0 = text;
        }
        return;
    }
    commands.spawn((
        DebugCursorReadout,
        Text::new(text),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(12.0),
            top: Val::Px(56.0),
            padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.06, 0.09, 0.11, 0.80)),
        TextFont::from_font_size(13.0),
        TextColor(Color::srgb(0.95, 0.85, 0.40)),
        FocusPolicy::Pass,
        Pickable::IGNORE,
        ZIndex(970),
    ));
}

/// Registered only in debug builds (see [`crate::gui::hud::dock_shell`]).
pub struct DebugCursorReadoutPlugin;

impl Plugin for DebugCursorReadoutPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            debug_cursor_readout_system.run_if(crate::gui::ui_gates::in_simulation_or_editor),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readout_names_tile_ground_and_water() {
        assert_eq!(cursor_readout_text(None, None, None), "cursor -");
        assert_eq!(
            cursor_readout_text(Some(Vec2::new(162.7, 151.2)), Some(540.4), Some(3.46)),
            "tile (162, 151)  ground 540 m  water 3.5 m"
        );
        assert_eq!(cursor_readout_text(Some(Vec2::new(10.0, 20.0)), Some(300.0), Some(0.0)), "tile (10, 20)  ground 300 m");
    }
}
