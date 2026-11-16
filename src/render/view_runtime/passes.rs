//! Schedule markers — systems in these passes must not cross [`super::ids::ViewIsolationGroup`].
//!
//! # LGT-06 — Tone mapping / HDR (V4-LGT06)
//!
//! Dual-cite: NEW **LGT-06** (gea4v2 12.7.1 p244-245) · LEGACY **`lighting_post`** /
//! **`post_processing`** (gea_v2_e4 p153 / §12.7 p244). Contract body:
//! [`super::tonemap_hdr`] — SDR map RTT, overlay `[SDR,HDR]` PSO index, PixelWorld
//! `Tonemapping::None`, overlays before `EarlyPostProcess`.

/// Read world / write view-local presentation only.
pub struct OverlayPass;

/// Route pointer input to one surface.
pub struct InteractionPass;

/// Apply Bevy `Camera` viewport / target for [`super::ids::ViewEntity`].
pub struct ViewCameraCommitPass;

/// Display-referred tone-map / HDR policy slot after overlay lighting (LGT-06).
///
/// Does not write sim. Systems that change Bevy `Tonemapping` or RTT format for a
/// map view must honor [`super::tonemap_hdr::MAP_VIEW_TONE_MAP_POLICY`].
pub struct ToneMapHdrPass;
