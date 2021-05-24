use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::pixels;
use sdl2::{
    event::Event,
    rect::Rect,
    render::{Canvas, RenderTarget, Texture},
};

const TILE_WIDTH: usize = 12;
const TILE_HEIGHT: usize = 12;
const TILES_PER_LINE: usize = 16;

fn draw_tile<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    texture: &Texture,
    source: (usize, usize),
    target: (usize, usize),
) -> Result<(), String> {
    let source_rect = Rect::new(
        source.0 as i32 * TILE_WIDTH as i32,
        source.1 as i32 * TILE_HEIGHT as i32,
        TILE_WIDTH as u32,
        TILE_HEIGHT as u32,
    );
    let target_rect = Rect::new(
        target.0 as i32 * TILE_WIDTH as i32,
        target.1 as i32 * TILE_HEIGHT as i32,
        TILE_WIDTH as u32,
        TILE_HEIGHT as u32,
    );
    canvas.copy(&texture, source_rect, target_rect)?;
    Ok(())
}

fn draw_character<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    texture: &Texture,
    character: char,
    target: (usize, usize),
) -> Result<(), String> {
    let source = {
        let tmp = if (character <= '\u{001F}') || (character >= '\u{0080}') {
            0x7F
        } else {
            character as usize
        };

        (tmp % TILES_PER_LINE, (tmp / TILES_PER_LINE) * 2)
    };

    draw_tile(canvas, texture, source, target)?;
    draw_tile(
        canvas,
        texture,
        (source.0, source.1 + 1),
        (target.0, target.1 + 1),
    )?;
    Ok(())
}

fn draw_string<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    texture: &Texture,
    string: &str,
    target: (usize, usize),
) -> Result<(), String> {
    for (index, character) in string.chars().enumerate() {
        draw_character(canvas, texture, character, (target.0 + index, target.1))?;
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
    draw_string(&mut canvas, &texture, "Hello, world!", (1, 1))?;
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
