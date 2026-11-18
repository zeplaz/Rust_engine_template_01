//! Immutable-per-frame audio mix plan (UI → ambient → spatial → combat → warning).

use bevy::prelude::*;

use crate::presentation::TruckPresentation;
use crate::render::view_runtime::{ViewProjectionAuthority, ViewSurfaceId};
use crate::systems::sim_attention::SimAttentionFrame;
use crate::systems::weather::{CameraLocalWeatherSample, ChunkWeather};

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

/// Max concurrent combat one-shots per mix frame (instance limit, AUD-02).
pub const COMBAT_ONESHOT_CAP: usize = 4;

/// Linear gain when ambience is ducked under warnings (−6 dB ≈ 0.501).
pub const AMBIENCE_DUCK_TARGET_GAIN: f32 = 0.501_187_2;

/// Hold ambience duck after a warning one-shot leaves the pending queue (CQ-10).
pub const WARNING_DUCK_HOLD_SECS: f32 = 1.25;

/// Smoothed ambience-bus duck state — sole writer: [`build_audio_mix_frame`].
#[derive(Resource, Clone, Debug)]
pub struct AudioMixDuckState {
    pub ambience_bus_gain: f32,
    /// Remaining real-time seconds to keep ducking after the last warning frame.
    pub warning_hold_secs: f32,
}

impl Default for AudioMixDuckState {
    fn default() -> Self {
        Self {
            ambience_bus_gain: 1.0,
            warning_hold_secs: 0.0,
        }
    }
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
    /// Smoothed linear gain applied to ambient bus requests this frame.
    pub ambience_duck_gain: f32,
    /// AUD-05 — low-pass corner for ambience / spatial bed (Hz); rain+fog pull this down.
    pub weather_lowpass_cutoff_hz: f32,
    /// AUD-05 — extra linear attenuation from wet air (1.0 = none).
    pub weather_attenuation_gain: f32,
    /// Camera-local fog density [0,1] used for this frame’s mix.
    pub ambient_fog: f32,
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

fn combat_listener_distance(listener: Option<Vec2>, world_xz: Option<Vec2>) -> f32 {
    match (listener, world_xz) {
        (Some(l), Some(w)) => l.distance(w),
        _ => f32::MAX,
    }
}

/// Keep up to `cap` combat one-shots: nearest to listener, then newest queue index.
pub fn cap_combat_oneshots_in_place(
    pending: &mut Vec<AudioMixRequest>,
    listener_xz: Option<Vec2>,
    cap: usize,
) {
    if pending.is_empty() {
        return;
    }
    if cap == 0 {
        pending.retain(|r| r.class != AudioEventClass::CombatOneShot);
        return;
    }
    if !pending.iter().any(|r| r.class == AudioEventClass::CombatOneShot) {
        return;
    }
    let mut combat: Vec<(usize, AudioMixRequest)> = Vec::new();
    let mut other: Vec<AudioMixRequest> = Vec::with_capacity(pending.len());
    for (idx, req) in pending.drain(..).enumerate() {
        if req.class == AudioEventClass::CombatOneShot {
            combat.push((idx, req));
        } else {
            other.push(req);
        }
    }
    if combat.len() <= cap {
        other.extend(combat.into_iter().map(|(_, r)| r));
        *pending = other;
        return;
    }
    combat.sort_by(|(ia, a), (ib, b)| {
        let da = combat_listener_distance(listener_xz, a.world_xz);
        let db = combat_listener_distance(listener_xz, b.world_xz);
        da.partial_cmp(&db)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| ib.cmp(ia))
    });
    combat.truncate(cap);
    other.extend(combat.into_iter().map(|(_, r)| r));
    *pending = other;
}

const DUCK_ATTACK_SECS: f32 = 0.08;
const DUCK_RELEASE_SECS: f32 = 0.45;

/// Advance smoothed ambience duck gain toward target using real-time seconds.
pub fn step_ambience_duck_gain(current: f32, warnings_active: bool, dt_secs: f32) -> f32 {
    let target = if warnings_active {
        AMBIENCE_DUCK_TARGET_GAIN
    } else {
        1.0
    };
    if dt_secs <= 0.0 {
        return current;
    }
    let tau = if target < current {
        DUCK_ATTACK_SECS
    } else {
        DUCK_RELEASE_SECS
    };
    let alpha = 1.0 - (-dt_secs / tau).exp();
    current + (target - current) * alpha
}

fn apply_ambience_duck(requests: &mut [AudioMixRequest], duck_gain: f32) {
    let g = duck_gain.clamp(0.0, 1.0);
    for req in requests.iter_mut() {
        if req.class == AudioEventClass::Ambient {
            req.gain = (req.gain * g).clamp(0.0, 1.0);
        }
    }
}

/// Dry air → full bandwidth; heavier rain and fog reduce cutoff and add attenuation (AUD-05).
#[must_use]
pub fn weather_audio_lowpass_hz(rain01: f32, fog: f32) -> f32 {
    const HI: f32 = 12_000.0;
    const LO: f32 = 650.0;
    let wet = (rain01 * 0.82 + fog * 0.58).clamp(0.0, 1.0);
    HI + (LO - HI) * wet
}

#[must_use]
pub fn weather_audio_attenuation_gain(rain01: f32, fog: f32) -> f32 {
    let wet_db = (rain01 * 5.0 + fog * 3.5).clamp(0.0, 14.0);
    10f32.powf(-wet_db / 20.0)
}

fn apply_weather_absorption(requests: &mut [AudioMixRequest], atten: f32) {
    let g = atten.clamp(0.05, 1.0);
    for req in requests.iter_mut() {
        match req.class {
            AudioEventClass::Ambient | AudioEventClass::SpatialContinuum => {
                req.gain = (req.gain * g).clamp(0.0, 1.0);
            }
            _ => {}
        }
    }
}

pub fn build_audio_mix_frame(
    mut mix: ResMut<AudioMixFrame>,
    mut oneshots: ResMut<AudioOneShotQueue>,
    mut duck: ResMut<AudioMixDuckState>,
    time: Res<Time>,
    attention: Option<Res<SimAttentionFrame>>,
    view_auth: Option<Res<ViewProjectionAuthority>>,
    camera_wx: Option<Res<CameraLocalWeatherSample>>,
    weather: Query<&ChunkWeather>,
    trucks: Query<&Transform, With<TruckPresentation>>,
) {
    mix.requests.clear();
    mix.listener_xz = None;
    mix.listener_zoom = 1.0;
    mix.ambient_rain = 0.0;
    mix.ambient_wind = 0.0;
    mix.ambient_fog = 0.0;
    mix.weather_lowpass_cutoff_hz = weather_audio_lowpass_hz(0.0, 0.0);
    mix.weather_attenuation_gain = 1.0;
    mix.ambience_duck_gain = duck.ambience_bus_gain;
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

    let (rain01, fog01) = if let Some(local) = camera_wx.as_ref().filter(|w| w.ready) {
        mix.ambient_rain = local.rain01;
        mix.ambient_fog = local.fog;
        mix.ambient_wind = local.sample.wind_mps.length();
        (local.rain01, local.fog)
    } else {
        let mut rain = 0.0_f32;
        let mut wind = 0.0_f32;
        let mut fog = 0.0_f32;
        let mut n = 0_u32;
        for wx in &weather {
            rain += wx.rain_intensity;
            wind += wx.wind_speed;
            fog += wx.fog_density;
            n += 1;
        }
        if n > 0 {
            mix.ambient_rain = rain / n as f32;
            mix.ambient_wind = wind / n as f32;
            mix.ambient_fog = fog / n as f32;
        }
        (mix.ambient_rain, mix.ambient_fog)
    };
    mix.weather_lowpass_cutoff_hz = weather_audio_lowpass_hz(rain01, fog01);
    mix.weather_attenuation_gain = weather_audio_attenuation_gain(rain01, fog01);

    // Ambient bed (intensity from camera-local sample when ready, else chunk mean).
    let ambient_rain = mix.ambient_rain;
    let ambient_wind = mix.ambient_wind;
    if ambient_rain > 0.0 || ambient_wind > 0.2 {
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
                // Only rumble when a near truck exists — phantom listener bed was a constant beep.
                if let Some((d, p)) = nearest {
                    let zoom_g = ((zoom - 0.35) / 1.5).clamp(0.0, 1.0);
                    let dist_g = (1.0 - d / 120.0).clamp(0.0, 1.0);
                    mix.requests.push(AudioMixRequest {
                        class: AudioEventClass::SpatialContinuum,
                        kind: None,
                        world_xz: Some(p),
                        gain: (zoom_g * dist_g * 0.55).clamp(0.0, 1.0),
                        tag: "spatial.near_field",
                    });
                }
            }
        }
    }

    let weather_gain = mix.weather_attenuation_gain;
    apply_weather_absorption(&mut mix.requests, weather_gain);

    cap_combat_oneshots_in_place(&mut oneshots.pending, mix.listener_xz, COMBAT_ONESHOT_CAP);

    let warnings_in_pending = oneshots
        .pending
        .iter()
        .any(|r| r.class == AudioEventClass::Warning);
    let dt = time.delta_secs();
    // CQ-10: pending is drained each frame — hold duck for WARNING_DUCK_HOLD_SECS after fire.
    if warnings_in_pending {
        duck.warning_hold_secs = WARNING_DUCK_HOLD_SECS;
    } else if duck.warning_hold_secs > 0.0 {
        duck.warning_hold_secs = (duck.warning_hold_secs - dt).max(0.0);
    }
    let warnings_active = warnings_in_pending || duck.warning_hold_secs > 0.0;
    duck.ambience_bus_gain =
        step_ambience_duck_gain(duck.ambience_bus_gain, warnings_active, dt);
    mix.ambience_duck_gain = duck.ambience_bus_gain;
    apply_ambience_duck(&mut mix.requests, duck.ambience_bus_gain);

    mix.requests.append(&mut oneshots.pending);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn combat_at(x: f32, z: f32) -> AudioMixRequest {
        AudioMixRequest {
            class: AudioEventClass::CombatOneShot,
            kind: Some(AudioOneShotKind::Shot),
            world_xz: Some(Vec2::new(x, z)),
            gain: 1.0,
            tag: "combat.shot",
        }
    }

    #[test]
    fn combat_cap_keeps_nearest_to_listener() {
        let listener = Vec2::ZERO;
        let mut pending = vec![
            combat_at(100.0, 0.0),
            combat_at(10.0, 0.0),
            combat_at(50.0, 0.0),
            combat_at(5.0, 0.0),
        ];
        cap_combat_oneshots_in_place(&mut pending, Some(listener), 2);
        let combat: Vec<_> = pending
            .iter()
            .filter(|r| r.class == AudioEventClass::CombatOneShot)
            .collect();
        assert_eq!(combat.len(), 2);
        let mut dists: Vec<f32> = combat
            .iter()
            .map(|r| r.world_xz.unwrap().distance(listener))
            .collect();
        dists.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert!((dists[0] - 5.0).abs() < 1e-3);
        assert!((dists[1] - 10.0).abs() < 1e-3);
    }

    #[test]
    fn combat_cap_prefers_newer_at_equal_distance() {
        let listener = Vec2::ZERO;
        let mut pending = vec![
            AudioMixRequest {
                class: AudioEventClass::CombatOneShot,
                kind: Some(AudioOneShotKind::Shot),
                world_xz: Some(Vec2::new(20.0, 0.0)),
                gain: 1.0,
                tag: "combat.old",
            },
            AudioMixRequest {
                class: AudioEventClass::CombatOneShot,
                kind: Some(AudioOneShotKind::Shot),
                world_xz: Some(Vec2::new(20.0, 0.0)),
                gain: 1.0,
                tag: "combat.mid",
            },
            AudioMixRequest {
                class: AudioEventClass::CombatOneShot,
                kind: Some(AudioOneShotKind::Shot),
                world_xz: Some(Vec2::new(20.0, 0.0)),
                gain: 1.0,
                tag: "combat.new",
            },
        ];
        cap_combat_oneshots_in_place(&mut pending, Some(listener), 2);
        let tags: Vec<_> = pending
            .iter()
            .filter(|r| r.class == AudioEventClass::CombatOneShot)
            .map(|r| r.tag)
            .collect();
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"combat.mid"));
        assert!(tags.contains(&"combat.new"));
    }

    #[test]
    fn ambience_duck_reaches_target_under_warning() {
        let mut g = 1.0;
        for _ in 0..200 {
            g = step_ambience_duck_gain(g, true, 1.0 / 60.0);
        }
        assert!(
            (g - AMBIENCE_DUCK_TARGET_GAIN).abs() < 0.02,
            "g={g} target={}",
            AMBIENCE_DUCK_TARGET_GAIN
        );
    }

    #[test]
    fn heavier_rain_lowers_weather_lowpass_cutoff() {
        let dry = weather_audio_lowpass_hz(0.0, 0.0);
        let wet = weather_audio_lowpass_hz(1.0, 0.0);
        assert!(wet < dry, "dry={dry} wet={wet}");
        let foggy = weather_audio_lowpass_hz(0.2, 1.0);
        assert!(foggy < dry);
    }

    #[test]
    fn ambience_duck_releases_after_warning_clears() {
        let mut g = AMBIENCE_DUCK_TARGET_GAIN;
        for _ in 0..400 {
            g = step_ambience_duck_gain(g, false, 1.0 / 60.0);
        }
        assert!((g - 1.0).abs() < 0.02, "g={g}");
    }
}
