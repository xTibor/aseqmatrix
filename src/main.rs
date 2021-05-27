use alsa::seq::{Addr, ClientIter, PortInfo, PortIter, PortSubscribe, PortSubscribeIter, QuerySubsType, Seq};
use alsa::seq::{PortCap, PortType};
use alsa::PollDescriptors;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::ffi::CString;
use std::sync::{Arc, Mutex};
use std::thread;

mod graphics;
use graphics::{draw_string, draw_tiled_background, draw_tiles, PixelDimension, PixelPosition, TilePosition};

mod skin;
use skin::Skin;

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

    fn render(&self, canvas: &mut Canvas<Window>, skin: &Skin) {
        canvas.set_draw_color(pixels::Color::RGB(128, 128, 128));
        canvas.clear();

        draw_tiled_background(canvas, &skin.background_texture).unwrap();

        let button_dimensions = PixelDimension(skin.controls_tile_size.0 * 2, skin.controls_tile_size.1 * 2);
        let (horizontal_arrow_width, vertical_arrow_height) =
            (skin.controls_tile_size.0 as isize, skin.controls_tile_size.0 as isize);

        for (output_index, (_, output_name)) in self.outputs.iter().enumerate() {
            let arrow_source = if match self.selection {
                Some((_, selection_y)) => selection_y == output_index,
                _ => false,
            } {
                TilePosition(5, 2)
            } else {
                TilePosition(5, 0)
            };

            let arrow_position = PixelPosition(
                skin.window_margin as isize + self.inputs.len() as isize * button_dimensions.0 as isize,
                skin.window_margin as isize + output_index as isize * button_dimensions.1 as isize,
            );

            let text_position = PixelPosition(
                arrow_position.0 + horizontal_arrow_width + skin.label_spacing as isize,
                arrow_position.1 + (button_dimensions.1 as isize - skin.font_tile_size.1 as isize) / 2,
            );

            draw_tiles(
                canvas,
                &skin.controls_texture,
                skin.controls_tile_size,
                arrow_source,
                arrow_position,
                1,
                2,
            )
            .unwrap();

            draw_string(
                canvas,
                &skin.font_texture,
                skin.font_tile_size,
                skin.font_tiles_per_dimension,
                output_name,
                text_position,
                0,
            )
            .unwrap();
        }

        for (input_index, (_, input_name)) in self.inputs.iter().enumerate() {
            let arrow_source = if match self.selection {
                Some((selection_x, _)) => selection_x == input_index,
                _ => false,
            } {
                TilePosition(6, 3)
            } else {
                TilePosition(6, 1)
            };

            let arrow_position = PixelPosition(
                skin.window_margin as isize + input_index as isize * button_dimensions.0 as isize,
                skin.window_margin as isize + self.outputs.len() as isize * button_dimensions.1 as isize,
            );

            let text_position = PixelPosition(
                arrow_position.0 + (button_dimensions.0 as isize - skin.font_tile_size.0 as isize) / 2,
                arrow_position.1
                    + vertical_arrow_height
                    + skin.label_spacing as isize
                    + input_name.len() as isize * skin.font_tile_size.0 as isize,
            );

            draw_tiles(
                canvas,
                &skin.controls_texture,
                skin.controls_tile_size,
                arrow_source,
                arrow_position,
                2,
                1,
            )
            .unwrap();

            draw_string(
                canvas,
                &skin.font_texture,
                skin.font_tile_size,
                skin.font_tiles_per_dimension,
                input_name,
                text_position,
                3,
            )
            .unwrap();
        }

        for (output_index, (output_addr, _)) in self.outputs.iter().enumerate() {
            for (input_index, (input_addr, _)) in self.inputs.iter().enumerate() {
                let button_source = if self.connections.contains(&(*input_addr, *output_addr)) {
                    TilePosition(0, 2)
                } else {
                    TilePosition(0, 0)
                };

                let button_position = PixelPosition(
                    skin.window_margin as isize + input_index as isize * button_dimensions.0 as isize,
                    skin.window_margin as isize + output_index as isize * button_dimensions.1 as isize,
                );

                draw_tiles(
                    canvas,
                    &skin.controls_texture,
                    skin.controls_tile_size,
                    button_source,
                    button_position,
                    2,
                    2,
                )
                .unwrap();
            }
        }

        canvas.present();
    }

    fn resize_window(&self, canvas: &mut Canvas<Window>, skin: &Skin) {
        let window_width = skin.window_margin
            + self.inputs.len() * (2 * skin.controls_tile_size.0) // Controls
            + skin.controls_tile_size.0 // Arrow
            + skin.label_spacing
            + self.outputs.iter().map(|(_, name)| name.len()).max().unwrap_or(0) * (skin.font_tile_size.0)
            + skin.window_margin;

        let window_height = skin.window_margin
            + self.outputs.len() * (2 * skin.controls_tile_size.1) // Controls
            + skin.controls_tile_size.1 // Arrow
            + skin.label_spacing
            + self.inputs.iter().map(|(_, name)| name.len()).max().unwrap_or(0) * (skin.font_tile_size.0)
            + skin.window_margin;

        let window = canvas.window_mut();
        window.set_size(window_width as u32, window_height as u32).unwrap();
    }

    fn control_under_position(&self, skin: &Skin, position: PixelPosition) -> Option<(usize, usize)> {
        let (px, py) = (
            position.0 - skin.window_margin as isize,
            position.1 - skin.window_margin as isize,
        );

        if (px < 0) || (py < 0) {
            return None;
        }

        let (control_x, control_y) = (
            px as usize / (skin.controls_tile_size.0 * 2),
            py as usize / (skin.controls_tile_size.1 * 2),
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
    let mut skin = Skin::new(&texture_creator, "amber")?;

    {
        let app = app.lock().unwrap();
        app.resize_window(&mut canvas, &skin);
        // This double rendering is a workaround for the screen corruption after
        // resizing the window. Double buffering seems to be fucked in SDL2.
        app.render(&mut canvas, &skin);
        app.render(&mut canvas, &skin);
    }

    let sdl_event = sdl_context.event()?;
    sdl_event.register_custom_event::<MidiPortChangeEvent>().unwrap();
    sdl_event.push_custom_event(MidiPortChangeEvent).unwrap();
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
                    println!("{:?}", event);
                    continue;
                }
                println!("poll");
                alsa::poll::poll(&mut fds, -1).unwrap();
            }
        });
    }

    let mut events = sdl_context.event_pump()?;
    'main: loop {
        for event in events.wait_iter() {
            println!("{:?}", event);
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

                    app.selection = app.control_under_position(&skin, PixelPosition(x as isize, y as isize));
                    println!("{:?}", app.selection);

                    if app.selection != last_selection {
                        app.render(&mut canvas, &skin);
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
                    app.resize_window(&mut canvas, &skin);
                    app.render(&mut canvas, &skin);
                    app.render(&mut canvas, &skin);
                }

                Event::KeyDown {
                    keycode: Some(Keycode::F12),
                    ..
                } => {
                    let app = app.lock().unwrap();
                    skin = Skin::new(&texture_creator, "test")?;
                    app.resize_window(&mut canvas, &skin);
                    app.render(&mut canvas, &skin);
                    app.render(&mut canvas, &skin);
                }

                _ => {}
            }
        }
    }

    Ok(())
}
