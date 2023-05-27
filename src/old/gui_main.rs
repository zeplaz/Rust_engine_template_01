
use glow::{HasContext, Context};

use std::time::Instant;
use std::ffi::CString;
use lazy_static::lazy_static;

lazy_static! {
    static ref TIMES_NEW_ROMAN_FONTFILE: string = "data/times.tff";
}

struct Gui{
    active_context: Context,
    font_data:  Font_Datas
}

impl Event_Handler for Gui {
    miniquad::start(conf::Conf::default(), |mut ctx| {
       // Set the clear color to white
       ctx.clear(Some((1.0, 1.0, 1.0, 1.0)), None, None);

       // Run the main loop
       while miniquad::is_running(&ctx) {
           // Draw your application here
           // ...

           // Present the drawn frame to the window
           ctx.swap_buffers();
       }
       // Shut down the application
   });
}

impl open_prompt for Gui
{
    input_text: ImString,
  // Keep track of when the text was last updated
    last_text_update: Instant,
}

// should like move to guifolder?
fn window_conf() ->Conf
    Conf{
        window_title: "prmoter_".to_owned(),
      window_width: 1280,
      window_height: 720,
       ..Default::default()
    }

struct Font_Datas{
     font_size: float,
     font_configs: Vec<FontConfig>,
}

impl Font_Datas {
    fn set_font(&self, ctx: &mut Context, path_to_font: String) {
        let font_size = self.font_size;
        let font_configs = &self.font_configs;
        let font_source = FontSource::TtfData {
            data: include_bytes!(path_to_font),
            size_pixels: font_size,
            config: font_configs.iter().fold(FontConfig::default(), |mut config, font| {
                font.apply(&mut config);
                config
            }),
        };
        ctx.fonts().add_font(&[font_source]);
    }
}

impl Default for font_datas{
     fn default()-> Self{
        Self{
            font_size: 13.0,
            font_config:vec![FontConfig::default()],
        }
     }
 }



struct InputBoxState {
    text_buffer: ImString,
}

impl InputBoxState {
    fn new() -> Self {
        InputBoxState {
            text_buffer: ImString::with_capacity(1024),
        }
    }

    fn draw_ui(&mut self, ui: &Ui) -> Option<String> {
        let mut submitted_text = None;
        Window::new(im_str!("Input Box"))
            .size([400.0, 200.0], Condition::FirstUseEver)
            .build(ui, || {
                ui.input_text_multiline(
                    im_str!("##Input Box Text"),
                    &mut self.text_buffer,
                    [380.0, 160.0],
                    imgui::ImGuiInputTextFlags::empty(),
                    None,
                )
                .build();
                if ui.button(im_str!("Submit"), [80.0, 25.0]) {
                    submitted_text = Some(self.text_buffer.to_string());
                    self.text_buffer.clear();
                }
            });
        submitted_text
    }
}
