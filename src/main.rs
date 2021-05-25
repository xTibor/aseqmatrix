use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::pixels;
use sdl2::{
    event::Event,
    rect::Rect,
    render::{Canvas, RenderTarget, Texture},
};

const TILE_SIZE: usize = 12;
const TILES_PER_LINE: usize = 16;

fn draw_tile<T: RenderTarget>(
    canvas: &mut Canvas<T>,
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

fn draw_tiles<T: RenderTarget>(
    canvas: &mut Canvas<T>,
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

fn draw_character<T: RenderTarget>(
    canvas: &mut Canvas<T>,
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

        (
            (tmp % TILES_PER_LINE) as isize,
            (tmp / TILES_PER_LINE) as isize * 2,
        )
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

fn draw_string<T: RenderTarget>(
    canvas: &mut Canvas<T>,
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
            (
                target.0 + index as isize * dx,
                target.1 + index as isize * dy,
            ),
            rotation,
        )?;
    }
    Ok(())
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let window = video_subsys
        .window("MIDI Matrix", 640, 480)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator.load_texture("assets/tileset.png")?;

    canvas.set_draw_color(pixels::Color::RGB(128, 128, 128));
    canvas.clear();
    draw_string(&mut canvas, &texture, "Hello, world!", (10, 10), 0)?;
    draw_string(&mut canvas, &texture, "Hello, world!", (10, 10), 1)?;
    draw_string(&mut canvas, &texture, "Hello, world!", (10, 10), 2)?;
    draw_string(&mut canvas, &texture, "Hello, world!", (10, 10), 3)?;

    draw_tiles(&mut canvas, &texture, (0, 0), (10, 0), 2, 2)?;
    draw_tiles(&mut canvas, &texture, (3, 0), (12, 0), 2, 2)?;
    draw_tiles(&mut canvas, &texture, (6, 0), (14, 0), 2, 2)?;

    canvas.present();

    let mut events = sdl_context.event_pump()?;

    'main: loop {
        for event in events.wait_iter() {
            println!("{:?}", event);
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main,
                _ => {}
            }
        }
    }

    Ok(())
}
