use std::ffi::CString;
use std::sync::{Arc, Mutex};
use std::{thread, time};

use alsa::seq::{
    Addr, ClientIter, PortCap, PortInfo, PortIter, PortSubscribe, PortSubscribeIter, PortType, QuerySubsType, Seq,
};
use alsa::PollDescriptors;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::render::Canvas;
use sdl2::video::Window;

mod graphics;
use graphics::{draw_borders, draw_string, draw_tiled_background, draw_tiles, PixelDimension, PixelPosition};

mod theme;
use theme::Theme;

mod error;
use error::{sdl_error, Error};

mod config;
use config::AppConfig;

struct MidiPortChangeEvent;

struct AppState {
    inputs: Vec<(Addr, String)>,
    outputs: Vec<(Addr, String)>,
    connections: Vec<(Addr, Addr)>,
    selection: Option<(usize, usize)>,
    mouse_down: bool,
    mouse_last_position: Option<PixelPosition>,
    config: AppConfig,
}

impl AppState {
    fn new() -> Result<AppState, Error> {
        Ok(AppState {
            inputs: Vec::new(),
            outputs: Vec::new(),
            connections: Vec::new(),
            selection: None,
            mouse_down: false,
            mouse_last_position: None,
            config: AppConfig::new()?,
        })
    }

    fn input_names(&self) -> Vec<String> {
        self.inputs
            .iter()
            .map(|(port_addr, port_name)| {
                if self.config.show_addresses {
                    format!("{} {:>3}:{}", port_name, port_addr.client, port_addr.port)
                } else {
                    port_name.clone()
                }
            })
            .collect()
    }

    fn output_names(&self) -> Vec<String> {
        self.outputs
            .iter()
            .map(|(port_addr, port_name)| {
                if self.config.show_addresses {
                    format!("{:>3}:{} {}", port_addr.client, port_addr.port, port_name)
                } else {
                    port_name.clone()
                }
            })
            .collect()
    }

    fn render(&self, canvas: &mut Canvas<Window>, theme: &Theme) -> Result<(), Error> {
        draw_tiled_background(canvas, &theme.background_texture)?;
        draw_borders(canvas, &theme.borders_texture)?;

        let button_dimensions = PixelDimension {
            width: theme.controls_texture.tile_size.width * 2,
            height: theme.controls_texture.tile_size.height * 2,
        };

        let (horizontal_arrow_width, vertical_arrow_height) =
            (theme.controls_texture.tile_size.width as isize, theme.controls_texture.tile_size.height as isize);

        for (output_index, output_name) in self.output_names().iter().enumerate() {
            let arrow_source = match self.selection {
                Some((_, selection_y)) if selection_y == output_index => Theme::RECT_ARROW_LEFT_ACTIVE,
                _ => Theme::RECT_ARROW_LEFT_NORMAL,
            };

            let arrow_position = PixelPosition {
                x: theme.manifest.metrics.window_margin as isize
                    + self.inputs.len() as isize * button_dimensions.width as isize,
                y: theme.manifest.metrics.window_margin as isize
                    + output_index as isize * button_dimensions.height as isize,
            };

            let text_position = PixelPosition {
                x: arrow_position.x + horizontal_arrow_width + theme.manifest.metrics.label_spacing as isize,
                y: arrow_position.y
                    + (button_dimensions.height as isize - theme.font_texture.tile_size.height as isize) / 2,
            };

            draw_tiles(canvas, &theme.controls_texture, arrow_source, arrow_position)?;
            draw_string(canvas, &theme.font_texture, output_name, text_position, 0)?;
        }

        for (input_index, input_name) in self.input_names().iter().enumerate() {
            let arrow_source = match self.selection {
                Some((selection_x, _)) if selection_x == input_index => Theme::RECT_ARROW_DOWN_ACTIVE,
                _ => Theme::RECT_ARROW_DOWN_NORMAL,
            };

            let arrow_position = PixelPosition {
                x: theme.manifest.metrics.window_margin as isize
                    + input_index as isize * button_dimensions.width as isize,
                y: theme.manifest.metrics.window_margin as isize
                    + self.outputs.len() as isize * button_dimensions.height as isize,
            };

            let text_position = PixelPosition {
                x: arrow_position.x
                    + (button_dimensions.width as isize - theme.font_texture.tile_size.height as isize) / 2,
                y: arrow_position.y
                    + vertical_arrow_height
                    + theme.manifest.metrics.label_spacing as isize
                    + input_name.len() as isize * theme.font_texture.tile_size.width as isize,
            };

            draw_tiles(canvas, &theme.controls_texture, arrow_source, arrow_position)?;
            draw_string(canvas, &theme.font_texture, input_name, text_position, 3)?;
        }

        for (output_index, (output_addr, _)) in self.outputs.iter().enumerate() {
            for (input_index, (input_addr, _)) in self.inputs.iter().enumerate() {
                let has_connection = self.connections.contains(&(*input_addr, *output_addr));
                let currently_hovered = self.selection == Some((input_index, output_index));
                let currently_down = (self.mouse_down) && (self.selection == Some((input_index, output_index)));

                let button_source = match (input_addr == output_addr, has_connection, currently_down, currently_hovered)
                {
                    (true, _, false, true) => Theme::RECT_BUTTON_DISABLED_HOVER,
                    (true, _, true, true) => Theme::RECT_BUTTON_DISABLED_DOWN,
                    (true, _, _, _) => Theme::RECT_BUTTON_DISABLED,
                    (false, false, false, true) => Theme::RECT_BUTTON_NORMAL_HOVER,
                    (false, false, true, true) => Theme::RECT_BUTTON_NORMAL_DOWN,
                    (false, false, _, _) => Theme::RECT_BUTTON_NORMAL,
                    (false, true, false, true) => Theme::RECT_BUTTON_ACTIVE_HOVER,
                    (false, true, true, true) => Theme::RECT_BUTTON_ACTIVE_DOWN,
                    (false, true, _, _) => Theme::RECT_BUTTON_ACTIVE,
                };

                let button_position = PixelPosition {
                    x: theme.manifest.metrics.window_margin as isize
                        + input_index as isize * button_dimensions.width as isize,
                    y: theme.manifest.metrics.window_margin as isize
                        + output_index as isize * button_dimensions.height as isize,
                };

                draw_tiles(canvas, &theme.controls_texture, button_source, button_position)?;
            }
        }

        canvas.present();
        Ok(())
    }

    fn resize_window(&mut self, canvas: &mut Canvas<Window>, theme: &Theme) -> Result<(), Error> {
        let window_width = theme.manifest.metrics.window_margin
            + self.inputs.len() * (2 * theme.controls_texture.tile_size.width) // Controls
            + theme.controls_texture.tile_size.width // Arrow
            + theme.manifest.metrics.label_spacing
            + self.output_names().iter().map(String::len).max().unwrap_or(0) * (theme.font_texture.tile_size.width)
            + theme.manifest.metrics.window_margin;

        let window_height = theme.manifest.metrics.window_margin
            + self.outputs.len() * (2 * theme.controls_texture.tile_size.height) // Controls
            + theme.controls_texture.tile_size.height // Arrow
            + theme.manifest.metrics.label_spacing
            + self.input_names().iter().map(String::len).max().unwrap_or(0) * (theme.font_texture.tile_size.width)
            + theme.manifest.metrics.window_margin;

        let window = canvas.window_mut();
        window.set_size(window_width as u32, window_height as u32)?;

        if let Some(mouse_last_position) = self.mouse_last_position {
            self.update_selection(canvas, theme, mouse_last_position, false)?;
        }

        // Workaround for SDL2 corrupting things right after resizing the window.
        thread::sleep(time::Duration::from_millis(20));

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
            px as usize / (theme.controls_texture.tile_size.width * 2),
            py as usize / (theme.controls_texture.tile_size.height * 2),
        );

        if (control_x < self.inputs.len()) && (control_y < self.outputs.len()) {
            Some((control_x, control_y))
        } else {
            None
        }
    }

    fn update_selection(
        &mut self,
        canvas: &mut Canvas<Window>,
        theme: &Theme,
        position: PixelPosition,
        force_redraw: bool,
    ) -> Result<(), Error> {
        let last_selection = self.selection;
        self.selection = self.control_under_position(theme, position);

        if (self.selection != last_selection) || force_redraw {
            self.render(canvas, theme)?;
        }

        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let app = Arc::new(Mutex::new(AppState::new()?));

    let sdl_context = sdl2::init().map_err(sdl_error)?;

    let video_subsys = sdl_context.video().map_err(sdl_error)?;
    video_subsys.enable_screen_saver();
    sdl2::hint::set("SDL_MOUSE_FOCUS_CLICKTHROUGH", "1");

    let window = video_subsys.window("ALSA Sequencer Matrix", 640, 480).hidden().build()?;

    let mut canvas = window.into_canvas().build()?;
    let texture_creator = canvas.texture_creator();

    let mut theme = {
        let app = app.lock().unwrap();
        Theme::new(&texture_creator, &app.config.theme_manifest_path)?
    };

    {
        let mut app = app.lock().unwrap();
        app.resize_window(&mut canvas, &theme)?;
        app.render(&mut canvas, &theme)?;

        // Window was created as hidden to avoid flickering during the initial resize
        canvas.window_mut().show();
    }

    let sdl_event = sdl_context.event().map_err(sdl_error)?;
    sdl_event.register_custom_event::<MidiPortChangeEvent>().map_err(sdl_error)?;
    sdl_event.push_custom_event(MidiPortChangeEvent).map_err(sdl_error)?;
    let tx = sdl_event.event_sender();

    {
        let app = Arc::clone(&app);
        thread::spawn(move || -> Result<(), Error> {
            let seq = Seq::open(None, None, false)?;

            let midi_name = CString::new("ALSA Sequencer Matrix")?;
            seq.set_client_name(&midi_name)?;

            let client_port = {
                let mut port_info = PortInfo::empty()?;
                port_info.set_capability(PortCap::WRITE);
                port_info.set_type(PortType::MIDI_GENERIC | PortType::APPLICATION);
                port_info.set_name(&midi_name);
                seq.create_port(&port_info)?;
                port_info.addr()
            };

            {
                let sub = PortSubscribe::empty()?;
                sub.set_sender(Addr::system_announce());
                sub.set_dest(client_port);
                seq.subscribe_port(&sub)?;
            }

            let refresh_midi_endpoints = || -> Result<(), Error> {
                let mut app = app.lock().unwrap();
                app.inputs.clear();
                app.outputs.clear();
                app.connections.clear();

                for client in ClientIter::new(&seq) {
                    for port in PortIter::new(&seq, client.get_client()) {
                        if port.get_capability().contains(PortCap::SUBS_READ) {
                            app.inputs.push((port.addr(), port.get_name()?.to_owned()));
                        }

                        if port.get_capability().contains(PortCap::SUBS_WRITE) {
                            app.outputs.push((port.addr(), port.get_name()?.to_owned()));
                        }

                        for sub in PortSubscribeIter::new(&seq, port.addr(), QuerySubsType::WRITE) {
                            app.connections.push((sub.get_sender(), sub.get_dest()));
                        }
                    }
                }

                Ok(())
            };

            refresh_midi_endpoints()?;

            let mut fds = Vec::<alsa::poll::pollfd>::new();
            fds.append(&mut (&seq, Some(alsa::Direction::Capture)).get()?);

            let mut seq_input = seq.input();
            loop {
                if seq_input.event_input_pending(true)? > 0 {
                    let event = seq_input.event_input()?;
                    //println!("{:?}", event);
                    // TODO: Filter events from this client
                    // TODO: Why doesn't the system announcement port send any events
                    //  about a port getting renamed under `EventType::PortChange`?
                    match event.get_type() {
                        alsa::seq::EventType::PortChange
                        | alsa::seq::EventType::PortExit
                        | alsa::seq::EventType::PortStart
                        | alsa::seq::EventType::PortSubscribed
                        | alsa::seq::EventType::PortUnsubscribed => {
                            refresh_midi_endpoints()?;
                            tx.push_custom_event(MidiPortChangeEvent).map_err(sdl_error)?;
                        }
                        _ => {}
                    }
                    continue;
                }
                alsa::poll::poll(&mut fds, -1)?;
            }
        });
    }

    let mut events = sdl_context.event_pump().map_err(sdl_error)?;
    'main: loop {
        for event in events.wait_iter() {
            //println!("{:?}", event);
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'main;
                }
                Event::MouseMotion { x, y, .. } => {
                    let mut app = app.lock().unwrap();
                    app.mouse_last_position = Some(PixelPosition { x: x as isize, y: y as isize });

                    app.update_selection(&mut canvas, &theme, PixelPosition { x: x as isize, y: y as isize }, false)?;
                }
                Event::MouseButtonDown { x, y, mouse_btn: MouseButton::Left, .. } => {
                    let mut app = app.lock().unwrap();

                    if app.selection.is_some() {
                        app.mouse_down = true;
                        app.update_selection(
                            &mut canvas,
                            &theme,
                            PixelPosition { x: x as isize, y: y as isize },
                            true,
                        )?;
                    }
                }
                Event::MouseButtonUp { x, y, mouse_btn: MouseButton::Left, .. } => {
                    let mut app = app.lock().unwrap();

                    if app.mouse_down {
                        app.mouse_down = false;
                        app.update_selection(
                            &mut canvas,
                            &theme,
                            PixelPosition { x: x as isize, y: y as isize },
                            true,
                        )?;

                        if let Some((selection_x, selection_y)) = app.selection {
                            // assert!(selection in bounds)
                            let new_input = app.inputs[selection_x].0;
                            let new_output = app.outputs[selection_y].0;

                            if new_input != new_output {
                                let seq = Seq::open(None, None, false)?;
                                if app.connections.contains(&(new_input, new_output)) {
                                    seq.unsubscribe_port(new_input, new_output)?;
                                } else {
                                    // <feedback_loop_resolver>
                                    let outgoing_target_ports = app
                                        .connections
                                        .iter()
                                        .filter(|(input, _)| *input == new_output)
                                        .map(|(_, output)| *output)
                                        .collect::<Vec<Addr>>();

                                    for &port in &outgoing_target_ports {
                                        let mut todo = vec![port];
                                        let mut done = vec![];
                                        let mut feedback_loop_found = false;

                                        while !todo.is_empty() {
                                            let current = todo.pop().unwrap(); // Cannot fail because `!is_empty()`
                                            if current == new_input {
                                                feedback_loop_found = true;
                                                break;
                                            }
                                            todo.extend_from_slice(
                                                &app.connections
                                                    .iter()
                                                    .filter(|(input, _)| *input == current)
                                                    .filter(|(_, output)| !done.contains(output))
                                                    .map(|(_, output)| *output)
                                                    .collect::<Vec<Addr>>(),
                                            );
                                            done.push(current);
                                        }

                                        if feedback_loop_found {
                                            seq.unsubscribe_port(new_output, port)?;
                                        }
                                    }
                                    // </feedback_loop_resolver>

                                    let sub = PortSubscribe::empty()?;
                                    sub.set_sender(new_input);
                                    sub.set_dest(new_output);
                                    seq.subscribe_port(&sub)?;
                                }
                            }
                        }
                    }
                }
                Event::User { .. } => {
                    // TODO: Check user_event kind
                    let mut app = app.lock().unwrap();
                    app.resize_window(&mut canvas, &theme)?;
                    app.render(&mut canvas, &theme)?;
                }
                Event::KeyDown { keycode: Some(Keycode::F11), .. } => {
                    let mut app = app.lock().unwrap();
                    app.config.show_addresses = !app.config.show_addresses;
                    app.config.save()?;
                    app.resize_window(&mut canvas, &theme)?;
                    app.render(&mut canvas, &theme)?;
                }
                Event::KeyDown { keycode: Some(Keycode::F12), .. } => {
                    let mut app = app.lock().unwrap();

                    let theme_manifest_paths = Theme::theme_manifest_paths()?;

                    let next_manifest_index = theme_manifest_paths
                        .iter()
                        .position(|manifest_path| manifest_path == &app.config.theme_manifest_path)
                        .map(|manifest_index| (manifest_index + 1) % theme_manifest_paths.len())
                        .unwrap_or(0);

                    app.config.theme_manifest_path = theme_manifest_paths[next_manifest_index].clone();
                    app.config.save()?;

                    theme = Theme::new(&texture_creator, &app.config.theme_manifest_path)?;
                    app.resize_window(&mut canvas, &theme)?;
                    app.render(&mut canvas, &theme)?;
                }
                Event::KeyDown { keycode: Some(Keycode::F5), .. } => {
                    let mut app = app.lock().unwrap();
                    theme = Theme::new(&texture_creator, &app.config.theme_manifest_path)?;
                    app.resize_window(&mut canvas, &theme)?;
                    app.render(&mut canvas, &theme)?;
                }
                _ => {}
            }
        }
    }

    Ok(())
}
