//! Transport R8 overlay slot for Wave S manifest composition.

use crate::io::save::manifest::OverlaySnapshotRef;

pub const TRANSPORT_OVERLAY_NAME: &str = "transport_r8";

#[must_use]
pub fn transport_overlay_ref(artifact_path: impl Into<String>) -> OverlaySnapshotRef {
    OverlaySnapshotRef {
        overlay_name: TRANSPORT_OVERLAY_NAME.into(),
        artifact_path: artifact_path.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transport_overlay_ref_uses_canonical_name() {
        let overlay = transport_overlay_ref("overlays/transport.ron");
        assert_eq!(overlay.overlay_name, TRANSPORT_OVERLAY_NAME);
    }
}
