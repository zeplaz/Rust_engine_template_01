//! Bevy assembly preview — spawn snapshot placements, capture PNG.
//!
//! Blank-frame root cause (BQ-Q2): screenshot fired on `LoadState::Loaded` before
//! `WorldAssetRoot` finished instantiating meshes / GPU presented a lit frame.
//! Invisible windows on some DXGI paths also yield all-black captures. Fix: wait for
//! `Mesh3d`, settle N frames, capture from an off-screen but visible window, and
//! reject flat/black PNGs (one retry with extra settle) before writing `status: done`.

use std::path::{Path, PathBuf};
use std::time::Instant;

use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy::world_serialization::{WorldAsset, WorldAssetRoot};
use bevy::render::view::screenshot::{save_to_disk, Captured, Screenshot};
use bevy::window::{PresentMode, Window, WindowPlugin, WindowPosition};
use serde::Serialize;

use crate::construction::procedural::{AssemblyModulePlacement, AssemblySnapshot};

use super::job::PreviewAssemblyJob;

/// Frames after first `Mesh3d` appears before requesting a screenshot.
const SETTLE_FRAMES: u32 = 12;
/// Extra settle frames after a blank capture before one retry.
const RETRY_EXTRA_SETTLE: u32 = 16;
/// Min luma span (hi−lo) to accept a preview PNG (mirrors Python `png_preview_usable`).
const MIN_LUMA_SPAN: u8 = 24;
const MAX_CAPTURE_ATTEMPTS: u8 = 2;

#[derive(Resource)]
pub struct PreviewWorkerConfig {
    pub repo_root: PathBuf,
    pub job_path: PathBuf,
    pub job: PreviewAssemblyJob,
    pub started: Instant,
}

#[derive(Resource, Default)]
struct PreviewWorkerState {
    spawn_done: bool,
    screenshot_requested: bool,
    screenshot_attempts: u8,
    modules_loaded: u32,
    missing_glb: Vec<String>,
    meshes_seen: bool,
    settle_remaining: u32,
}

#[derive(Debug, Serialize)]
struct PreviewStatusDone {
    status: &'static str,
    png: String,
    elapsed_ms: u64,
    modules_loaded: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    missing_glb: Vec<String>,
    mode: &'static str,
}

#[derive(Debug, Serialize)]
struct PreviewStatusFailed {
    status: &'static str,
    error: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    missing_glb: Vec<String>,
    mode: &'static str,
}

#[must_use]
pub fn repo_root_from_manifest() -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn load_assembly_snapshot_json(path: &Path) -> Result<AssemblySnapshot, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("read snapshot: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("parse snapshot: {e}"))
}

fn placement_label(row: &AssemblyModulePlacement, index: usize) -> String {
    let label = format!(
        "{}@({},{},f{})",
        row.module_id, row.grid_x, row.grid_y, row.floor
    );
    if label.len() > 80 {
        format!("placement_{index}")
    } else {
        label
    }
}

fn write_status(path: &Path, body: &impl Serialize) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir status: {e}"))?;
    }
    let text = serde_json::to_string_pretty(body).map_err(|e| format!("serialize status: {e}"))?;
    std::fs::write(path, format!("{text}\n")).map_err(|e| format!("write status: {e}"))
}

fn write_failed(config: &PreviewWorkerConfig, error: String, missing: Vec<String>) {
    let status_path = config.job.status_path(&config.job_path);
    let _ = write_status(
        &status_path,
        &PreviewStatusFailed {
            status: "failed",
            error,
            missing_glb: missing,
            mode: "bevy_worker",
        },
    );
}

fn placement_center(placements: &[AssemblyModulePlacement]) -> Vec3 {
    if placements.is_empty() {
        return Vec3::ZERO;
    }
    let mut sum = Vec3::ZERO;
    let n = placements.len() as f32;
    for p in placements {
        sum += Vec3::new(p.position[0] as f32, p.position[1] as f32, p.position[2] as f32);
    }
    sum / n
}

fn iso_camera_transform(center: Vec3, distance_m: f32) -> Transform {
    let d = distance_m.max(8.0);
    // iso_se — +X/−Z so street/south openings read at city-sim distance (not hid behind N wall).
    Transform::from_translation(center + Vec3::new(d * 0.85, d * 0.65, -d * 0.85))
        .looking_at(center, Vec3::Y)
}

fn spawn_preview_scene(
    mut commands: Commands,
    config: Res<PreviewWorkerConfig>,
    asset_server: Res<AssetServer>,
    mut state: ResMut<PreviewWorkerState>,
) {
    if state.spawn_done {
        return;
    }

    let snap_path = config.job.snapshot_path(&config.repo_root);
    let snapshot = match load_assembly_snapshot_json(&snap_path) {
        Ok(s) => s,
        Err(e) => {
            write_failed(&config, e, Vec::new());
            commands.write_message(AppExit::error());
            return;
        }
    };

    commands.spawn((
        DirectionalLight {
            illuminance: 14_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, 0.4, 0.0)),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 4_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.8, 0.0)),
    ));
    commands.spawn(AmbientLight {
        color: Color::srgb(0.92, 0.94, 0.98),
        brightness: 120.0,
        ..default()
    });

    let center = placement_center(&snapshot.module_placements);
    commands.spawn((
        Camera3d::default(),
        iso_camera_transform(center, config.job.camera.distance_m),
    ));

    let mut scene_handles: Vec<Handle<WorldAsset>> = Vec::new();
    for (i, row) in snapshot.module_placements.iter().enumerate() {
        let rel = row.glb_path.replace('\\', "/");
        let glb = config.repo_root.join(&rel);
        if !glb.is_file() {
            state.missing_glb.push(placement_label(row, i));
            continue;
        }
        let asset_path = rel.strip_prefix("assets/").unwrap_or(&rel);
        let handle: Handle<WorldAsset> = asset_server.load(format!("{asset_path}#Scene0"));
        scene_handles.push(handle.clone());

        let pos = Vec3::new(
            row.position[0] as f32,
            row.position[1] as f32,
            row.position[2] as f32,
        );
        let rot = Quat::from_euler(
            EulerRot::XYZ,
            row.rotation_euler[0] as f32,
            row.rotation_euler[1] as f32,
            row.rotation_euler[2] as f32,
        );
        let mut xf = Transform::from_translation(pos).with_rotation(rot);
        if let Some(s) = row.scale {
            xf.scale = Vec3::new(s[0] as f32, s[1] as f32, s[2] as f32);
        }
        commands.spawn((
            WorldAssetRoot(handle),
            xf,
            Visibility::default(),
        ));
    }

    state.spawn_done = true;
    state.modules_loaded = scene_handles.len() as u32;
    commands.insert_resource(PreviewSceneHandles(scene_handles));
}

#[derive(Resource)]
struct PreviewSceneHandles(Vec<Handle<WorldAsset>>);

fn scenes_ready(asset_server: Res<AssetServer>, handles: Option<Res<PreviewSceneHandles>>) -> bool {
    let Some(handles) = handles else {
        return false;
    };
    !handles.0.is_empty()
        && handles.0.iter().all(|h| {
            matches!(
                asset_server.get_load_state(h.id()),
                Some(LoadState::Loaded) | Some(LoadState::Failed(_))
            )
        })
}

/// Reject missing / flat / near-black PNGs (same bar as Python `png_preview_usable`).
fn png_preview_usable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    if meta.len() < 256 {
        return false;
    }
    let Ok(img) = image::open(path) else {
        return false;
    };
    let gray = img.to_luma8();
    let mut lo = u8::MAX;
    let mut hi = 0u8;
    for p in gray.pixels() {
        let v = p.0[0];
        lo = lo.min(v);
        hi = hi.max(v);
        if hi.saturating_sub(lo) >= MIN_LUMA_SPAN {
            return true;
        }
    }
    hi.saturating_sub(lo) >= MIN_LUMA_SPAN
}

fn request_screenshot_when_ready(
    mut commands: Commands,
    config: Res<PreviewWorkerConfig>,
    handles: Option<Res<PreviewSceneHandles>>,
    asset_server: Res<AssetServer>,
    mut state: ResMut<PreviewWorkerState>,
    pending: Query<Entity, With<Screenshot>>,
    meshes: Query<(), With<Mesh3d>>,
) {
    if state.screenshot_requested || !state.spawn_done || !pending.is_empty() {
        return;
    }
    if !scenes_ready(asset_server, handles) {
        return;
    }
    // WorldAssetRoot spawns children after LoadState::Loaded — wait for meshes.
    if meshes.iter().next().is_none() {
        return;
    }
    if !state.meshes_seen {
        state.meshes_seen = true;
        state.settle_remaining = SETTLE_FRAMES;
    }
    if state.settle_remaining > 0 {
        state.settle_remaining = state.settle_remaining.saturating_sub(1);
        return;
    }

    let png_path = config.job.png_path(&config.repo_root);
    if let Some(parent) = png_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    // Drop stale blank from a prior attempt so capture writes a fresh file.
    let _ = std::fs::remove_file(&png_path);

    state.screenshot_requested = true;
    state.screenshot_attempts = state.screenshot_attempts.saturating_add(1);
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(png_path));
}

fn on_screenshot_captured(
    mut commands: Commands,
    config: Res<PreviewWorkerConfig>,
    mut state: ResMut<PreviewWorkerState>,
    captured: Query<Entity, Added<Captured>>,
    mut exit: MessageWriter<AppExit>,
) {
    let Ok(entity) = captured.single() else {
        return;
    };

    let png_path = config.job.png_path(&config.repo_root);
    // save_to_disk observer usually wins the race; poll briefly if needed.
    let deadline = Instant::now() + std::time::Duration::from_millis(1500);
    while Instant::now() < deadline {
        if png_preview_usable(&png_path) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(40));
    }

    if !png_preview_usable(&png_path) {
        commands.entity(entity).despawn();
        if state.screenshot_attempts < MAX_CAPTURE_ATTEMPTS {
            // Blank first frame — settle longer and retry once.
            state.screenshot_requested = false;
            state.settle_remaining = RETRY_EXTRA_SETTLE;
            eprintln!(
                "bevy_preview_worker: blank PNG attempt {} — retrying after {} settle frames",
                state.screenshot_attempts, RETRY_EXTRA_SETTLE
            );
            return;
        }
        write_failed(
            &config,
            "preview PNG blank or unusable after settle+retry".into(),
            state.missing_glb.clone(),
        );
        exit.write(AppExit::error());
        return;
    }

    let rel_png = png_path
        .strip_prefix(&config.repo_root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| png_path.to_string_lossy().into_owned());

    let status_path = config.job.status_path(&config.job_path);
    let body = PreviewStatusDone {
        status: "done",
        png: rel_png,
        elapsed_ms: config.started.elapsed().as_millis() as u64,
        modules_loaded: state.modules_loaded,
        missing_glb: state.missing_glb.clone(),
        mode: "bevy_worker",
    };
    if write_status(&status_path, &body).is_err() {
        exit.write(AppExit::error());
        return;
    }
    commands.entity(entity).despawn();
    exit.write(AppExit::Success);
}

fn watchdog_no_assets(
    config: Res<PreviewWorkerConfig>,
    state: Res<PreviewWorkerState>,
    handles: Option<Res<PreviewSceneHandles>>,
    mut exit: MessageWriter<AppExit>,
) {
    if !state.spawn_done {
        return;
    }
    if state.modules_loaded > 0 {
        return;
    }
    if handles.is_some() {
        write_failed(
            &config,
            "no resolvable GLB placements".into(),
            state.missing_glb.clone(),
        );
        exit.write(AppExit::error());
    }
}

fn watchdog_timeout(
    config: Res<PreviewWorkerConfig>,
    state: Res<PreviewWorkerState>,
    mut exit: MessageWriter<AppExit>,
) {
    if config.started.elapsed().as_secs() <= 90 {
        return;
    }
    let reason = if state.modules_loaded > 0 && !state.meshes_seen {
        "preview worker timeout (90s) — assets loaded but no Mesh3d spawned"
    } else if state.screenshot_attempts > 0 {
        "preview worker timeout (90s) — capture/validate stalled"
    } else {
        "preview worker timeout (90s)"
    };
    write_failed(&config, reason.into(), state.missing_glb.clone());
    exit.write(AppExit::error());
}

/// Run one preview job; returns process exit code.
pub fn run_preview_job(job_path: &Path) -> i32 {
    let repo_root = repo_root_from_manifest();
    let job_path = if job_path.is_absolute() {
        job_path.to_path_buf()
    } else {
        repo_root.join(job_path)
    };

    let job = match PreviewAssemblyJob::load(&job_path, &repo_root) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("bevy_preview_worker: {e}");
            return 1;
        }
    };

    let width = job.output.width.max(64);
    let height = job.output.height.max(64);

    // Ensure AssetServer resolves repo `assets/` even when cwd drifts.
    if std::env::var_os("BEVY_ASSET_ROOT").is_none() {
        // SAFETY: single-threaded before App::run; set once for this process.
        unsafe {
            std::env::set_var("BEVY_ASSET_ROOT", &repo_root);
        }
    }

    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.18, 0.20, 0.24)))
        .insert_resource(PreviewWorkerConfig {
            repo_root: repo_root.clone(),
            job_path: job_path.clone(),
            job: job.clone(),
            started: Instant::now(),
        })
        .insert_resource(PreviewWorkerState::default())
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "bevy_preview_worker".into(),
                        resolution: (width, height).into(),
                        present_mode: PresentMode::AutoNoVsync,
                        // Invisible windows often capture all-black on DXGI; keep
                        // visible but parked off-screen so the GPU presents a frame.
                        visible: true,
                        position: WindowPosition::new(IVec2::new(-12_000, -12_000)),
                        ..default()
                    }),
                    ..default()
                })
                .build(),
        )
        .add_systems(Startup, spawn_preview_scene)
        .add_systems(Update, request_screenshot_when_ready)
        .add_systems(Update, on_screenshot_captured.after(request_screenshot_when_ready))
        .add_systems(Update, (watchdog_no_assets, watchdog_timeout));

    let exit = app.run();
    if exit.is_success() {
        0
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_job_roundtrip_warehouse_example() {
        let root = repo_root_from_manifest();
        let snap = root.join(
            "tools/mcp/schemas/examples/assembly_snapshot_warehouse_industrial_west_production_v1.json",
        );
        if !snap.is_file() {
            return;
        }
        let job_path = root.join("debug_runs/preview_jobs/test_worker_job.json");
        let body = serde_json::json!({
            "schema_version": 1,
            "operation": "preview_assembly",
            "job_id": "test_worker_job",
            "assembly_snapshot": snap.strip_prefix(&root).unwrap().to_string_lossy(),
            "camera": { "preset": "iso_se", "distance_m": 24.0 },
            "output": {
                "png": "debug_runs/preview_jobs/test_worker_job.png",
                "width": 256,
                "height": 256
            }
        });
        if let Some(parent) = job_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&job_path, serde_json::to_string_pretty(&body).unwrap()).unwrap();
        let job = PreviewAssemblyJob::load(&job_path, &root).expect("job load");
        assert_eq!(job.operation, "preview_assembly");
    }
}
