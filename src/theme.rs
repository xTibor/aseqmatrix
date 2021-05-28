use std::path::Path;

use sdl2::{
    image::LoadTexture,
    render::{Texture, TextureCreator},
    video::WindowContext,
};

use crate::graphics::{PixelDimension, TileDimension};

pub struct Theme<'a> {
    pub background_texture: Texture<'a>,

    pub controls_texture: Texture<'a>,
    pub controls_tiles_per_dimension: TileDimension,
    pub controls_tile_size: PixelDimension,

    pub font_texture: Texture<'a>,
    pub font_tiles_per_dimension: TileDimension,
    pub font_tile_size: PixelDimension,

    pub window_margin: usize,
    pub label_spacing: usize,
}

impl<'a> Theme<'a> {
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>, manifest_path: &Path) -> Result<Theme<'a>, String> {
        // TODO: sanitize paths to avoid path traversal vulns
        // TODO: check width, height mod
        let theme_directory = manifest_path.parent().unwrap();

        let background_path = theme_directory.join("background.png");
        let background_texture = texture_creator.load_texture(background_path)?;

        let controls_path = theme_directory.join("controls.png");
        let controls_texture = texture_creator.load_texture(controls_path)?;
        let controls_tiles_per_dimension = TileDimension { width: 16, height: 16 };
        let controls_tile_size = {
            let query = controls_texture.query();
            PixelDimension {
                width: query.width as usize / controls_tiles_per_dimension.width,
                height: query.height as usize / controls_tiles_per_dimension.height,
            }
        };

        let font_path = theme_directory.join("font.png");
        let font_texture = texture_creator.load_texture(font_path)?;
        let font_tiles_per_dimension = TileDimension { width: 16, height: 8 };
        let font_tile_size = {
            let query = font_texture.query();
            PixelDimension {
                width: query.width as usize / font_tiles_per_dimension.width,
                height: query.height as usize / font_tiles_per_dimension.height,
            }
        };

        let window_margin = 12;
        let label_spacing = 12;

        Ok(Theme {
            background_texture,

            controls_texture,
            controls_tiles_per_dimension,
            controls_tile_size,

            font_texture,
            font_tiles_per_dimension,
            font_tile_size,

            window_margin,
            label_spacing,
        })
    }
}
