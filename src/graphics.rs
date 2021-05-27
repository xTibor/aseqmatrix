use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

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

pub fn draw_tiles(
    canvas: &mut Canvas<Window>,
    texture: &Texture,
    tile_size: PixelDimension,
    source: TileRect,
    target: PixelPosition,
) -> Result<(), String> {
    let source_rect = Rect::new(
        source.x as i32 * tile_size.width as i32,
        source.y as i32 * tile_size.height as i32,
        source.width as u32 * tile_size.width as u32,
        source.height as u32 * tile_size.height as u32,
    );

    let target_rect = Rect::new(
        target.x as i32,
        target.y as i32,
        source.width as u32 * tile_size.width as u32,
        source.height as u32 * tile_size.height as u32,
    );

    canvas.copy(&texture, source_rect, target_rect)?;

    Ok(())
}

pub fn draw_character(
    canvas: &mut Canvas<Window>,
    texture: &Texture,
    tile_size: PixelDimension,
    tiles_per_dimension: TileDimension,
    character: char,
    target: PixelPosition,
    rotation: usize,
) -> Result<(), String> {
    let source_rect = {
        let tile_index = if (character <= '\u{001F}') || (character >= '\u{0080}') {
            0x7F
        } else {
            character as usize
        };
        let tile_position = TilePosition {
            x: tile_index % tiles_per_dimension.width,
            y: tile_index / tiles_per_dimension.width,
        };
        Rect::new(
            tile_position.x as i32 * tile_size.width as i32,
            tile_position.y as i32 * tile_size.height as i32,
            tile_size.width as u32,
            tile_size.height as u32,
        )
    };

    let target_rect = Rect::new(
        target.x as i32,
        target.y as i32,
        tile_size.width as u32,
        tile_size.height as u32,
    );

    canvas.copy_ex(
        &texture,
        source_rect,
        target_rect,
        90.0 * rotation as f64,
        Point::new(0, 0),
        false,
        false,
    )?;

    Ok(())
}

pub fn draw_string(
    canvas: &mut Canvas<Window>,
    texture: &Texture,
    tile_size: PixelDimension,
    tiles_per_dimension: TileDimension,
    string: &str,
    target: PixelPosition,
    rotation: usize,
) -> Result<(), String> {
    let (dx, dy) = match rotation {
        0 => (tile_size.width as isize, 0),
        1 => (0, tile_size.width as isize),
        2 => (-(tile_size.width as isize), 0),
        3 => (0, -(tile_size.width as isize)),
        _ => unreachable!(),
    };

    for (index, character) in string.chars().enumerate() {
        draw_character(
            canvas,
            texture,
            tile_size,
            tiles_per_dimension,
            character,
            PixelPosition {
                x: target.x + index as isize * dx,
                y: target.y + index as isize * dy,
            },
            rotation,
        )?;
    }
    Ok(())
}

pub fn draw_tiled_background(canvas: &mut Canvas<Window>, texture: &Texture) -> Result<(), String> {
    let (canvas_width, canvas_height) = canvas.output_size()?;
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
            canvas.copy(texture, source_rect, target_rect)?;
        }
    }

    Ok(())
}
