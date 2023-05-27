use imgui::{Condition, ImString};
use imgui_miniquad::{ImGuiRenderer, RendererConfig};
use miniquad::{conf, EventHandler, KeyCode, MouseButton, UserData, Window, WindowEventHandler};
use std::time::Instant;

pub struct MyApp {
    // Keep track of the text input
    input_text: ImString,
    // Keep track of when the text was last updated
    last_text_update: Instant,
}

impl MyApp {
    pub fn new() -> MyApp {
        MyApp {
            input_text: ImString::new("Enter your text here"),
            last_text_update: Instant::now(),
        }
    }

    fn update_input_text(&mut self, ui: &imgui::Ui) {
        ui.window(im_str!("Text Input"))
            .position([50.0, 50.0], Condition::Always)
            .size([400.0, 200.0], Condition::Always)
            .build(|| {
                ui.input_text_multiline(im_str!(""), &mut self.input_text, [380.0, 160.0])
                    .build();
            });
    }

    fn render(&mut self, ui: &mut imgui::Ui) {
        // Update the text input every 100 milliseconds
        let now = Instant::now();
        if now.duration_since(self.last_text_update).as_millis() >= 100 {
            self.update_input_text(ui);
            self.last_text_update = now;
        }

        // Render the UI
        let draw_data = ui.render();
        unsafe {
            renderer.render(draw_data);
        }
    }
}

struct Stage {
    app: MyApp,
}

impl EventHandler for Stage {
    fn update(&mut self) {}

    fn draw(&mut self) {}

    fn resize_event(&mut self, _width: f32, _height: f32) {}

    fn mouse_motion_event(&mut self, _x: f32, _y: f32) {}

    fn mouse_wheel_event(&mut self, _x: f32, _y: f32) {}

    fn mouse_button_down_event(&mut self, _btn: MouseButton, _x: f32, _y: f32) {}

    fn mouse_button_up_event(&mut self, _btn: MouseButton, _x: f32, _y: f32) {}

    fn char_event(&mut self, character: char, _keymods: miniquad::KeyMods, _repeat: bool) {
        self.app.input_text.push(character);
    }

    fn key_down_event(
        &mut self,
        keycode: KeyCode,
        _keymods: miniquad::KeyMods,
        _repeat: bool,
    ) {
        if keycode == KeyCode::Backspace {
            self.app.input_text.pop();
        }
    }

    fn key_up_event(&mut self, _keycode: KeyCode, _keymods: miniquad::KeyMods) {}
}
