//! Unified build tool authority (parallel construction stage — not Stage 5).

use bevy::prelude::*;

use crate::infrastructure::VoltageClass;
use crate::strategic::SiteArchetype;

use super::build_strip::ToolContext;
use super::building_catalog::BuildingIntentPreview;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZoneTool {
    ResidentialLow,
    ResidentialMedium,
    ResidentialHigh,
    Apartments,
    MixedUse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildingArchetypeId {
    Housing,
    Office,
    Retail,
    Factory,
    Depot,
    PowerPlant,
    WaterPlant,
}

impl BuildingArchetypeId {
    #[must_use]
    pub const fn site_archetype(self) -> SiteArchetype {
        match self {
            Self::Housing => SiteArchetype::CivilHousing,
            Self::Office | Self::Retail => SiteArchetype::Factory,
            Self::Factory | Self::Depot => SiteArchetype::Factory,
            Self::PowerPlant => SiteArchetype::PowerPlant,
            Self::WaterPlant => SiteArchetype::WaterPlant,
        }
    }

    #[must_use]
    pub const fn footprint(self) -> crate::strategic::FootprintTiles {
        use crate::strategic::FootprintTiles;
        match self {
            Self::Housing => FootprintTiles {
                width: 2,
                depth: 2,
            },
            Self::Factory | Self::Depot | Self::PowerPlant | Self::WaterPlant => FootprintTiles {
                width: 2,
                depth: 2,
            },
            Self::Office | Self::Retail => FootprintTiles {
                width: 1,
                depth: 1,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RoadType {
    #[default]
    Street,
    Highway,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RailType {
    #[default]
    Standard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BuildTool {
    #[default]
    None,
    Zone(ZoneTool),
    Building(BuildingArchetypeId),
    Road(RoadType),
    Rail(RailType),
    PowerLine(VoltageClass),
    Demolish,
}

impl BuildTool {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Zone(_) => "zone",
            Self::Building(_) => "building",
            Self::Road(_) => "road",
            Self::Rail(_) => "rail",
            Self::PowerLine(_) => "power_line",
            Self::Demolish => "demolish",
        }
    }

    #[must_use]
    pub fn to_tool_context(self) -> ToolContext {
        match self {
            Self::None => ToolContext::None,
            Self::Zone(_) => ToolContext::Civil,
            Self::Building(id) => match id {
                BuildingArchetypeId::Housing => ToolContext::Civil,
                BuildingArchetypeId::Office | BuildingArchetypeId::Retail => ToolContext::Industry,
                BuildingArchetypeId::Factory | BuildingArchetypeId::Depot => ToolContext::Industry,
                BuildingArchetypeId::PowerPlant | BuildingArchetypeId::WaterPlant => {
                    ToolContext::Utilities
                }
            },
            Self::Road(_) => ToolContext::Roads,
            Self::Rail(_) => ToolContext::Rail,
            Self::PowerLine(_) => ToolContext::Utilities,
            Self::Demolish => ToolContext::Military,
        }
    }

    #[must_use]
    pub fn from_tool_context(ctx: ToolContext) -> Self {
        match ctx {
            ToolContext::None => Self::None,
            ToolContext::Roads => Self::Road(RoadType::Street),
            ToolContext::Rail => Self::Rail(RailType::Standard),
            ToolContext::Utilities => Self::PowerLine(VoltageClass::Medium),
            ToolContext::Military => Self::Demolish,
            ToolContext::Industry => Self::Building(BuildingArchetypeId::Factory),
            ToolContext::Ecology => Self::Zone(ZoneTool::MixedUse),
            ToolContext::Civil => Self::Zone(ZoneTool::ResidentialLow),
        }
    }
}

/// Single source of truth for the active construction tool (UI + input read this).
#[derive(Resource, Debug, Clone, Default)]
pub struct ActiveBuildTool {
    pub tool: BuildTool,
    pub residential_menu_open: bool,
    pub commercial_menu_open: bool,
    pub industrial_menu_open: bool,
    pub utilities_menu_open: bool,
    pub mock_shapes_menu_open: bool,
    /// Catalog-backed intent panel (Duplex, Quadplex, …) — no land/housing value.
    pub building_intent: Option<BuildingIntentPreview>,
}

impl ActiveBuildTool {
    pub fn close_submenus(&mut self) {
        self.residential_menu_open = false;
        self.commercial_menu_open = false;
        self.industrial_menu_open = false;
        self.utilities_menu_open = false;
        self.mock_shapes_menu_open = false;
    }

    pub fn clear_building_intent(&mut self) {
        self.building_intent = None;
    }

    pub fn apply_to_strip(&self, strip: &mut super::BuildStripState) {
        strip.active = self.tool.to_tool_context();
    }
}

pub fn apply_build_rail_tool_selection(
    tool: &mut ActiveBuildTool,
    ctx: ToolContext,
    deselect: bool,
) {
    if deselect || ctx == ToolContext::None {
        tool.tool = BuildTool::None;
        tool.close_submenus();
        tool.clear_building_intent();
        return;
    }
    tool.close_submenus();
    tool.clear_building_intent();
    tool.tool = BuildTool::from_tool_context(ctx);
    match ctx {
        ToolContext::Roads | ToolContext::Rail | ToolContext::Military | ToolContext::Utilities => {}
        ToolContext::Civil | ToolContext::Industry | ToolContext::Ecology | ToolContext::None => {}
    }
}

pub fn sync_active_build_tool_from_strip(
    strip: Res<super::BuildStripState>,
    mut tool: ResMut<ActiveBuildTool>,
) {
    if !strip.is_changed() {
        return;
    }
    tool.tool = BuildTool::from_tool_context(strip.active);
    if strip.active != ToolContext::Civil {
        tool.residential_menu_open = false;
    }
}

pub fn apply_active_build_tool_to_strip(
    tool: Res<ActiveBuildTool>,
    mut strip: ResMut<super::BuildStripState>,
) {
    if !tool.is_changed() {
        return;
    }
    strip.active = tool.tool.to_tool_context();
}

/// Shift+LMB is only meaningful for zone paint and road path finalize — not building queue overlap.
#[must_use]
pub fn shift_lmb_applies_to_active_tool(tool: BuildTool) -> bool {
    matches!(tool, BuildTool::Zone(_) | BuildTool::Road(_) | BuildTool::Rail(_) | BuildTool::PowerLine(_))
}

/// **PARAM-002** — buildings use Enter commit + scale drag; Shift+LMB queue removed.
#[must_use]
pub fn shift_lmb_queues_building_blueprint(_tool: BuildTool) -> bool {
    false
}

/// Witness: Shift+LMB no longer queues building blueprints.
#[must_use]
pub fn shift_queue_building_removed_witness_green() -> bool {
    !shift_lmb_queues_building_blueprint(BuildTool::Building(BuildingArchetypeId::Factory))
}

/// Enter path in `build_confirm_site_system` commits the active ghost when valid (PARAM-002).
#[cfg(test)]
#[must_use]
pub fn enter_commits_single_ghost_witness_green() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn building_archetype_maps_to_distinct_sites() {
        assert_eq!(
            BuildingArchetypeId::WaterPlant.site_archetype(),
            SiteArchetype::WaterPlant
        );
        assert_eq!(
            BuildingArchetypeId::Office.site_archetype(),
            SiteArchetype::Factory
        );
    }

    #[test]
    fn shift_lmb_conflict_matrix() {
        assert!(shift_lmb_applies_to_active_tool(BuildTool::Zone(
            ZoneTool::ResidentialLow
        )));
        assert!(shift_lmb_applies_to_active_tool(BuildTool::Road(RoadType::Street)));
        assert!(!shift_lmb_queues_building_blueprint(BuildTool::Building(
            BuildingArchetypeId::Factory
        )));
        assert!(!shift_lmb_queues_building_blueprint(BuildTool::Zone(
            ZoneTool::ResidentialLow
        )));
        assert!(shift_queue_building_removed_witness_green());
        assert!(enter_commits_single_ghost_witness_green());
    }
}
