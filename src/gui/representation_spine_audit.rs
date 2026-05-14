//! Representation spine audit — allowed producer/consumer boundaries (Gates 1–2).
//!
//! **Allowed LOD inputs (resolver domain only):** `MapCameraDesired`, `LodZoneRegistry`,
//! gameplay importance, `WorldLodPolicyEngine` inside `WorldRepresentationResolver`.
//!
//! **Forbidden in render/compute/upload consumers:** `visibility_for_band`, per-frame zoom
//! branches that change GPU rows without `RepresentationResult`.

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

/// Paths that may call band tables when building [`super::RepresentationResult`].
pub const RESOLVER_DOMAIN_PATHS: &[&str] = &[
    "src/gui/world_representation.rs",
    "src/gui/representation_policy.rs",
];

/// Paths that must not import `visibility_for_band` (policy bypass).
pub const POLICY_CONSUMER_PATHS: &[&str] = &[
    "src/compute/compute_dispatch_graph.rs",
    "src/compute/heat_diffusion.rs",
    "src/render/gpu_weather_fire_field.rs",
    "src/render/gpu_particle_draw.rs",
    "src/render/gpu_particles.rs",
    "src/systems/atmosphere/render_layers.rs",
];

/// Sole ECS fire scan authority for visual snapshots (`FireVisualFrame`).
pub const FIRE_VISUAL_EXTRACT_AUTHORITY: &[&str] = &[
    "src/render/extraction/fire_visual_extract.rs",
    "src/render/extraction/fire_emission_profile.rs",
];

/// Render / preview / particle consumers must not query [`crate::systems::fire::ChunkSurfaceFire`].
pub const FIRE_VISUAL_CONSUMER_ROOTS: &[&str] = &[
    "src/render",
    "src/gui/editor/world_preview",
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn repo_root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
    }

    #[test]
    fn registered_visual_producers_include_single_fire_authority() {
        assert_eq!(fire_visual_producer_count(), 1);
        assert!(REGISTERED_VISUAL_PRODUCERS
            .iter()
            .any(|row| row.semantic == "fire_visual"));
    }

    #[test]
    fn resolver_domain_paths_exist() {
        let root = repo_root();
        for rel in RESOLVER_DOMAIN_PATHS {
            let path = root.join(rel);
            assert!(path.is_file(), "missing resolver domain path: {rel}");
        }
    }

    #[test]
    fn policy_consumers_do_not_import_visibility_for_band() {
        for rel in POLICY_CONSUMER_PATHS {
            let path = repo_root().join(rel);
            let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel}: {e}"));
            assert!(
                !text.contains("visibility_for_band"),
                "{rel} must read RepresentationResult only, not visibility_for_band"
            );
        }
    }

    #[test]
    fn burst_hints_only_in_projection_graph_and_fx_module() {
        let root = repo_root();
        let mut offenders = Vec::new();
        for entry in walkdir_lite(&root.join("src")) {
            if entry
                .file_name()
                .is_some_and(|n| n == "fx_burst_request.rs" || n == "render_projection_graph.rs" || n == "mod.rs" || n == "representation_spine_audit.rs")
            {
                continue;
            }
            let text = std::fs::read_to_string(&entry).unwrap();
            if text.contains("FxParticleBurstRequest") && !entry.ends_with("mod.rs") {
                offenders.push(entry);
            }
        }
        assert!(
            offenders.is_empty(),
            "FxParticleBurstRequest outside projection: {:?}",
            offenders
        );
    }

    #[test]
    fn chunk_surface_fire_not_scanned_in_visual_consumers() {
        let root = repo_root();
        let allowed: Vec<_> = FIRE_VISUAL_EXTRACT_AUTHORITY
            .iter()
            .map(|rel| root.join(rel))
            .collect();
        let mut offenders = Vec::new();
        for rel in FIRE_VISUAL_CONSUMER_ROOTS {
            for entry in walkdir_lite(&root.join(rel)) {
                if allowed.iter().any(|a| entry == *a) {
                    continue;
                }
                if entry
                    .file_name()
                    .is_some_and(|n| n == "representation_spine_audit.rs")
                {
                    continue;
                }
                let text = std::fs::read_to_string(&entry).unwrap_or_default();
                if text.contains("ChunkSurfaceFire") {
                    offenders.push(entry);
                }
            }
        }
        assert!(
            offenders.is_empty(),
            "ChunkSurfaceFire ECS scan outside extract authority: {:?}",
            offenders
        );
    }

    #[test]
    fn gpu_buffer_allocation_authority_is_registry() {
        let root = repo_root();
        let registry = root.join("src/render/gpu_buffer_registry.rs");
        let mut offenders = Vec::new();
        for entry in walkdir_lite(&root.join("src/render")) {
            if entry == registry {
                continue;
            }
            let text = std::fs::read_to_string(&entry).unwrap_or_default();
            if text.contains("create_buffer(") {
                offenders.push(entry);
            }
        }
        assert!(
            offenders.is_empty(),
            "RenderDevice::create_buffer outside GPUBufferRegistry: {:?}",
            offenders
        );
    }

    fn walkdir_lite(dir: &Path) -> Vec<std::path::PathBuf> {
        let mut out = Vec::new();
        if let Ok(read) = std::fs::read_dir(dir) {
            for entry in read.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    out.extend(walkdir_lite(&path));
                } else if path.extension().is_some_and(|e| e == "rs") {
                    out.push(path);
                }
            }
        }
        out
    }
}
