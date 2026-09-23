//! EFFECTS-SYSTEM ES-2 — particle domain registry (fire = domain 0).
//!
//! Buffer IDs and GPU layouts stay under existing `FIRE_*` / fire_vfx paths — this module is the
//! **authoritative catalog** of which presentation domains share the instanced-quad backend.

use bevy::prelude::*;

/// Stable domain id for GPU instanced-quad frontends (ES-2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ParticleDomainId {
    Fire = 0,
    /// Reserved — weather precip streaks (ES-5).
    WeatherPrecip = 1,
    /// Reserved — ash / ember secondary (future).
    Ash = 2,
}

impl ParticleDomainId {
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Fire => "fire",
            Self::WeatherPrecip => "weather_precip",
            Self::Ash => "ash",
        }
    }
}

/// One registered frontend domain (documentation + witness; buffer ownership stays in fx spine).
#[derive(Clone, Copy, Debug)]
pub struct ParticleDomainEntry {
    pub id: ParticleDomainId,
    pub active: bool,
    pub backend: &'static str,
    pub frontend_module: &'static str,
}

/// Authoritative registry — fire is the only shipped domain until ES-5.
#[derive(Resource, Clone, Debug)]
pub struct ParticleDomainRegistry {
    pub entries: Vec<ParticleDomainEntry>,
}

impl Default for ParticleDomainRegistry {
    fn default() -> Self {
        Self {
            entries: vec![
                ParticleDomainEntry {
                    id: ParticleDomainId::Fire,
                    active: true,
                    backend: "gpu_instanced_quad",
                    frontend_module: "render::fire_vfx",
                },
                ParticleDomainEntry {
                    id: ParticleDomainId::WeatherPrecip,
                    active: false,
                    backend: "gpu_instanced_quad",
                    frontend_module: "render::weather_vfx",
                },
                ParticleDomainEntry {
                    id: ParticleDomainId::Ash,
                    active: false,
                    backend: "gpu_instanced_quad",
                    frontend_module: "reserved",
                },
            ],
        }
    }
}

impl ParticleDomainRegistry {
    #[must_use]
    pub fn fire_active(&self) -> bool {
        self.entries
            .iter()
            .any(|e| e.id == ParticleDomainId::Fire && e.active)
    }

    #[must_use]
    pub fn weather_precip_active(&self) -> bool {
        self.entries
            .iter()
            .any(|e| e.id == ParticleDomainId::WeatherPrecip && e.active)
    }

    /// ES-5 — activate WeatherPrecip only when CPU mesh soft-gate is OFF (no dual-write).
    pub fn set_weather_precip_active(&mut self, active: bool) {
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|e| e.id == ParticleDomainId::WeatherPrecip)
        {
            entry.active = active;
        }
    }

    #[must_use]
    pub fn active_count(&self) -> usize {
        self.entries.iter().filter(|e| e.active).count()
    }

    #[must_use]
    pub fn witness_json(&self) -> serde_json::Value {
        serde_json::json!({
            "particle_domain_registry": true,
            "active_domains": self.active_count(),
            "fire_domain_0": self.fire_active(),
            "domains": self.entries.iter().map(|e| {
                serde_json::json!({
                    "id": e.id.as_u8(),
                    "label": e.id.label(),
                    "active": e.active,
                    "backend": e.backend,
                    "frontend": e.frontend_module,
                })
            }).collect::<Vec<_>>(),
        })
    }
}

pub struct ParticleDomainRegistryPlugin;

impl Plugin for ParticleDomainRegistryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParticleDomainRegistry>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_is_domain_zero_and_active() {
        let reg = ParticleDomainRegistry::default();
        assert!(reg.fire_active());
        assert_eq!(reg.entries[0].id, ParticleDomainId::Fire);
        assert_eq!(ParticleDomainId::Fire.as_u8(), 0);
        let w = reg.witness_json();
        assert_eq!(w["particle_domain_registry"], true);
        assert_eq!(w["fire_domain_0"], true);
    }
}
