use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};








pub struct Window {
    title: String,
    width: f32,
    height: f32,
}

impl Window {
    pub fn new(title: &str, width: f32, height: f32) -> Window {
        Window {
            title: title.to_string(),
            width,
            height,
        }
    }

    pub fn spawn(&self, commands: &mut Commands) {
            commands.spawn_bundle(WindowDesc::new(|| {
                bevy::DefaultPlugins
            }).title(&self.title).dimensions(self.width, self.height));
        }
}

struct WidgetState {
    input_text: String,
    show: bool,
}

impl Default for WidgetState {
    fn default() -> Self {
        Self {
            input_text: "".to_string(),
            show: false,
        }
    }
}

fn popup_system(
    mut popup_state: ResMut<WidgetState>,
    egui_ctx: ResMut<EguiContext>,
) {
    // Open the popup when a certain event happens
    if popup_state.show_popup {
        egui::Window::new("Input popup").show(egui_ctx.ctx(), |ui| {
            ui.add(egui::TextEdit::singleline(&mut popup_state.input_text));
            if ui.button("Confirm").clicked() {
                // Do something with the input text
                println!("User input: {}", popup_state.input_text);
                popup_state.show_popup = false;
            }
        });
    }
}
