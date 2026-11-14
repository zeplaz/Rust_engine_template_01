//! Immutable-per-frame audio mix plan (UI → ambient → spatial → combat → warning).

use bevy::prelude::*;

use crate::gui::ViewAuthoritySystemSet;
use crate::presentation::TruckPresentation;
use crate::render::view_runtime::{ViewProjectionAuthority, ViewSurfaceId};
use crate::systems::sim_attention::SimAttentionFrame;
use crate::systems::weather::ChunkWeather;

/// SystemSet for the sole [`AudioMixFrame`] writer.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AudioMixSet {
    Build,
}

/// High-level event classes (gea_v2 audio catalog mapping).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AudioEventClass {
    Ui,
    Ambient,
    SpatialContinuum,
    CombatOneShot,
    Warning,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AudioOneShotKind {
    UiClick,
    UiConfirm,
    UiDeny,
    Shot,
    Explosion,
    MissileLaunch,
    LaunchWarning,
    WeatherAlert,
}

/// One requested voice / one-shot for this frame (playback wiring comes later).
#[derive(Clone, Debug)]
pub struct AudioMixRequest {
    pub class: AudioEventClass,
    pub kind: Option<AudioOneShotKind>,
    /// World XZ for spatial continuum / one-shots; `None` = non-spatial (UI / bed).
    pub world_xz: Option<Vec2>,
    pub gain: f32,
    pub tag: &'static str,
}

/// Sole-writer mix plan for the frame — consumers (playback) read only.
#[derive(Resource, Clone, Debug, Default)]
pub struct AudioMixFrame {
    pub requests: Vec<AudioMixRequest>,
    pub listener_xz: Option<Vec2>,
    pub listener_zoom: f32,
    pub ambient_rain: f32,
    pub ambient_wind: f32,
    pub spatial_enabled: bool,
}

/// Drain UI / combat one-shot intents without touching sim resources.
#[derive(Resource, Default, Debug)]
pub struct AudioOneShotQueue {
    pub pending: Vec<AudioMixRequest>,
}

impl AudioOneShotQueue {
    pub fn push_ui(&mut self, kind: AudioOneShotKind) {
        let tag = super::load_sfx::tag_for_oneshot(kind);
        self.pending.push(AudioMixRequest {
            class: AudioEventClass::Ui,
            kind: Some(kind),
            world_xz: None,
            gain: 1.0,
            tag,
        });
    }

    pub fn push_warning(&mut self, kind: AudioOneShotKind, world_xz: Option<Vec2>) {
        let tag = super::load_sfx::tag_for_oneshot(kind);
        self.pending.push(AudioMixRequest {
            class: AudioEventClass::Warning,
            kind: Some(kind),
            world_xz,
            gain: 1.0,
            tag,
        });
    }

    pub fn push_combat(&mut self, kind: AudioOneShotKind, world_xz: Vec2) {
        let tag = super::load_sfx::tag_for_oneshot(kind);
        self.pending.push(AudioMixRequest {
            class: AudioEventClass::CombatOneShot,
            kind: Some(kind),
            world_xz: Some(world_xz),
            gain: 1.0,
            tag,
        });
    }
}

pub fn build_audio_mix_frame(
    mut mix: ResMut<AudioMixFrame>,
    mut oneshots: ResMut<AudioOneShotQueue>,
    attention: Option<Res<SimAttentionFrame>>,
    view_auth: Option<Res<ViewProjectionAuthority>>,
    weather: Query<&ChunkWeather>,
    trucks: Query<&Transform, With<TruckPresentation>>,
) {
    mix.requests.clear();
    mix.listener_xz = None;
    mix.listener_zoom = 1.0;
    mix.ambient_rain = 0.0;
    mix.ambient_wind = 0.0;
    mix.spatial_enabled = attention
        .as_ref()
        .map(|a| a.player_theater_active)
        .unwrap_or(false);

    if let Some(auth) = view_auth.as_ref() {
        if let Some(surface) = auth.surface(ViewSurfaceId::WorldMain) {
            mix.listener_xz = Some(surface.camera.translation);
            mix.listener_zoom = surface.camera.zoom.max(0.06);
        }
    }

    // Ambient bed from mean weather (cheap aggregate — no per-chunk spatial yet).
    let mut rain = 0.0_f32;
    let mut wind = 0.0_f32;
    let mut n = 0_u32;
    for wx in &weather {
        rain += wx.rain_intensity;
        wind += wx.wind_speed;
        n += 1;
    }
    if n > 0 {
        let ambient_rain = rain / n as f32;
        let ambient_wind = wind / n as f32;
        mix.ambient_rain = ambient_rain;
        mix.ambient_wind = ambient_wind;
        if ambient_rain > 0.05 {
            mix.requests.push(AudioMixRequest {
                class: AudioEventClass::Ambient,
                kind: None,
                world_xz: None,
                gain: ambient_rain.clamp(0.0, 1.0),
                tag: "ambient.rain",
            });
        }
        if ambient_wind > 0.2 {
            mix.requests.push(AudioMixRequest {
                class: AudioEventClass::Ambient,
                kind: None,
                world_xz: None,
                gain: (ambient_wind * 0.5).clamp(0.0, 1.0),
                tag: "ambient.wind",
            });
        }
    }

    // Zoom-in spatial continuum — nearest truck rumble when player theater + zoomed.
    let spatial_enabled = mix.spatial_enabled;
    let listener_xz = mix.listener_xz;
    let zoom = mix.listener_zoom;
    if spatial_enabled {
        if let Some(listener) = listener_xz {
            if zoom > 0.35 {
                let mut nearest: Option<(f32, Vec2)> = None;
                for tf in &trucks {
                    let p = tf.translation.truncate();
                    let d = p.distance(listener);
                    if d < 120.0 && nearest.is_none_or(|(bd, _)| d < bd) {
                        nearest = Some((d, p));
                    }
                }
                if let Some((d, p)) = nearest {
                    let zoom_g = ((zoom - 0.35) / 1.5).clamp(0.0, 1.0);
                    let dist_g = (1.0 - d / 120.0).clamp(0.0, 1.0);
                    mix.requests.push(AudioMixRequest {
                        class: AudioEventClass::SpatialContinuum,
                        kind: None,
                        world_xz: Some(p),
                        gain: (zoom_g * dist_g).clamp(0.0, 1.0),
                        tag: "spatial.near_field",
                    });
                } else {
                    mix.requests.push(AudioMixRequest {
                        class: AudioEventClass::SpatialContinuum,
                        kind: None,
                        world_xz: Some(listener),
                        gain: ((zoom - 0.35) / 1.5).clamp(0.0, 1.0) * 0.35,
                        tag: "spatial.near_field",
                    });
                }
            }
        }
    }

    mix.requests.append(&mut oneshots.pending);
}
