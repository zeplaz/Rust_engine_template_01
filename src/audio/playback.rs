//! Bind [`AudioMixFrame`] → Bevy audio players when [`SfxCatalog`] has handles.
//! No-op (and zero spawn) when catalog entries are missing — safe without SFX packs.

use bevy::prelude::*;

use super::mix_frame::{AudioEventClass, AudioMixFrame, AudioMixSet};
use super::sfx_catalog::{AudioCuePicker, SfxCatalog};

#[derive(Component)]
pub struct AudioMixVoice {
    pub tag: &'static str,
}

/// CQ-10: map AUD-05 low-pass corner → extra linear gain (no rodio EQ yet).
/// Dry air (~12 kHz) → 1.0; wet air (~650 Hz) → ~0.55 on ambient / spatial beds.
#[must_use]
fn weather_lowpass_gain(cutoff_hz: f32) -> f32 {
    const HI: f32 = 12_000.0;
    const LO: f32 = 650.0;
    let t = ((cutoff_hz - LO) / (HI - LO)).clamp(0.0, 1.0);
    0.55 + 0.45 * t
}

/// Spawn one-shot / start continuum voices for catalog hits. Ambient continuum is
/// recreated each frame only when a handle exists (placeholder until looping beds).
pub fn apply_audio_mix_playback(
    mut commands: Commands,
    mix: Res<AudioMixFrame>,
    catalog: Res<SfxCatalog>,
    mut cue_picker: ResMut<AudioCuePicker>,
    sources: Res<Assets<AudioSource>>,
    existing: Query<(Entity, &AudioMixVoice)>,
) {
    // Tear down prior continuum voices; one-shots are fire-and-forget entities.
    for (e, voice) in &existing {
        let keep_oneshot = mix.requests.iter().any(|r| {
            r.tag == voice.tag && matches!(r.class, AudioEventClass::CombatOneShot | AudioEventClass::Ui | AudioEventClass::Warning)
        });
        if !keep_oneshot {
            commands.entity(e).despawn();
        }
    }

    let lowpass_g = weather_lowpass_gain(mix.weather_lowpass_cutoff_hz);
    for req in &mix.requests {
        let play_tag = catalog.resolve_play_tag(req.tag, &mut cue_picker);
        let Some(handle) = catalog.get(play_tag) else {
            continue;
        };
        // Skip until decoded — Bevy 0.20 rodio path panics on empty/invalid sources.
        if sources.get(handle).is_none() {
            continue;
        }
        let already = existing.iter().any(|(_, v)| v.tag == play_tag);
        if already && matches!(
            req.class,
            AudioEventClass::Ambient | AudioEventClass::SpatialContinuum
        ) {
            continue;
        }
        // Duck already applied on Ambient requests in mix_frame; CQ-10 wires low-pass here.
        let mut gain = req.gain.clamp(0.0, 1.0);
        if matches!(
            req.class,
            AudioEventClass::Ambient | AudioEventClass::SpatialContinuum
        ) {
            gain = (gain * lowpass_g).clamp(0.0, 1.0);
        }
        commands.spawn((
            AudioPlayer::new(handle.clone()),
            PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(gain)),
            AudioMixVoice { tag: play_tag },
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
