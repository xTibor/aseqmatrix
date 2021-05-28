use std::ffi::CString;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use alsa::seq::{
    Addr, ClientIter, PortCap, PortInfo, PortIter, PortSubscribe, PortSubscribeIter, PortType, QuerySubsType, Seq,
};
use alsa::PollDescriptors;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::Canvas;
use sdl2::video::Window;

mod graphics;
use graphics::{draw_string, draw_tiled_background, draw_tiles, PixelDimension, PixelPosition, TileRect};

mod theme;
use theme::Theme;

struct MidiPortChangeEvent;

struct MidiMatrixApp {
    inputs: Vec<(Addr, String)>,
    outputs: Vec<(Addr, String)>,
    connections: Vec<(Addr, Addr)>,
    selection: Option<(usize, usize)>,
}

impl MidiMatrixApp {
    fn new() -> MidiMatrixApp {
        MidiMatrixApp {
            inputs: Vec::new(),
            outputs: Vec::new(),
            connections: Vec::new(),
            selection: None,
        }
    }

    fn render(&self, canvas: &mut Canvas<Window>, theme: &Theme) -> Result<(), String> {
        draw_tiled_background(canvas, &theme.background_texture)?;

        let button_dimensions = PixelDimension {
            width: theme.controls_tile_size.width * 2,
            height: theme.controls_tile_size.height * 2,
        };

        let (horizontal_arrow_width, vertical_arrow_height) = (
            theme.controls_tile_size.width as isize,
            theme.controls_tile_size.height as isize,
        );

        for (output_index, (_, output_name)) in self.outputs.iter().enumerate() {
            let arrow_source = if match self.selection {
                Some((_, selection_y)) => selection_y == output_index,
                _ => false,
            } {
                TileRect {
                    x: 5,
                    y: 2,
                    width: 1,
                    height: 2,
                }
            } else {
                TileRect {
                    x: 5,
                    y: 0,
                    width: 1,
                    height: 2,
                }
            };

            let arrow_position = PixelPosition {
                x: theme.manifest.metrics.window_margin as isize
                    + self.inputs.len() as isize * button_dimensions.width as isize,
                y: theme.manifest.metrics.window_margin as isize
                    + output_index as isize * button_dimensions.height as isize,
            };

            let text_position = PixelPosition {
                x: arrow_position.x + horizontal_arrow_width + theme.manifest.metrics.label_spacing as isize,
                y: arrow_position.y + (button_dimensions.height as isize - theme.font_tile_size.height as isize) / 2,
            };

            draw_tiles(
                canvas,
                &theme.controls_texture,
                theme.controls_tile_size,
                arrow_source,
                arrow_position,
            )?;

            draw_string(
                canvas,
                &theme.font_texture,
                theme.font_tile_size,
                theme.font_tiles_per_dimension,
                output_name,
                text_position,
                0,
            )?;
        }

        for (input_index, (_, input_name)) in self.inputs.iter().enumerate() {
            let arrow_source = if match self.selection {
                Some((selection_x, _)) => selection_x == input_index,
                _ => false,
            } {
                TileRect {
                    x: 6,
                    y: 3,
                    width: 2,
                    height: 1,
                }
            } else {
                TileRect {
                    x: 6,
                    y: 1,
                    width: 2,
                    height: 1,
                }
            };

            let arrow_position = PixelPosition {
                x: theme.manifest.metrics.window_margin as isize
                    + input_index as isize * button_dimensions.width as isize,
                y: theme.manifest.metrics.window_margin as isize
                    + self.outputs.len() as isize * button_dimensions.height as isize,
            };

            let text_position = PixelPosition {
                x: arrow_position.x + (button_dimensions.width as isize - theme.font_tile_size.width as isize) / 2,
                y: arrow_position.y
                    + vertical_arrow_height
                    + theme.manifest.metrics.label_spacing as isize
                    + input_name.len() as isize * theme.font_tile_size.width as isize,
            };

            draw_tiles(
                canvas,
                &theme.controls_texture,
                theme.controls_tile_size,
                arrow_source,
                arrow_position,
            )?;

            draw_string(
                canvas,
                &theme.font_texture,
                theme.font_tile_size,
                theme.font_tiles_per_dimension,
                input_name,
                text_position,
                3,
            )?;
        }

        for (output_index, (output_addr, _)) in self.outputs.iter().enumerate() {
            for (input_index, (input_addr, _)) in self.inputs.iter().enumerate() {
                let button_source = if self.connections.contains(&(*input_addr, *output_addr)) {
                    TileRect {
                        x: 0,
                        y: 2,
                        width: 2,
                        height: 2,
                    }
                } else {
                    TileRect {
                        x: 0,
                        y: 0,
                        width: 2,
                        height: 2,
                    }
                };

                let button_position = PixelPosition {
                    x: theme.manifest.metrics.window_margin as isize
                        + input_index as isize * button_dimensions.width as isize,
                    y: theme.manifest.metrics.window_margin as isize
                        + output_index as isize * button_dimensions.height as isize,
                };

                draw_tiles(
                    canvas,
                    &theme.controls_texture,
                    theme.controls_tile_size,
                    button_source,
                    button_position,
                )?;
            }
        }

        canvas.present();
        Ok(())
    }

    fn resize_window(&self, canvas: &mut Canvas<Window>, theme: &Theme) -> Result<(), String> {
        let window_width = theme.manifest.metrics.window_margin
            + self.inputs.len() * (2 * theme.controls_tile_size.width) // Controls
            + theme.controls_tile_size.width // Arrow
            + theme.manifest.metrics.label_spacing
            + self.outputs.iter().map(|(_, name)| name.len()).max().unwrap_or(0) * (theme.font_tile_size.width)
            + theme.manifest.metrics.window_margin;

        let window_height = theme.manifest.metrics.window_margin
            + self.outputs.len() * (2 * theme.controls_tile_size.height) // Controls
            + theme.controls_tile_size.height // Arrow
            + theme.manifest.metrics.label_spacing
            + self.inputs.iter().map(|(_, name)| name.len()).max().unwrap_or(0) * (theme.font_tile_size.width)
            + theme.manifest.metrics.window_margin;

        let window = canvas.window_mut();
        window
            .set_size(window_width as u32, window_height as u32)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    fn control_under_position(&self, theme: &Theme, position: PixelPosition) -> Option<(usize, usize)> {
        let (px, py) = (
            position.x - theme.manifest.metrics.window_margin as isize,
            position.y - theme.manifest.metrics.window_margin as isize,
        );

        if (px < 0) || (py < 0) {
            return None;
        }

        let (control_x, control_y) = (
            px as usize / (theme.controls_tile_size.width * 2),
            py as usize / (theme.controls_tile_size.height * 2),
        );

        if (control_x < self.inputs.len()) && (control_y < self.outputs.len()) {
            Some((control_x, control_y))
        } else {
            None
        }
    }
}

fn main() -> Result<(), String> {
    let app = Arc::new(Mutex::new(MidiMatrixApp::new()));

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
    let mut theme = Theme::new(&texture_creator, Path::new("themes/amber/theme.toml"))?;

    {
        let app = app.lock().unwrap();
        app.resize_window(&mut canvas, &theme)?;
        // This double rendering is a workaround for the screen corruption after
        // resizing the window. Double buffering seems to be fucked in SDL2.
        app.render(&mut canvas, &theme)?;
        app.render(&mut canvas, &theme)?;
    }

    let sdl_event = sdl_context.event()?;
    sdl_event.register_custom_event::<MidiPortChangeEvent>()?;
    sdl_event.push_custom_event(MidiPortChangeEvent)?;
    let tx = sdl_event.event_sender();

    {
        let app = Arc::clone(&app);
        thread::spawn(move || {
            let mut seq = Seq::open(None, None, false).unwrap();

            let midi_name = CString::new("MIDI Matrix").unwrap();
            seq.set_client_name(&midi_name).unwrap();

            let client_port = {
                let mut port_info = PortInfo::empty().unwrap();
                port_info.set_capability(PortCap::WRITE);
                port_info.set_type(PortType::MIDI_GENERIC | PortType::APPLICATION);
                port_info.set_name(&midi_name);
                seq.create_port(&port_info).unwrap();
                port_info.addr()
            };

            {
                let mut sub = PortSubscribe::empty().unwrap();
                sub.set_sender(Addr::system_announce());
                sub.set_dest(client_port);
                seq.subscribe_port(&sub).unwrap();
            }

            let refresh_midi_endpoints = || {
                let mut app = app.lock().unwrap();
                app.inputs.clear();
                app.outputs.clear();
                app.connections.clear();

                for client in ClientIter::new(&seq) {
                    for port in PortIter::new(&seq, client.get_client()) {
                        if port.get_capability().contains(PortCap::SUBS_READ) {
                            app.inputs.push((port.addr(), port.get_name().unwrap().to_owned()))
                        }

                        if port.get_capability().contains(PortCap::SUBS_WRITE) {
                            app.outputs.push((port.addr(), port.get_name().unwrap().to_owned()))
                        }

                        for sub in PortSubscribeIter::new(&seq, port.addr(), QuerySubsType::WRITE) {
                            app.connections.push((sub.get_sender(), sub.get_dest()))
                        }
                    }
                }
            };

            refresh_midi_endpoints();

            let mut fds = Vec::<alsa::poll::pollfd>::new();
            fds.append(&mut (&seq, Some(alsa::Direction::Capture)).get().unwrap());

            let mut seq_input = seq.input();
            loop {
                if seq_input.event_input_pending(true).unwrap() > 0 {
                    let event = seq_input.event_input().unwrap();

                    // TODO: filter events from this client
                    match event.get_type() {
                        alsa::seq::EventType::PortChange
                        | alsa::seq::EventType::PortExit
                        | alsa::seq::EventType::PortStart
                        | alsa::seq::EventType::PortSubscribed
                        | alsa::seq::EventType::PortUnsubscribed => {
                            refresh_midi_endpoints();
                            tx.push_custom_event(MidiPortChangeEvent).unwrap();
                        }
                        _ => {}
                    }
                    continue;
                }
                alsa::poll::poll(&mut fds, -1).unwrap();
            }
        });
    }

    let mut events = sdl_context.event_pump()?;
    'main: loop {
        for event in events.wait_iter() {
            //println!("{:?}", event);
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'main;
                }
                Event::MouseMotion { x, y, .. } => {
                    let mut app = app.lock().unwrap();

                    let last_selection = app.selection;
                    app.selection = app.control_under_position(
                        &theme,
                        PixelPosition {
                            x: x as isize,
                            y: y as isize,
                        },
                    );

                    if app.selection != last_selection {
                        app.render(&mut canvas, &theme)?;
                    }
                }
                Event::MouseButtonUp { .. } => {
                    let app = app.lock().unwrap();
                    if let Some((selection_x, selection_y)) = app.selection {
                        // assert!(selection in bounds)
                        let input_addr = app.inputs[selection_x].0;
                        let output_addr = app.outputs[selection_y].0;

                        let seq = Seq::open(None, None, false).unwrap();
                        if app.connections.contains(&(input_addr, output_addr)) {
                            seq.unsubscribe_port(input_addr, output_addr).unwrap();
                        } else {
                            let mut sub = PortSubscribe::empty().unwrap();
                            sub.set_sender(input_addr);
                            sub.set_dest(output_addr);
                            seq.subscribe_port(&sub).unwrap();
                        }
                    }
                }

                Event::User { .. } => {
                    // TODO: Check user_event kind
                    let app = app.lock().unwrap();
                    app.resize_window(&mut canvas, &theme)?;
                    app.render(&mut canvas, &theme)?;
                    app.render(&mut canvas, &theme)?;
                }

                Event::KeyDown {
                    keycode: Some(Keycode::F12),
                    ..
                } => {
                    let app = app.lock().unwrap();
                    theme = Theme::new(&texture_creator, Path::new("themes/test/theme.toml"))?;
                    app.resize_window(&mut canvas, &theme)?;
                    app.render(&mut canvas, &theme)?;
                    app.render(&mut canvas, &theme)?;
                }

                _ => {}
            }
        }
    }

    Ok(())
}
