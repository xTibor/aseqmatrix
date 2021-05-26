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
use graphics::{draw_string, draw_tiles, TILE_SIZE};

struct MidiPortChangeEvent;

struct MidiMatrixApp {
    inputs: Vec<(Addr, String)>,
    outputs: Vec<(Addr, String)>,
}

impl MidiMatrixApp {
    fn new() -> MidiMatrixApp {
        MidiMatrixApp {
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    fn render(&self, canvas: &mut Canvas<Window>, texture: &Texture) {
        canvas.set_draw_color(pixels::Color::RGB(128, 128, 128));
        canvas.clear();

        for (output_index, (_, output_name)) in self.outputs.iter().enumerate() {
            let y = 1 + output_index as isize * 2;
            let x_arrow_right = self.inputs.len() as isize * 2 + 1;
            let x_text = x_arrow_right + 2;

            draw_tiles(canvas, texture, (2, 0), (x_arrow_right, y), 1, 2).unwrap();
            draw_string(canvas, texture, output_name, (x_text, y), 0).unwrap();
        }

        for (input_index, (_, input_name)) in self.inputs.iter().enumerate() {
            let x = 1 + input_index as isize * 2;
            let y_arrow_down = self.outputs.len() as isize * 2 + 1;
            let y_text = y_arrow_down + input_name.len() as isize + 1;

            draw_tiles(canvas, texture, (0, 3), (x, y_arrow_down), 2, 1).unwrap();
            draw_string(canvas, texture, input_name, (x, y_text), 3).unwrap();
        }

        for y in 0..self.outputs.len() {
            for x in 0..self.inputs.len() {
                draw_tiles(canvas, texture, (0, 0), (1 + x as isize * 2, 1 + y as isize * 2), 2, 2).unwrap();
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
    let texture = texture_creator.load_texture("assets/tileset.png")?;

    {
        let app = app.lock().unwrap();
        app.resize_window(&mut canvas);
        app.render(&mut canvas, &texture);
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

                for client in ClientIter::new(&seq) {
                    for port in PortIter::new(&seq, client.get_client()) {
                        if port.get_capability().contains(PortCap::SUBS_READ) {
                            app.inputs.push((port.addr(), port.get_name().unwrap().to_owned()))
                        }

                        if port.get_capability().contains(PortCap::SUBS_WRITE) {
                            app.outputs.push((port.addr(), port.get_name().unwrap().to_owned()))
                        }

                        //for sub in PortSubscribeIter::new(&seq, port.addr(), QuerySubsType::WRITE) {
                        //    println!(">>> {:?} -> {:?}", sub.get_sender(), sub.get_dest());
                        //}
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
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    let mut seq = Seq::open(None, None, false).unwrap();
                    seq.unsubscribe_port(Addr { client: 14, port: 0 }, Addr { client: 129, port: 0 })
                        .unwrap();
                }
                Event::User { .. } => {
                    // TODO: Check user_event kind
                    let app = app.lock().unwrap();
                    app.resize_window(&mut canvas);
                    app.render(&mut canvas, &texture);
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
