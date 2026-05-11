//! Resource-delivered construction progression (P2-C).

use bevy::prelude::*;

use super::components::{ConstructionSite, SiteConstructionRate, SiteResourceManifest};
use super::resources::{SiteConstructionBook, SiteConstructionPhase, SiteId};

/// Advances `operational_readiness` from manifest ratios × rate modifiers (not wall-clock timers).
pub fn site_construction_progression_system(
    mut q: Query<(
        &mut ConstructionSite,
        &SiteResourceManifest,
        Option<&SiteConstructionRate>,
    )>,
    mut book: ResMut<SiteConstructionBook>,
) {
    for (mut site, manifest, rate) in &mut q {
        if site.phase != SiteConstructionPhase::UnderConstruction {
            continue;
        }
        let r = rate.copied().unwrap_or_default();
        let eff = (r.labor_efficiency * r.machinery_efficiency * r.weather_penalty).clamp(0.0, 1.0);
        site.operational_readiness = (manifest.delivered_ratio() * eff).clamp(0.0, 1.0);
        let sid = SiteId(site.site_id);
        if let Some(st) = book.by_site.get_mut(&sid) {
            st.progress = site.operational_readiness;
            st.phase = SiteConstructionPhase::UnderConstruction;
        }
        if site.operational_readiness >= 1.0 {
            site.phase = SiteConstructionPhase::Provisioning;
            site.operational_readiness = 0.0;
            if let Some(st) = book.by_site.get_mut(&sid) {
                st.phase = SiteConstructionPhase::Provisioning;
                st.progress = 0.0;
            }
        }
    }
}
