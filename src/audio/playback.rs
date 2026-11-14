//! Bind [`AudioMixFrame`] → Bevy audio players when [`SfxCatalog`] has handles.
//! No-op (and zero spawn) when catalog entries are missing — safe without SFX packs.

use bevy::prelude::*;

use super::mix_frame::{AudioEventClass, AudioMixFrame, AudioMixSet};
use super::sfx_catalog::SfxCatalog;

#[derive(Component)]
pub struct AudioMixVoice {
    pub tag: &'static str,
}

/// Spawn one-shot / start continuum voices for catalog hits. Ambient continuum is
/// recreated each frame only when a handle exists (placeholder until looping beds).
pub fn apply_audio_mix_playback(
    mut commands: Commands,
    mix: Res<AudioMixFrame>,
    catalog: Res<SfxCatalog>,
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

    for req in &mix.requests {
        let Some(handle) = catalog.get(req.tag) else {
            continue;
        };
        // Skip until decoded — Bevy 0.20 rodio path panics on empty/invalid sources.
        if sources.get(handle).is_none() {
            continue;
        }
        let already = existing.iter().any(|(_, v)| v.tag == req.tag);
        if already && matches!(
            req.class,
            AudioEventClass::Ambient | AudioEventClass::SpatialContinuum
        ) {
            continue;
        }
        commands.spawn((
            AudioPlayer::new(handle.clone()),
            PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(req.gain.clamp(0.0, 1.0))),
            AudioMixVoice { tag: req.tag },
            Name::new(format!("sfx:{}", req.tag)),
        ));
    }
}

pub fn configure_playback(app: &mut App) {
    app.add_systems(
        Update,
        apply_audio_mix_playback.after(AudioMixSet::Build),
    );
}
