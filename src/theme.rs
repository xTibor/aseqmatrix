use std::fs;
use std::path::Path;

use sdl2::image::LoadTexture;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;
use serde_derive::Deserialize;

use crate::graphics::{PixelDimension, TileDimension};

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ThemeMetadata {
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub license: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ThemeMetrics {
    pub window_margin: usize,
    pub label_spacing: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ThemeManifest {
    #[serde(rename = "theme")]
    pub metadata: ThemeMetadata,
    pub metrics: ThemeMetrics,
}

pub struct Theme<'a> {
    pub manifest: ThemeManifest,

    pub background_texture: Texture<'a>,

    pub controls_texture: Texture<'a>,
    pub controls_tiles_per_dimension: TileDimension,
    pub controls_tile_size: PixelDimension,

    pub font_texture: Texture<'a>,
    pub font_tiles_per_dimension: TileDimension,
    pub font_tile_size: PixelDimension,
}

impl<'a> Theme<'a> {
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>, manifest_path: &Path) -> Result<Theme<'a>, String> {
        // TODO: check width, height mod
        let manifest = toml::from_slice(&fs::read(manifest_path).unwrap()).unwrap();
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

        Ok(Theme {
            manifest,

            background_texture,

            controls_texture,
            controls_tiles_per_dimension,
            controls_tile_size,

            font_texture,
            font_tiles_per_dimension,
            font_tile_size,
        })
    }
}
