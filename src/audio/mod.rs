//! Presentation audio — reads sim events + view pose; **never** writes sim truth.
//!
//! PLAN-REF-BOOKS Ranking **C1** / Slice C. Sole mix writer: [`mix_frame::build_audio_mix_frame`].

mod load_sfx;
mod mix_frame;
mod playback;
mod plugin;
mod sfx_catalog;
mod ui_hooks;

pub use load_sfx::{enqueue_oneshot, tag_for_oneshot};
pub use mix_frame::{
    AudioEventClass, AudioMixDuckState, AudioMixFrame, AudioMixRequest, AudioMixSet,
    AudioOneShotKind, AudioOneShotQueue, AMBIENCE_DUCK_TARGET_GAIN, COMBAT_ONESHOT_CAP,
};
pub use plugin::AudioMixPlugin;
pub use sfx_catalog::{tags as sfx_tags, AudioCuePicker, AudioLocalRng, SfxCatalog};
pub use ui_hooks::{
    alert_bark_kind_for_oneshot, alert_bark_priority, AdmitAlertBark, AlertBarkKind, AlertBarkLane,
};
