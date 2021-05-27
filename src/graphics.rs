use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

#[derive(Copy, Clone, Debug)]
pub struct PixelPosition(pub isize, pub isize);

#[derive(Copy, Clone, Debug)]
pub struct PixelDimension(pub usize, pub usize);

#[derive(Copy, Clone, Debug)]
pub struct TilePosition(pub usize, pub usize);

#[derive(Copy, Clone, Debug)]
pub struct TileDimension(pub usize, pub usize);

pub fn draw_tiles(
    canvas: &mut Canvas<Window>,
    texture: &Texture,
    tile_size: PixelDimension,
    source: TilePosition,
    target: PixelPosition,
    width: usize,
    height: usize,
) -> Result<(), String> {
    let source_rect = Rect::new(
        source.0 as i32 * tile_size.0 as i32,
        source.1 as i32 * tile_size.1 as i32,
        width as u32 * tile_size.0 as u32,
        height as u32 * tile_size.1 as u32,
    );

    let target_rect = Rect::new(
        target.0 as i32,
        target.1 as i32,
        width as u32 * tile_size.0 as u32,
        height as u32 * tile_size.1 as u32,
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
        let tile_position = TilePosition(tile_index % tiles_per_dimension.0, tile_index / tiles_per_dimension.0);
        Rect::new(
            tile_position.0 as i32 * tile_size.0 as i32,
            tile_position.1 as i32 * tile_size.1 as i32,
            tile_size.0 as u32,
            tile_size.1 as u32,
        )
    };

    let target_rect = Rect::new(target.0 as i32, target.1 as i32, tile_size.0 as u32, tile_size.1 as u32);

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
        0 => (tile_size.0 as isize, 0),
        1 => (0, tile_size.0 as isize),
        2 => (-(tile_size.0 as isize), 0),
        3 => (0, -(tile_size.0 as isize)),
        _ => unreachable!(),
    };

    for (index, character) in string.chars().enumerate() {
        draw_character(
            canvas,
            texture,
            tile_size,
            tiles_per_dimension,
            character,
            PixelPosition(target.0 + index as isize * dx, target.1 + index as isize * dy),
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
