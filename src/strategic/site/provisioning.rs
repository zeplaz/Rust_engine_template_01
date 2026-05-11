//! Provisioning → operational (P2-D). Built ≠ operating until networks + stats satisfy gates.

use bevy::prelude::*;

use super::components::{ConstructionSite, SiteNetworkAttachment, SiteOperationalStats};
use super::resources::{SiteConstructionBook, SiteConstructionPhase, SiteId};

pub fn site_provisioning_system(
    mut q: Query<(
        &mut ConstructionSite,
        &SiteNetworkAttachment,
        &SiteOperationalStats,
    )>,
    mut book: ResMut<SiteConstructionBook>,
) {
    for (mut site, _net, stats) in &mut q {
        if site.phase != SiteConstructionPhase::Provisioning {
            continue;
        }
        let sid = SiteId(site.site_id);
        let readiness =
            (stats.power_ratio + stats.supply_ratio + stats.workforce_ratio) / 3.0;
        site.operational_readiness = readiness.clamp(0.0, 1.0);
        if let Some(st) = book.by_site.get_mut(&sid) {
            st.progress = site.operational_readiness;
            st.phase = SiteConstructionPhase::Provisioning;
        }
        if stats.power_ratio >= 0.25
            && stats.supply_ratio >= 0.25
            && stats.workforce_ratio >= 0.25
        {
            site.phase = SiteConstructionPhase::Operational;
            site.operational_readiness = 1.0;
            if let Some(st) = book.by_site.get_mut(&sid) {
                st.phase = SiteConstructionPhase::Operational;
                st.progress = 1.0;
            }
        }
    }
}
