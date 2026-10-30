//! Representation spine — registered visual producers (semantic → sole authority path).
//!
//! **Allowed LOD inputs (resolver domain only):** `MapCameraDesired`, `LodZoneRegistry`,
//! gameplay importance, `WorldLodPolicyEngine` inside `WorldRepresentationResolver`.
//!
//! **Forbidden in render/compute/upload consumers:** `visibility_for_band`, per-frame zoom
//! branches that change GPU rows without `RepresentationResult`.
//! (Source-scan audit tests for these rules were retired; the rules stand.)

/// Registered visual producers (semantic → sole authority path).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisualProducerRegistration {
    pub semantic: &'static str,
    pub producer: &'static str,
}

pub const REGISTERED_VISUAL_PRODUCERS: &[VisualProducerRegistration] = &[
    VisualProducerRegistration {
        semantic: "fire_visual",
        producer: "src/render/extraction/fire_visual_extract.rs",
    },
    VisualProducerRegistration {
        semantic: "overlay_heat",
        producer: "src/render/overlay_field_buffers.rs",
    },
];

#[must_use]
pub fn fire_visual_producer_count() -> u32 {
    REGISTERED_VISUAL_PRODUCERS
        .iter()
        .filter(|row| row.semantic == "fire_visual")
        .count() as u32
}
