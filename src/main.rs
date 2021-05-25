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

    let inputs = [
        "Timer",
        "Announce",
        "Midi Through Port-0",
        "VirMIDI 3-0",
        "VirMIDI 3-1",
        "VirMIDI 3-2",
        "VirMIDI 3-3",
    ];
    let outputs = [
        "Midi Through Port-0",
        "VirMIDI 3-0",
        "VirMIDI 3-1",
        "VirMIDI 3-2",
        "VirMIDI 3-3",
        "TiMidity port 0",
        "TiMidity port 1",
        "TiMidity port 2",
        "TiMidity port 3",
        "BinaryPiano2",
        "aseqdump",
    ];

    {
        let window_width = (1 + inputs.len() * 2 + 1 + outputs.iter().map(|s| s.len()).max().unwrap_or(0) + 2) * TILE_SIZE;
        let window_height = (1 + outputs.len() * 2 + 1 + inputs.iter().map(|s| s.len()).max().unwrap_or(0) + 2) * TILE_SIZE;
        let window = canvas.window_mut();
        window.set_size(window_width as u32, window_height as u32).map_err(|e| e.to_string())?;
    }

    for (output_index, output) in outputs.iter().enumerate() {
        let y = 1 + output_index as isize * 2;
        let x_arrow_right = inputs.len() as isize * 2 + 1;
        let x_text = x_arrow_right + 2;

        draw_tiles(&mut canvas, &texture, (2, 0), (x_arrow_right, y), 1, 2)?;
        draw_string(&mut canvas, &texture, output, (x_text, y), 0)?;
    }

    for (input_index, input) in inputs.iter().enumerate() {
        let x = 1 + input_index as isize * 2;
        let y_arrow_down = outputs.len() as isize * 2 + 1;
        let y_text = y_arrow_down + input.len() as isize + 1;

        draw_tiles(&mut canvas, &texture, (0, 3), (x, y_arrow_down), 2, 1)?;
        draw_string(&mut canvas, &texture, input, (x, y_text), 3)?;
    }

    for y in 0..outputs.len() {
        for x in 0..inputs.len() {
            draw_tiles(&mut canvas, &texture, (0, 0), (1 + x as isize * 2, 1 + y as isize * 2), 2, 2)?;
        }
    }

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
