//! Startup: load placeholder WAVs from `assets/sfx/` into [`SfxCatalog`].

use bevy::prelude::*;

use super::mix_frame::{AudioEventClass, AudioOneShotKind, AudioOneShotQueue};
use super::sfx_catalog::{tags, SfxCatalog};

/// Tag → asset path under the Bevy asset root.
const SFX_FILES: &[(&str, &str)] = &[
    (tags::AMBIENT_RAIN, "sfx/ambient_rain.wav"),
    (tags::AMBIENT_WIND, "sfx/ambient_wind.wav"),
    (tags::SPATIAL_NEAR, "sfx/spatial_near.wav"),
    (tags::UI_CLICK_V0, "sfx/ui_click.wav"),
    (tags::UI_CLICK_V1, "sfx/ui_click.wav"),
    (tags::UI_CONFIRM, "sfx/ui_confirm.wav"),
    (tags::UI_DENY, "sfx/ui_deny.wav"),
    (tags::SHOT_V0, "sfx/combat_shot.wav"),
    (tags::SHOT_V1, "sfx/combat_shot.wav"),
    (tags::SHOT_V2, "sfx/combat_shot.wav"),
    (tags::EXPLOSION_V0, "sfx/combat_explosion.wav"),
    (tags::EXPLOSION_V1, "sfx/combat_explosion.wav"),
    (tags::MISSILE_LAUNCH, "sfx/combat_missile_launch.wav"),
    (tags::LAUNCH_WARNING, "sfx/warning_launch.wav"),
    (tags::WEATHER_ALERT, "sfx/warning_weather.wav"),
];

const SFX_CUES: &[(&str, &[&str])] = &[
    (tags::UI_CLICK, &[tags::UI_CLICK_V0, tags::UI_CLICK_V1]),
    (tags::SHOT, &[tags::SHOT_V0, tags::SHOT_V1, tags::SHOT_V2]),
    (
        tags::EXPLOSION,
        &[tags::EXPLOSION_V0, tags::EXPLOSION_V1],
    ),
];

pub fn load_sfx_catalog(mut catalog: ResMut<SfxCatalog>, asset_server: Res<AssetServer>) {
    for &(tag, path) in SFX_FILES {
        catalog.insert(tag, asset_server.load(path));
    }
    for &(cue_id, variants) in SFX_CUES {
        catalog.register_cue(cue_id, variants);
    }
}

/// Map one-shot kinds onto catalog tags when enqueueing from UI/combat.
#[must_use]
pub fn tag_for_oneshot(kind: AudioOneShotKind) -> &'static str {
    match kind {
        AudioOneShotKind::UiClick => tags::UI_CLICK,
        AudioOneShotKind::UiConfirm => tags::UI_CONFIRM,
        AudioOneShotKind::UiDeny => tags::UI_DENY,
        AudioOneShotKind::Shot => tags::SHOT,
        AudioOneShotKind::Explosion => tags::EXPLOSION,
        AudioOneShotKind::MissileLaunch => tags::MISSILE_LAUNCH,
        AudioOneShotKind::LaunchWarning => tags::LAUNCH_WARNING,
        AudioOneShotKind::WeatherAlert => tags::WEATHER_ALERT,
    }
}

/// Helper so callers set the correct mix tag for playback.
pub fn enqueue_oneshot(
    queue: &mut AudioOneShotQueue,
    class: AudioEventClass,
    kind: AudioOneShotKind,
    world_xz: Option<Vec2>,
) {
    let tag = tag_for_oneshot(kind);
    queue.pending.push(super::mix_frame::AudioMixRequest {
        class,
        kind: Some(kind),
        world_xz,
        gain: 1.0,
        tag,
    });
}
