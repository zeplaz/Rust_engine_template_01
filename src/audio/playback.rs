//! Bind [`AudioMixFrame`] → Bevy audio players when [`SfxCatalog`] has handles.
//! No-op (and zero spawn) when catalog entries are missing — safe without SFX packs.
//!
//! Continuum beds (ambient / spatial) must **loop and persist** across frames. Recreating
//! short WAVs every tick sounds like beep-beep-doom.

use bevy::prelude::*;

use super::mix_frame::{AudioEventClass, AudioMixFrame, AudioMixSet};
use super::sfx_catalog::{AudioCuePicker, SfxCatalog};

#[derive(Component)]
pub struct AudioMixVoice {
    pub tag: &'static str,
    pub continuum: bool,
}

/// CQ-10: map AUD-05 low-pass corner → extra linear gain (no rodio EQ yet).
#[must_use]
fn weather_lowpass_gain(cutoff_hz: f32) -> f32 {
    const HI: f32 = 12_000.0;
    const LO: f32 = 650.0;
    let t = ((cutoff_hz - LO) / (HI - LO)).clamp(0.0, 1.0);
    0.55 + 0.45 * t
}

pub fn apply_audio_mix_playback(
    mut commands: Commands,
    mix: Res<AudioMixFrame>,
    catalog: Res<SfxCatalog>,
    mut cue_picker: ResMut<AudioCuePicker>,
    sources: Res<Assets<AudioSource>>,
    existing: Query<(Entity, &AudioMixVoice)>,
) {
    let lowpass_g = weather_lowpass_gain(mix.weather_lowpass_cutoff_hz);

    // Continuum tags requested this frame (resolved play tags).
    let mut continuum_wanted: Vec<&'static str> = Vec::new();
    for req in &mix.requests {
        if !matches!(
            req.class,
            AudioEventClass::Ambient | AudioEventClass::SpatialContinuum
        ) {
            continue;
        }
        let play_tag = catalog.resolve_play_tag(req.tag, &mut cue_picker);
        if !continuum_wanted.contains(&play_tag) {
            continuum_wanted.push(play_tag);
        }
    }

    // Drop continuum voices no longer requested. Leave fire-and-forget oneshots alone.
    for (e, voice) in &existing {
        if voice.continuum && !continuum_wanted.iter().any(|t| *t == voice.tag) {
            commands.entity(e).despawn();
        }
    }

    for req in &mix.requests {
        let play_tag = catalog.resolve_play_tag(req.tag, &mut cue_picker);
        let Some(handle) = catalog.get(play_tag) else {
            continue;
        };
        if sources.get(handle).is_none() {
            continue;
        }

        let is_continuum = matches!(
            req.class,
            AudioEventClass::Ambient | AudioEventClass::SpatialContinuum
        );

        if is_continuum {
            let already = existing
                .iter()
                .any(|(_, v)| v.continuum && v.tag == play_tag);
            if already {
                continue;
            }
            let mut gain = req.gain.clamp(0.0, 1.0);
            gain = (gain * lowpass_g).clamp(0.0, 1.0);
            // Soften placeholder beds — short loops are harsh at full gain.
            gain *= 0.35;
            commands.spawn((
                AudioPlayer::new(handle.clone()),
                PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(gain)),
                AudioMixVoice {
                    tag: play_tag,
                    continuum: true,
                },
                Name::new(format!("sfx:{}", play_tag)),
            ));
            continue;
        }

        // One-shots: fire-and-forget.
        let mut gain = req.gain.clamp(0.0, 1.0);
        // UI clicks are especially piercing when chrome refreshes.
        if matches!(req.class, AudioEventClass::Ui) {
            gain *= 0.45;
        }
        commands.spawn((
            AudioPlayer::new(handle.clone()),
            PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(gain)),
            AudioMixVoice {
                tag: play_tag,
                continuum: false,
            },
            Name::new(format!("sfx:{}", play_tag)),
        ));
    }
}

pub fn configure_playback(app: &mut App) {
    app.add_systems(
        Update,
        apply_audio_mix_playback.after(AudioMixSet::Build),
    );
}
