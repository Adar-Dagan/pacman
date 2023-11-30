use bevy::prelude::*;
use text_to_png::{FontSize, TextRenderer};

const ASSET_DIR: &str = "assets";
const TEMP_FONTS_DIR: &str = "temp_fonts";

#[derive(Resource)]
pub struct TextProvider;

pub struct TextProviderPlugin;

impl Plugin for TextProviderPlugin {
    fn build(&self, app: &mut App) {
        std::fs::create_dir_all(format!("{}/{}", ASSET_DIR, TEMP_FONTS_DIR)).unwrap();
        app.insert_resource(TextProvider);
    }
}

impl TextProvider {
    pub fn get_image(
        &self,
        text: &str,
        color: Color,
        asset_server: &Res<AssetServer>,
    ) -> Handle<Image> {
        let renderer =
            TextRenderer::try_new_with_ttf_font_data(include_bytes!("../../assets/joystix.otf"))
                .expect("Failed to create text renderer");

        let [r, g, b, _] = color.as_rgba_u8();
        let color = text_to_png::Color::new(r, g, b);
        let png = renderer
            .render_text_to_png_data(text, FontSize::FillHeight(7.0), color)
            .expect("Failed to render text");

        let file_name = format!(
            "{}/{}_{:02x}{:02x}{:02x}.png",
            TEMP_FONTS_DIR, text, r, g, b
        );
        std::fs::write(format!("{}/{}", ASSET_DIR, file_name), &png.data)
            .expect("Failed to write image to file");

        return asset_server.load(file_name);
    }

    pub fn get_size(&self, text: &str) -> Vec2 {
        return Vec2::new(8.0 * text.len() as f32, 7.0);
    }
}

impl Drop for TextProviderPlugin {
    fn drop(&mut self) {
        std::fs::remove_dir_all("assets/temp_fonts").unwrap();
    }
}
