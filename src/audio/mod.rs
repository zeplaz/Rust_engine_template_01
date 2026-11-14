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
    AudioEventClass, AudioMixFrame, AudioMixRequest, AudioMixSet, AudioOneShotKind, AudioOneShotQueue,
};
pub use plugin::AudioMixPlugin;
pub use sfx_catalog::{tags as sfx_tags, SfxCatalog};
