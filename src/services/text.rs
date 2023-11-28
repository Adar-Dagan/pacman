use bevy::prelude::*;
use text_to_png::{FontSize, TextRenderer};

const ASSET_DIR: &str = "assets";
const TEMP_FONTS_DIR: &str = "temp_fonts";

#[derive(Component, Default)]
pub struct InGameText {
    text: &'static str,
    color: Color,
    initiated: bool,
}

#[derive(Bundle, Default)]
pub struct InGameTextBundle {
    pub text: InGameText,
    sprite: SpriteBundle,
}

pub struct InGameTextPlugin;

impl Plugin for InGameTextPlugin {
    fn build(&self, app: &mut App) {
        std::fs::create_dir_all(format!("{}/{}", ASSET_DIR, TEMP_FONTS_DIR)).unwrap();
        app.add_systems(Update, update);
    }
}

impl InGameTextBundle {
    pub fn new(text: &'static str, color: Color) -> Self {
        debug_assert!(text
            .chars()
            .all(|c| c.is_uppercase() || c.is_ascii_punctuation()));
        Self {
            text: InGameText {
                text,
                color,
                initiated: false,
            },
            ..default()
        }
    }
}

fn update(mut query: Query<(&mut InGameText, &mut Handle<Image>)>, asset_server: Res<AssetServer>) {
    for (mut text, mut image) in query.iter_mut().filter(|(text, _)| !text.initiated) {
        let renderer =
            TextRenderer::try_new_with_ttf_font_data(include_bytes!("../../assets/joystix.otf"))
                .expect("Failed to create text renderer");

        let [r, g, b, _] = text.color.as_rgba_u8();
        let color = text_to_png::Color::new(r, g, b);
        let png = renderer
            .render_text_to_png_data(&text.text, FontSize::FillHeight(7.0), color)
            .expect("Failed to render text");

        let file_name = format!(
            "{}/{}_{:02x}{:02x}{:02x}.png",
            TEMP_FONTS_DIR, text.text, r, g, b
        );
        std::fs::write(format!("{}/{}", ASSET_DIR, file_name), &png.data)
            .expect("Failed to write image to file");

        *image = asset_server.load(file_name);
        text.initiated = true;
    }
}

impl Drop for InGameTextPlugin {
    fn drop(&mut self) {
        std::fs::remove_dir_all("assets/temp_fonts").unwrap();
    }
}
