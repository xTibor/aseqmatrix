use std::fs;
use std::path::Path;

use sdl2::image::LoadTexture;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;
use serde_derive::Deserialize;

use crate::graphics::{TileDimension, TileTexture};

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
    pub controls_texture: TileTexture<'a>,
    pub font_texture: TileTexture<'a>,
}

impl<'a> Theme<'a> {
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>, manifest_path: &Path) -> Result<Theme<'a>, String> {
        // TODO: check width, height mod
        let manifest = toml::from_slice(&fs::read(manifest_path).unwrap()).unwrap();
        let theme_directory = manifest_path.parent().unwrap();

        let background_texture = texture_creator.load_texture(theme_directory.join("background.png"))?;
        let controls_texture = TileTexture::new(
            texture_creator,
            theme_directory.join("controls.png"),
            TileDimension { width: 16, height: 16 },
        )?;
        let font_texture = TileTexture::new(
            texture_creator,
            theme_directory.join("font.png"),
            TileDimension { width: 16, height: 8 },
        )?;

        Ok(Theme {
            manifest,
            background_texture,
            controls_texture,
            font_texture,
        })
    }
}
