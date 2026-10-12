//! Zone paint highlights — Bevy visual_authority owns map paint (retired egui painter).

use crate::construction::build_tool_authority::ZoneTool;

pub fn zone_fill(zone: ZoneTool) -> crate::gui::UiRgba {
    match zone {
        ZoneTool::ResidentialLow => crate::gui::UiRgba::from_rgba_unmultiplied(80, 200, 120, 90),
        ZoneTool::ResidentialMedium => crate::gui::UiRgba::from_rgba_unmultiplied(100, 220, 140, 100),
        ZoneTool::ResidentialHigh => crate::gui::UiRgba::from_rgba_unmultiplied(60, 180, 200, 100),
        ZoneTool::Apartments => crate::gui::UiRgba::from_rgba_unmultiplied(120, 140, 220, 110),
        ZoneTool::MixedUse => crate::gui::UiRgba::from_rgba_unmultiplied(200, 160, 80, 100),
    }
}
