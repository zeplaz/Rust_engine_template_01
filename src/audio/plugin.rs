//! Audio mix plugin — presentation only.

use bevy::prelude::*;

use crate::gui::ViewAuthoritySystemSet;

use super::load_sfx::load_sfx_catalog;
use super::mix_frame::{
    build_audio_mix_frame, AudioMixDuckState, AudioMixFrame, AudioMixSet, AudioOneShotQueue,
};
use super::playback::configure_playback;
use super::sfx_catalog::SfxCatalog;
use super::sfx_catalog::AudioCuePicker;
use super::ui_hooks::{apply_alert_bark_exclusivity, enqueue_ui_button_clicks, AlertBarkLane};

pub struct AudioMixPlugin;

impl Plugin for AudioMixPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioMixFrame>()
            .init_resource::<AudioMixDuckState>()
            .init_resource::<AudioOneShotQueue>()
            .init_resource::<AlertBarkLane>()
            .insert_resource(AudioCuePicker::new(0x41A0_5F00_0003))
            .init_resource::<SfxCatalog>()
            .add_systems(Startup, load_sfx_catalog)
            .configure_sets(
                Update,
                AudioMixSet::Build.after(ViewAuthoritySystemSet::SyncViewManager),
            )
            .add_systems(
                Update,
                (
                    enqueue_ui_button_clicks.before(AudioMixSet::Build),
                    apply_alert_bark_exclusivity.before(AudioMixSet::Build),
                    build_audio_mix_frame.in_set(AudioMixSet::Build),
                ),
            );
        configure_playback(app);
    }
}
