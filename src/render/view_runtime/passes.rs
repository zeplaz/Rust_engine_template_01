//! Schedule markers — systems in these passes must not cross [`super::ids::ViewIsolationGroup`].

/// Read world / write view-local presentation only.
pub struct OverlayPass;

/// Route pointer input to one surface.
pub struct InteractionPass;

/// Apply Bevy `Camera` viewport / target for [`super::ids::ViewEntity`].
pub struct ViewCameraCommitPass;
