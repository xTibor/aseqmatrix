use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

pub const TILE_SIZE: usize = 12;
pub const TILES_PER_LINE: usize = 16;

pub fn draw_tile(
    canvas: &mut Canvas<Window>,
    texture: &Texture,
    source: (isize, isize),
    target: (isize, isize),
    rotation: usize,
) -> Result<(), String> {
    let source_rect = Rect::new(
        source.0 as i32 * TILE_SIZE as i32,
        source.1 as i32 * TILE_SIZE as i32,
        TILE_SIZE as u32,
        TILE_SIZE as u32,
    );
    let target_rect = Rect::new(
        target.0 as i32 * TILE_SIZE as i32,
        target.1 as i32 * TILE_SIZE as i32,
        TILE_SIZE as u32,
        TILE_SIZE as u32,
    );
    canvas.copy_ex(
        &texture,
        source_rect,
        target_rect,
        90.0 * rotation as f64,
        None,
        false,
        false,
    )?;
    Ok(())
}

pub fn draw_tiles(
    canvas: &mut Canvas<Window>,
    texture: &Texture,
    source: (isize, isize),
    target: (isize, isize),
    width: usize,
    height: usize,
) -> Result<(), String> {
    for dy in 0..height {
        for dx in 0..width {
            draw_tile(
                canvas,
                texture,
                (source.0 + dx as isize, source.1 + dy as isize),
                (target.0 + dx as isize, target.1 + dy as isize),
                0,
            )?;
        }
    }
    Ok(())
}

pub fn draw_character(
    canvas: &mut Canvas<Window>,
    texture: &Texture,
    character: char,
    target: (isize, isize),
    rotation: usize,
) -> Result<(), String> {
    let source = {
        let tmp = if (character <= '\u{001F}') || (character >= '\u{0080}') {
            0x7F
        } else {
            character as usize
        };

        ((tmp % TILES_PER_LINE) as isize, (tmp / TILES_PER_LINE) as isize * 2)
    };

    let (dx, dy) = match rotation {
        0 => (0, 1),
        1 => (-1, 0),
        2 => (0, -1),
        3 => (1, 0),
        _ => unreachable!(),
    };

    draw_tile(canvas, texture, source, target, rotation)?;
    draw_tile(
        canvas,
        texture,
        (source.0, source.1 + 1),
        (target.0 + dx, target.1 + dy),
        rotation,
    )?;
    Ok(())
}

pub fn draw_string(
    canvas: &mut Canvas<Window>,
    texture: &Texture,
    string: &str,
    target: (isize, isize),
    rotation: usize,
) -> Result<(), String> {
    let (dx, dy) = match rotation {
        0 => (1, 0),
        1 => (0, 1),
        2 => (-1, 0),
        3 => (0, -1),
        _ => unreachable!(),
    };

    for (index, character) in string.chars().enumerate() {
        draw_character(
            canvas,
            texture,
            character,
            (target.0 + index as isize * dx, target.1 + index as isize * dy),
            rotation,
        )?;
    }
    Ok(())
}

pub fn draw_tiled_background(
    canvas: &mut Canvas<Window>,
    texture: &Texture,
)-> Result<(), String> {
    let (canvas_width, canvas_height) = canvas.output_size()?;
    let (texture_width, texture_height) = {
        let query = texture.query();
        (query.width, query.height)
    };

    let source_rect = Rect::new(0, 0, texture_width, texture_height);
    for x in 0..canvas_width / texture_width + 1 {
        for y in 0..canvas_height / texture_height + 1 {
            let target_rect = Rect::new(texture_width as i32 * x as i32, texture_height as i32 * y as i32, texture_width, texture_height);
            canvas.copy(texture, source_rect, target_rect)?;
        }
    }

    Ok(())
}
