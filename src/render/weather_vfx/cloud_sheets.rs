//! W3 — translucent drifting cloud sheets (procedural quads; not volumetric).

use bevy::prelude::*;

use crate::gui::MainWorldCamera;
use crate::render::extraction::extracted_camera_metrics::{
    ExtractedCameraMetrics, ExtractedCameraMetricsSet,
};
use crate::systems::atmosphere::pipeline::AtmospherePipelineSet;
use crate::systems::weather::CameraLocalWeatherSample;
use crate::systems::weather::WEATHER_TACTICAL_PRECIP_ZOOM_ALPHA;

const SHEET_COUNT: usize = 6;
const SHEET_Z_LOW: f32 = 420.0;
const SHEET_Z_HIGH: f32 = 520.0;

#[derive(Component)]
pub struct WeatherCloudSheet {
    pub index: u8,
}

#[derive(Resource, Default)]
struct WeatherCloudSheetMaterials {
    handles: Vec<Handle<ColorMaterial>>,
}

fn attach_cloud_sheets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    cameras: Query<Entity, With<MainWorldCamera>>,
    existing: Query<Entity, With<WeatherCloudSheet>>,
    roots: Query<Entity, With<crate::systems::weather::WeatherVfxCameraChild>>,
) {
    if !existing.is_empty() {
        return;
    }
    let Ok(cam) = cameras.single() else {
        return;
    };
    let rtt = crate::gui::simulation_map_rtt_render_layers();
    let mut handles = Vec::with_capacity(SHEET_COUNT);
    let parent = if let Ok(root) = roots.single() {
        root
    } else {
        let root = commands
            .spawn((
                crate::systems::weather::WeatherVfxCameraChild,
                rtt.clone(),
                Name::new("WeatherCloudSheetRoot"),
                Transform::default(),
                Visibility::Visible,
            ))
            .id();
        commands.entity(cam).add_child(root);
        root
    };

    for i in 0..SHEET_COUNT {
        let mat = materials.add(ColorMaterial::from_color(Color::srgba(0.85, 0.88, 0.92, 0.0)));
        handles.push(mat.clone());
        let mesh = meshes.add(Rectangle::new(280.0 + i as f32 * 40.0, 160.0 + i as f32 * 18.0));
        let z = if i % 2 == 0 { SHEET_Z_LOW } else { SHEET_Z_HIGH };
        let child = commands
            .spawn((
                WeatherCloudSheet { index: i as u8 },
                rtt.clone(),
                Mesh2d(mesh),
                MeshMaterial2d(mat),
                Transform::from_translation(Vec3::new(
                    (i as f32 - 2.5) * 90.0,
                    (i as f32 % 3.0 - 1.0) * 70.0,
                    z,
                )),
                Visibility::Visible,
            ))
            .id();
        commands.entity(parent).add_child(child);
    }
    commands.insert_resource(WeatherCloudSheetMaterials { handles });
}

fn update_cloud_sheets(
    local: Res<CameraLocalWeatherSample>,
    cam: Res<ExtractedCameraMetrics>,
    time: Res<Time>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    sheet_mats: Option<Res<WeatherCloudSheetMaterials>>,
    mut sheets: Query<(&WeatherCloudSheet, &mut Transform, &mut Visibility)>,
) {
    let Some(mats) = sheet_mats else {
        return;
    };
    let cloud = if local.ready {
        local.sample.cloud.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let wind = if local.ready {
        local.sample.wind_mps
    } else {
        Vec2::ZERO
    };
    let tactical = cam.zoom_alpha > WEATHER_TACTICAL_PRECIP_ZOOM_ALPHA;
    let base_alpha = if cloud < 0.08 {
        0.0
    } else if tactical {
        (cloud * 0.22).min(0.28)
    } else {
        (cloud * 0.38).min(0.45)
    };
    let t = time.elapsed_secs();
    for (sheet, mut tf, mut vis) in &mut sheets {
        let i = sheet.index as f32;
        let drift = wind * (0.35 + i * 0.05) * t.sin().abs().max(0.15);
        tf.translation.x = (i - 2.5) * (if tactical { 70.0 } else { 110.0 }) + drift.x * 4.0
            + (t * (0.11 + i * 0.03) + i).sin() * 28.0;
        tf.translation.y = (i % 3.0 - 1.0) * (if tactical { 55.0 } else { 85.0 })
            + drift.y * 3.0
            + (t * (0.09 + i * 0.02)).cos() * 22.0;
        let scale = if tactical { 0.75 } else { 1.15 + cloud * 0.35 };
        tf.scale = Vec3::splat(scale);
        let alpha = base_alpha * (0.55 + 0.45 * ((i * 0.7 + t * 0.2).sin() * 0.5 + 0.5));
        *vis = if alpha < 0.02 {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
        if let Some(handle) = mats.handles.get(sheet.index as usize) {
            if let Some(mut mat) = materials.get_mut(handle) {
                mat.color = Color::srgba(0.82, 0.86, 0.92, alpha);
            }
        }
    }
}

pub fn register_cloud_sheets(app: &mut App) {
    app.add_systems(
        Update,
        (
            attach_cloud_sheets.after(AtmospherePipelineSet::VisualExtract),
            update_cloud_sheets
                .after(crate::systems::weather::publish_camera_local_weather_sample)
                .after(ExtractedCameraMetricsSet::Sync),
        ),
    );
}
