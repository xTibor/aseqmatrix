use std::path::Path;

use sdl2::image::LoadTexture;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

use crate::error::{sdl_error, Error};

#[derive(Copy, Clone, Debug)]
pub struct PixelPosition {
    pub x: isize,
    pub y: isize,
}

#[derive(Copy, Clone, Debug)]
pub struct PixelDimension {
    pub width: usize,
    pub height: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct TilePosition {
    pub x: usize,
    pub y: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct TileDimension {
    pub width: usize,
    pub height: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct TileRect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

pub struct TileTexture<'a> {
    pub texture: Texture<'a>,
    pub tile_size: PixelDimension,
    pub tiles_per_dimension: TileDimension,
}

impl<'a> TileTexture<'a> {
    pub fn new<P: AsRef<Path>>(
        texture_creator: &'a TextureCreator<WindowContext>,
        texture_path: P,
        tiles_per_dimension: TileDimension,
    ) -> Result<TileTexture<'a>, Error> {
        let texture = texture_creator.load_texture(texture_path).map_err(sdl_error)?;

        let tile_size = {
            let query = texture.query();
            PixelDimension {
                width: query.width as usize / tiles_per_dimension.width,
                height: query.height as usize / tiles_per_dimension.height,
            }
        };

        Ok(TileTexture { texture, tile_size, tiles_per_dimension })
    }
}

pub fn draw_tiles(
    canvas: &mut Canvas<Window>,
    tile_texture: &TileTexture,
    source: TileRect,
    target: PixelPosition,
) -> Result<(), Error> {
    let source_rect = Rect::new(
        source.x as i32 * tile_texture.tile_size.width as i32,
        source.y as i32 * tile_texture.tile_size.height as i32,
        source.width as u32 * tile_texture.tile_size.width as u32,
        source.height as u32 * tile_texture.tile_size.height as u32,
    );

    let target_rect = Rect::new(
        target.x as i32,
        target.y as i32,
        source.width as u32 * tile_texture.tile_size.width as u32,
        source.height as u32 * tile_texture.tile_size.height as u32,
    );

    canvas.copy(&tile_texture.texture, source_rect, target_rect).map_err(sdl_error)?;

    Ok(())
}

pub fn draw_character(
    canvas: &mut Canvas<Window>,
    tile_texture: &TileTexture,
    character: char,
    target: PixelPosition,
    rotation: usize,
) -> Result<(), Error> {
    let source_rect = {
        let tile_index = if (character <= '\u{001F}') || (character >= '\u{0080}') { 0x7F } else { character as usize };
        let tile_position = TilePosition {
            x: tile_index % tile_texture.tiles_per_dimension.width,
            y: tile_index / tile_texture.tiles_per_dimension.width,
        };
        Rect::new(
            tile_position.x as i32 * tile_texture.tile_size.width as i32,
            tile_position.y as i32 * tile_texture.tile_size.height as i32,
            tile_texture.tile_size.width as u32,
            tile_texture.tile_size.height as u32,
        )
    };

    let target_rect = Rect::new(
        target.x as i32,
        target.y as i32,
        tile_texture.tile_size.width as u32,
        tile_texture.tile_size.height as u32,
    );

    canvas
        .copy_ex(
            &tile_texture.texture,
            source_rect,
            target_rect,
            90.0 * rotation as f64,
            Point::new(0, 0),
            false,
            false,
        )
        .map_err(sdl_error)?;

    Ok(())
}

pub fn draw_string(
    canvas: &mut Canvas<Window>,
    tile_texture: &TileTexture,
    string: &str,
    target: PixelPosition,
    rotation: usize,
) -> Result<(), Error> {
    let (dx, dy) = match rotation {
        0 => (tile_texture.tile_size.width as isize, 0),
        1 => (0, tile_texture.tile_size.width as isize),
        2 => (-(tile_texture.tile_size.width as isize), 0),
        3 => (0, -(tile_texture.tile_size.width as isize)),
        _ => unreachable!(),
    };

    for (index, character) in string.chars().enumerate() {
        draw_character(
            canvas,
            tile_texture,
            character,
            PixelPosition { x: target.x + index as isize * dx, y: target.y + index as isize * dy },
            rotation,
        )?;
    }
    Ok(())
}

pub fn draw_tiled_background(canvas: &mut Canvas<Window>, texture: &Texture) -> Result<(), Error> {
    let (canvas_width, canvas_height) = canvas.output_size().map_err(sdl_error)?;
    let (texture_width, texture_height) = {
        let query = texture.query();
        (query.width, query.height)
    };

    let source_rect = Rect::new(0, 0, texture_width, texture_height);
    for x in 0..canvas_width / texture_width + 1 {
        for y in 0..canvas_height / texture_height + 1 {
            let target_rect = Rect::new(
                texture_width as i32 * x as i32,
                texture_height as i32 * y as i32,
                texture_width,
                texture_height,
            );
            canvas.copy(texture, source_rect, target_rect).map_err(sdl_error)?;
        }
    }

    Ok(())
}
