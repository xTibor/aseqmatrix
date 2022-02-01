use std::fs;
use std::path::{Path, PathBuf};

use sdl2::image::LoadTexture;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;
use serde_derive::Deserialize;

use crate::error::{sdl_error, Error};
use crate::graphics::{TileDimension, TileRect, TileTexture};

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
    pub const RECT_ARROW_LEFT_NORMAL: TileRect = TileRect { x: 5, y: 0, width: 1, height: 2 };
    pub const RECT_ARROW_LEFT_ACTIVE: TileRect = TileRect { x: 5, y: 2, width: 1, height: 2 };

    pub const RECT_ARROW_DOWN_NORMAL: TileRect = TileRect { x: 6, y: 1, width: 2, height: 1 };
    pub const RECT_ARROW_DOWN_ACTIVE: TileRect = TileRect { x: 6, y: 3, width: 2, height: 1 };

    pub const RECT_BUTTON_NORMAL: TileRect = TileRect { x: 0, y: 0, width: 2, height: 2 };
    pub const RECT_BUTTON_ACTIVE: TileRect = TileRect { x: 0, y: 2, width: 2, height: 2 };
    pub const RECT_BUTTON_DISABLED: TileRect = TileRect { x: 0, y: 4, width: 2, height: 2 };
    pub const RECT_BUTTON_NORMAL_DOWN: TileRect = TileRect { x: 2, y: 0, width: 2, height: 2 };
    pub const RECT_BUTTON_ACTIVE_DOWN: TileRect = TileRect { x: 2, y: 2, width: 2, height: 2 };
    pub const RECT_BUTTON_DISABLED_DOWN: TileRect = TileRect { x: 2, y: 4, width: 2, height: 2 };

    pub fn new(texture_creator: &'a TextureCreator<WindowContext>, manifest_path: &Path) -> Result<Theme<'a>, Error> {
        // TODO: check width, height mod
        let manifest = toml::from_slice(&fs::read(&manifest_path)?)?;
        let theme_directory = manifest_path
            .parent()
            .ok_or(Error::GeneralError("failed to retrieve parent directory of theme manifest path"))?;

        let background_texture =
            texture_creator.load_texture(theme_directory.join("background.png")).map_err(sdl_error)?;

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

        Ok(Theme { manifest, background_texture, controls_texture, font_texture })
    }

    pub fn theme_manifest_paths() -> Result<Vec<PathBuf>, Error> {
        let themes_directory_builtin = "themes";
        let themes_directory_user = dirs::data_dir()
            .ok_or(Error::GeneralError("failed to retrieve data directory"))?
            .join("aseqmatrix")
            .join("themes");

        let mut manifest_paths = fs::read_dir(themes_directory_builtin)?
            .chain(fs::read_dir(themes_directory_user)?)
            .filter_map(Result::ok)
            .filter(|direntry| direntry.file_type().map(|filetype| filetype.is_dir()).unwrap_or(false))
            .map(|direntry| direntry.path().join("theme.toml"))
            .filter(|path| path.exists())
            .collect::<Vec<PathBuf>>();
        manifest_paths.sort();

        Ok(manifest_paths)
    }
}
