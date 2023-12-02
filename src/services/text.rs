use std::fmt::Display;

use bevy::{prelude::*, utils::HashMap};
use text_to_png::{FontSize, TextRenderer};

const ASSET_DIR: &str = "assets";
const TEMP_FONTS_DIR: &str = "temp_fonts";

#[derive(Resource)]
pub struct TextProvider {
    renderer: TextRenderer,
    cache: HashMap<String, Handle<Image>>,
}

pub struct TextProviderPlugin;

impl Plugin for TextProviderPlugin {
    fn build(&self, app: &mut App) {
        std::fs::create_dir_all(format!("{}/{}", ASSET_DIR, TEMP_FONTS_DIR)).unwrap();
        app.insert_resource(TextProvider {
            renderer: TextRenderer::try_new_with_ttf_font_data(include_bytes!(
                "../../assets/joystix.otf"
            ))
            .expect("Failed to create text renderer"),
            cache: HashMap::new(),
        });
    }
}

impl TextProvider {
    pub fn get_image<T: Display>(
        &mut self,
        text: T,
        color: Color,
        asset_server: &AssetServer,
    ) -> Handle<Image> {
        let text = format!("{}", text);
        let text = text.to_uppercase();

        let [r, g, b, _] = color.as_rgba_u8();

        let file_name = format!(
            "{}/{}_{:02x}{:02x}{:02x}.png",
            TEMP_FONTS_DIR, text, r, g, b
        );

        if let Some(handle) = self.cache.get(&file_name) {
            return handle.clone();
        }

        let color = text_to_png::Color::new(r, g, b);
        let png = self
            .renderer
            .render_text_to_png_data(&text, FontSize::FillHeight(7.0), color)
            .expect("Failed to render text");

        let path = format!("{}/{}", ASSET_DIR, file_name);
        std::fs::write(path, &png.data).expect("Failed to write image to file");

        let handle = asset_server.load(&file_name);
        self.cache.insert(file_name, handle.clone());
        return handle;
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
