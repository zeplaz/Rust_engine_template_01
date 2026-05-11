//! Small overview image (same atlas); optional richer minimap later.

use bevy_egui::egui;

pub fn world_preview_minimap(ui: &mut egui::Ui, texture_id: egui::TextureId, tex_w: u32, tex_h: u32) {
    ui.label(egui::RichText::new("Overview").small().strong());
    let max_side = 140.0f32;
    let tw = tex_w.max(1) as f32;
    let th = tex_h.max(1) as f32;
    let s = max_side / tw.max(th);
    let w = tw * s;
    let h = th * s;
    let sized = egui::load::SizedTexture::new(texture_id, [w, h]);
    ui.image(sized);
}
