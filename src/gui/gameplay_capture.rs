//! Full-window gameplay capture: PNG sequence while recording, animated GIF on stop.
//!
//! Default hotkeys: **F11** start, **F12** stop (rebind in Options). Output goes under the user
//! captures directory (see [`GameplayRecorder::default_captures_root`]).

use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::window::PrimaryWindow;
use crossbeam_channel::{unbounded, Receiver, Sender};

use super::input_bindings::InputBindings;
use crate::gui::options_keybindings_ui::KeybindingsUiState;
use crate::gui::ui_gates::in_simulation_or_editor;
use image::codecs::gif::{GifEncoder, Repeat};
use image::{Delay, Frame as AnimationFrame};

/// Rebindable limits; serialized key bindings live in `InputBindings`, not here.
#[derive(Resource, Debug, Clone)]
pub struct GameplayRecorder {
    pub active: bool,
    /// While active, PNGs are written as `frame_00000.png`, …
    pub session_dir: Option<PathBuf>,
    pub frames_recorded: u32,
    /// Capture one frame every `frame_stride` updates (1 = as fast as the GPU screenshot pipeline allows).
    pub frame_stride: u32,
    pub max_frames: u32,
    /// Delay written into the GIF for each frame (milliseconds).
    pub gif_frame_delay_ms: u32,
    stride_counter: u32,
    last_session_summary: Option<String>,
}

impl Default for GameplayRecorder {
    fn default() -> Self {
        Self {
            active: false,
            session_dir: None,
            frames_recorded: 0,
            frame_stride: 1,
            max_frames: 3_600,
            gif_frame_delay_ms: 33,
            stride_counter: 0,
            last_session_summary: None,
        }
    }
}

impl GameplayRecorder {
    #[must_use]
    pub fn default_captures_root() -> PathBuf {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata).join("proc_A_dine01").join("captures");
        }
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home)
                .join(".local/share/proc_A_dine01/captures");
        }
        PathBuf::from("user_settings/captures")
    }

    #[must_use]
    pub fn last_summary(&self) -> Option<&str> {
        self.last_session_summary.as_deref()
    }
}

#[derive(Resource)]
struct GameplayExportTx(Sender<ExportOutcome>);

#[derive(Resource)]
struct GameplayExportRx(Receiver<ExportOutcome>);

enum ExportOutcome {
    Ok {
        session: PathBuf,
        gif: PathBuf,
        frames: usize,
    },
    Err {
        session: PathBuf,
        message: String,
    },
}

pub struct GameplayCapturePlugin;

impl Plugin for GameplayCapturePlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = unbounded();
        app.init_resource::<GameplayRecorder>()
            .insert_resource(GameplayExportTx(tx))
            .insert_resource(GameplayExportRx(rx))
            .add_systems(
                Update,
                (
                    gameplay_capture_hotkeys,
                    gameplay_capture_grab_frame,
                    gameplay_export_poll,
                )
                    .chain()
                    .run_if(in_simulation_or_editor),
            );
    }
}

fn gameplay_capture_hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    kb: Res<KeybindingsUiState>,
    mut recorder: ResMut<GameplayRecorder>,
    export_tx: Res<GameplayExportTx>,
) {
    if kb.capture_slot.is_some() {
        return;
    }

    if keys.just_pressed(bindings.start_gameplay_recording) {
        if recorder.active {
            return;
        }
        let root = GameplayRecorder::default_captures_root();
        if let Err(e) = fs::create_dir_all(&root) {
            recorder.last_session_summary =
                Some(format!("Capture: could not create {} — {e}", root.display()));
            return;
        }
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let session = root.join(format!("session_{stamp}"));
        if let Err(e) = fs::create_dir_all(&session) {
            recorder.last_session_summary =
                Some(format!("Capture: could not create session dir — {e}"));
            return;
        }
        recorder.session_dir = Some(session);
        recorder.frames_recorded = 0;
        recorder.stride_counter = 0;
        recorder.active = true;
        recorder.last_session_summary = None;
        info!(
            "Gameplay capture: recording → {} ({} stops)",
            recorder.session_dir.as_ref().unwrap().display(),
            InputBindings::format_key(bindings.stop_gameplay_recording)
        );
    }

    if keys.just_pressed(bindings.stop_gameplay_recording) {
        if !recorder.active {
            return;
        }
        recorder.active = false;
        let session_dir = recorder.session_dir.take().unwrap_or_else(|| PathBuf::from("."));
        let session_log = session_dir.display().to_string();
        let delay_ms = recorder.gif_frame_delay_ms;
        let tx = export_tx.0.clone();
        thread::spawn(move || {
            let outcome = finalize_session_gif(&session_dir, delay_ms);
            let _ = tx.send(outcome);
        });
        info!(
            "Gameplay capture: stopped; building GIF for {}",
            session_log
        );
    }
}

fn gameplay_capture_grab_frame(
    mut commands: Commands,
    mut recorder: ResMut<GameplayRecorder>,
    pending: Query<Entity, With<Screenshot>>,
    primary: Query<Entity, With<PrimaryWindow>>,
) {
    if !recorder.active {
        return;
    }
    if primary.iter().next().is_none() {
        return;
    }
    if !pending.is_empty() {
        return;
    }
    if recorder.frames_recorded >= recorder.max_frames {
        return;
    }

    recorder.stride_counter = recorder.stride_counter.wrapping_add(1);
    if recorder.frame_stride > 1 && recorder.stride_counter % recorder.frame_stride != 0 {
        return;
    }

    let Some(dir) = recorder.session_dir.clone() else {
        return;
    };
    let idx = recorder.frames_recorded;
    let path = dir.join(format!("frame_{idx:05}.png"));
    recorder.frames_recorded = recorder.frames_recorded.saturating_add(1);

    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
}

fn gameplay_export_poll(
    rx: Res<GameplayExportRx>,
    mut recorder: ResMut<GameplayRecorder>,
) {
    while let Ok(msg) = rx.0.try_recv() {
        match msg {
            ExportOutcome::Ok {
                session,
                gif,
                frames,
            } => {
                recorder.last_session_summary = Some(format!(
                    "Gameplay capture: {frames} frames → GIF {}",
                    gif.display()
                ));
                info!(
                    "Gameplay capture: wrote {} ({} PNGs in {})",
                    gif.display(),
                    frames,
                    session.display()
                );
            }
            ExportOutcome::Err { session, message } => {
                recorder.last_session_summary = Some(format!(
                    "Gameplay capture failed ({}): {message}",
                    session.display()
                ));
                warn!("Gameplay capture: {message}");
            }
        }
    }
}

fn finalize_session_gif(session_dir: &Path, frame_delay_ms: u32) -> ExportOutcome {
    let rd = match fs::read_dir(session_dir) {
        Ok(r) => r,
        Err(e) => {
            return ExportOutcome::Err {
                session: session_dir.to_path_buf(),
                message: format!("read_dir: {e}"),
            };
        }
    };
    let mut pngs: Vec<PathBuf> = Vec::new();
    for ent in rd.flatten() {
        let p = ent.path();
        if p.extension().and_then(|x| x.to_str()) == Some("png")
            && p
                .file_stem()
                .is_some_and(|s| s.to_str().is_some_and(|t| t.starts_with("frame_")))
        {
            pngs.push(p);
        }
    }

    pngs.sort();

    if pngs.is_empty() {
        return ExportOutcome::Err {
            session: session_dir.to_path_buf(),
            message: "no frame_*.png in session (record longer or check GPU capture)".into(),
        };
    }

    let gif_path = session_dir.join("clip.gif");
    let delay = Delay::from_numer_denom_ms(frame_delay_ms.max(1), 1);

    let mut file = match fs::File::create(&gif_path) {
        Ok(f) => f,
        Err(e) => {
            return ExportOutcome::Err {
                session: session_dir.to_path_buf(),
                message: format!("create gif: {e}"),
            };
        }
    };

    let mut encoder = GifEncoder::new(&mut file);

    if let Err(e) = encoder.set_repeat(Repeat::Infinite) {
        return ExportOutcome::Err {
            session: session_dir.to_path_buf(),
            message: format!("gif repeat: {e}"),
        };
    }

    let frame_count = pngs.len();
    for path in &pngs {
        let img = match image::open(path) {
            Ok(i) => i.into_rgba8(),
            Err(e) => {
                return ExportOutcome::Err {
                    session: session_dir.to_path_buf(),
                    message: format!("open {}: {e}", path.display()),
                };
            }
        };
        let frame = AnimationFrame::from_parts(img, 0, 0, delay);
        if let Err(e) = encoder.encode_frame(frame) {
            return ExportOutcome::Err {
                session: session_dir.to_path_buf(),
                message: format!("encode frame: {e}"),
            };
        }
    }

    drop(encoder);
    drop(file);

    let _ = write_session_meta(session_dir, frame_count, frame_delay_ms, &gif_path);

    ExportOutcome::Ok {
        session: session_dir.to_path_buf(),
        gif: gif_path,
        frames: frame_count,
    }
}

fn write_session_meta(session: &Path, frames: usize, delay_ms: u32, gif: &Path) -> std::io::Result<()> {
    let meta = session.join("session.txt");
    let text = format!(
        "frames={frames}\ngif_frame_delay_ms={delay_ms}\npng_glob=frame_*.png\ngif={}\n",
        gif.file_name().and_then(|n| n.to_str()).unwrap_or("clip.gif")
    );
    fs::write(meta, text)
}
