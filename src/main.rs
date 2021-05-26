use alsa::seq::{Addr, ClientIter, PortInfo, PortIter, PortSubscribe, PortSubscribeIter, QuerySubsType, Seq};
use alsa::seq::{PortCap, PortType};
use alsa::PollDescriptors;
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::pixels;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use std::ffi::CString;
use std::sync::{Arc, Mutex};
use std::thread;

mod graphics;
use graphics::{TILE_SIZE, draw_string, draw_tiled_background, draw_tiles};

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

    fn render(&self, canvas: &mut Canvas<Window>, texture_foreground: &Texture, texture_background: &Texture) {
        canvas.set_draw_color(pixels::Color::RGB(128, 128, 128));
        canvas.clear();

        draw_tiled_background(canvas, texture_background).unwrap();

        for (output_index, (_, output_name)) in self.outputs.iter().enumerate() {
            let source = if match self.selection {
                Some((_, selection_y)) => selection_y == output_index,
                _ => false,
            } {
                (5, 0)
            } else {
                (2, 0)
            };

            let y = 1 + output_index as isize * 2;

            let x_arrow_right = self.inputs.len() as isize * 2 + 1;
            draw_tiles(canvas, texture_foreground, source, (x_arrow_right, y), 1, 2).unwrap();

            let x_text = x_arrow_right + 2;
            draw_string(canvas, texture_foreground, output_name, (x_text, y), 0).unwrap();
        }

        for (input_index, (_, input_name)) in self.inputs.iter().enumerate() {
            let source = if match self.selection {
                Some((selection_x, _)) => selection_x == input_index,
                _ => false,
            } {
                (3, 3)
            } else {
                (0, 3)
            };

            let x = 1 + input_index as isize * 2;

            let y_arrow_down = self.outputs.len() as isize * 2 + 1;
            draw_tiles(canvas, texture_foreground, source, (x, y_arrow_down), 2, 1).unwrap();

            let y_text = y_arrow_down + input_name.len() as isize + 1;
            draw_string(canvas, texture_foreground, input_name, (x, y_text), 3).unwrap();
        }

        for (output_index, (output_addr, _)) in self.outputs.iter().enumerate() {
            for (input_index, (input_addr, _)) in self.inputs.iter().enumerate() {
                let source = if self.connections.contains(&(*input_addr, *output_addr)) {
                    (3, 0)
                } else {
                    (0, 0)
                };

                draw_tiles(
                    canvas,
                    texture_foreground,
                    source,
                    (1 + input_index as isize * 2, 1 + output_index as isize * 2),
                    2,
                    2,
                )
                .unwrap();
            }
        }

        canvas.present();
    }

    fn resize_window(&self, canvas: &mut Canvas<Window>) {
        let window_width =
            (self.inputs.len() * 2 + self.outputs.iter().map(|(_, name)| name.len()).max().unwrap_or(0) + 4)
                * TILE_SIZE;
        let window_height =
            (self.outputs.len() * 2 + self.inputs.iter().map(|(_, name)| name.len()).max().unwrap_or(0) + 4)
                * TILE_SIZE;
        let window = canvas.window_mut();
        window.set_size(window_width as u32, window_height as u32).unwrap();
    }
}

fn main() -> Result<(), String> {
    let mut app = Arc::new(Mutex::new(MidiMatrixApp::new()));

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
    let texture_foreground = texture_creator.load_texture("skins/amber-foreground.png")?;
    let texture_background = texture_creator.load_texture("skins/amber-background.png")?;

    {
        let app = app.lock().unwrap();
        app.resize_window(&mut canvas);
        app.render(&mut canvas, &texture_foreground, &texture_background);
    }

    let mut sdl_event = sdl_context.event()?;
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

                    // TODO: rounding problems at the top/left edges
                    let (selection_x, selection_y) = (
                        ((x - TILE_SIZE as i32) / TILE_SIZE as i32) / 2,
                        ((y - TILE_SIZE as i32) / TILE_SIZE as i32) / 2
                    );

                    app.selection = if (selection_x >= 0)
                        && (selection_x < app.inputs.len() as i32)
                        && (selection_y >= 0)
                        && (selection_y < app.outputs.len() as i32)
                    {
                        Some((selection_x as usize, selection_y as usize))
                    } else {
                        None
                    };

                    println!("{:?}", app.selection);

                    if app.selection != last_selection {
                        app.render(&mut canvas, &texture_foreground, &texture_background);
                    }
                }
                Event::MouseButtonUp { .. } => {
                    let mut app = app.lock().unwrap();
                    if let Some((selection_x, selection_y)) = app.selection {
                        // assert!(selection in bounds)
                        let input_addr = app.inputs[selection_x].0;
                        let output_addr = app.outputs[selection_y].0;

                        let mut seq = Seq::open(None, None, false).unwrap();
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
                    app.resize_window(&mut canvas);
                    app.render(&mut canvas, &texture_foreground, &texture_background);
                }
                _ => {}
            }
        }
    }

    Ok(())
}

// Unsub
//seq.unsubscribe_port(Addr { client: 14, port: 0 }, Addr { client: 129, port: 0 })
//    .map_err(|e| e.to_string())?;

// Sub
//let mut sub = PortSubscribe::empty().map_err(|e| e.to_string())?;
//sub.set_sender(Addr { client: 14, port: 0 });
//sub.set_dest(Addr { client: 129, port: 0 });
//seq.subscribe_port(&sub).map_err(|e| e.to_string())?;
