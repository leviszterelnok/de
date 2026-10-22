mod assets;
mod state;

use std::{error::Error, ffi::OsStr, process::Child, sync::Arc};

use smithay::{
    backend::{
        allocator::Fourcc,
        input::{AbsolutePositionEvent, ButtonState, Event, KeyState, KeyboardKeyEvent, PointerButtonEvent,
            InputEvent},
        renderer::{
            element::{
                memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
                surface::{render_elements_from_surface_tree, WaylandSurfaceRenderElement},
                Kind,
            },
            gles::GlesRenderer,
            utils::draw_render_elements,
            Color32F, Frame, Renderer,
        },
        winit::{self, WinitEvent},
    },
    input::{
        keyboard::{keysyms, FilterResult},
        pointer::{ButtonEvent, MotionEvent},
    },
    reexports::{
        wayland_server::{Display, ListeningSocket},
    },
    utils::{Logical, Physical, Point, Rectangle, Size, Transform, SERIAL_COUNTER},
    wayland::compositor::{with_surface_tree_downward, SurfaceAttributes, TraversalAction},
};
use ::winit::platform::pump_events::PumpStatus;

use state::{ClientState, State};

const BACKGROUND: [f32; 4] = [0.07, 0.07, 0.08, 1.0];
const TEXT_SCALE: usize = 6;
const CURSOR_SCALE: usize = 2;
const TERMINAL: &str = "kitty";

/// A drawable buffer plus its size in pixels.
struct Sprite {
    buffer: MemoryRenderBuffer,
    width: i32,
    height: i32,
}

fn sprite(bitmap: assets::Bitmap) -> Sprite {
    Sprite {
        buffer: MemoryRenderBuffer::from_slice(
            &bitmap.pixels,
            Fourcc::Argb8888,
            (bitmap.width, bitmap.height),
            1,
            Transform::Normal,
            None,
        ),
        width: bitmap.width,
        height: bitmap.height,
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let fullscreen = std::env::args().any(|arg| arg == "--fullscreen");
    run(fullscreen)
}

/// Starts a terminal connected to our socket. The DE's own environment is not changed,
/// so a nested session keeps its own WAYLAND_DISPLAY.
fn spawn_terminal(socket: &OsStr, children: &mut Vec<Child>) {
    match std::process::Command::new(TERMINAL)
        .env("WAYLAND_DISPLAY", socket)
        .env_remove("DISPLAY")
        .spawn()
    {
        Ok(child) => children.push(child),
        Err(err) => eprintln!("could not start {TERMINAL}: {err}"),
    }
}

fn run(fullscreen: bool) -> Result<(), Box<dyn Error>> {
    let mut display: Display<State> = Display::new()?;
    let mut dh = display.handle();

    let (mut backend, mut winit) = winit::init::<GlesRenderer>()?;
    let initial_size = backend.window_size();

    // The host cursor would sit on top of ours, so hide it.
    backend.window().set_cursor_visible(false);
    if fullscreen {
        backend
            .window()
            .set_fullscreen(Some(::winit::window::Fullscreen::Borderless(None)));
    }

    let mut seat_state = smithay::input::SeatState::new();
    let mut seat = seat_state.new_wl_seat(&dh, "winit");
    let keyboard = seat.add_keyboard(Default::default(), 200, 25)?;
    let pointer = seat.add_pointer();

    let mut state = State {
        compositor_state: smithay::wayland::compositor::CompositorState::new::<State>(&dh),
        xdg_shell_state: smithay::wayland::shell::xdg::XdgShellState::new::<State>(&dh),
        shm_state: smithay::wayland::shm::ShmState::new::<State>(&dh, vec![]),
        seat_state,
        seat,
        windows: Vec::new(),
        output_size: Size::from((initial_size.w, initial_size.h)),
        pointer_location: Point::from((initial_size.w as f64 / 2.0, initial_size.h as f64 / 2.0)),
        spawn_requested: false,
    };

    let listener = ListeningSocket::bind_auto("wayland", 0..33)?;
    let socket_name = listener
        .socket_name()
        .ok_or("listening socket has no name")?
        .to_owned();
    println!("listening on {}", socket_name.to_string_lossy());

    let mut children: Vec<Child> = Vec::new();
    spawn_terminal(&socket_name, &mut children);

    let hello = sprite(assets::text("hello world", TEXT_SCALE, assets::OCEAN_BLUE));
    let cursor = sprite(assets::cursor(CURSOR_SCALE));

    loop {
        let size = backend.window_size();

        let status = winit.dispatch_new_events(|event| match event {
            WinitEvent::Resized { size, .. } => {
                state.set_output_size(Size::from((size.w, size.h)));
            }
            WinitEvent::Input(InputEvent::Keyboard { event }) => {
                let serial = SERIAL_COUNTER.next_serial();
                keyboard.input::<(), _>(
                    &mut state,
                    event.key_code(),
                    event.state(),
                    serial,
                    event.time_msec(),
                    |state, modifiers, handle| {
                        // Super+Return opens a terminal. Everything else goes to the client.
                        let is_press = event.state() == KeyState::Pressed;
                        if is_press && modifiers.logo && handle.modified_sym().raw() == keysyms::KEY_Return {
                            state.spawn_requested = true;
                            FilterResult::Intercept(())
                        } else {
                            FilterResult::Forward
                        }
                    },
                );
            }
            WinitEvent::Input(InputEvent::PointerMotionAbsolute { event }) => {
                let location: Point<f64, Logical> = Point::from((
                    event.x_transformed(size.w),
                    event.y_transformed(size.h),
                ));
                state.pointer_location = location;
                let focus = state
                    .windows
                    .last()
                    .map(|w| (w.wl_surface().clone(), Point::from((0.0, 0.0))));
                pointer.motion(
                    &mut state,
                    focus,
                    &MotionEvent {
                        location,
                        serial: SERIAL_COUNTER.next_serial(),
                        time: event.time_msec(),
                    },
                );
                pointer.frame(&mut state);
            }
            WinitEvent::Input(InputEvent::PointerButton { event }) => {
                pointer.button(
                    &mut state,
                    &ButtonEvent {
                        button: event.button_code(),
                        state: event.state(),
                        serial: SERIAL_COUNTER.next_serial(),
                        time: event.time_msec(),
                    },
                );
                pointer.frame(&mut state);
                if event.state() == ButtonState::Pressed {
                    state.focus_top_window();
                }
            }
            _ => {}
        });

        match status {
            PumpStatus::Continue => (),
            PumpStatus::Exit(_) => return Ok(()),
        }

        if std::mem::take(&mut state.spawn_requested) {
            spawn_terminal(&socket_name, &mut children);
        }

        // Accept new clients and process their requests.
        while let Some(stream) = listener.accept()? {
            dh.insert_client(stream, Arc::new(ClientState::default()))?;
        }
        display.dispatch_clients(&mut state)?;

        // Reap terminals that have exited so they do not stay as zombies.
        children.retain_mut(|child| matches!(child.try_wait(), Ok(None)));

        let damage = Rectangle::from_size(size);
        {
            let (renderer, mut framebuffer) = backend.bind()?;

            let text_x = (size.w - hello.width) / 2;
            let text_y = (size.h - hello.height) / 2;

            let background_elements = vec![MemoryRenderBufferRenderElement::from_buffer(
                renderer,
                Point::from((text_x as f64, text_y as f64)),
                &hello.buffer,
                None,
                None,
                None,
                Kind::Unspecified,
            )?];

            let window_elements: Vec<WaylandSurfaceRenderElement<GlesRenderer>> = state
                .windows
                .iter()
                .flat_map(|window| {
                    render_elements_from_surface_tree(
                        renderer,
                        window.wl_surface(),
                        (0, 0),
                        1.0,
                        1.0,
                        Kind::Unspecified,
                    )
                })
                .collect();

            let cursor_elements = vec![MemoryRenderBufferRenderElement::from_buffer(
                renderer,
                Point::<f64, Physical>::from((state.pointer_location.x, state.pointer_location.y)),
                &cursor.buffer,
                None,
                None,
                None,
                Kind::Unspecified,
            )?];

            let mut frame = renderer.render(&mut framebuffer, size, Transform::Flipped180)?;
            frame.clear(Color32F::from(BACKGROUND), &[damage])?;
            draw_render_elements(&mut frame, 1.0, &background_elements, &[damage])?;
            draw_render_elements(&mut frame, 1.0, &window_elements, &[damage])?;
            draw_render_elements(&mut frame, 1.0, &cursor_elements, &[damage])?;
            let _ = frame.finish()?;
        }

        // Frame callbacks tell clients it is time to draw their next frame.
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u32)
            .unwrap_or(0);
        for window in &state.windows {
            send_frames_surface_tree(window.wl_surface(), time);
        }

        display.flush_clients()?;
        backend.submit(Some(&[damage]))?;
    }
}

fn send_frames_surface_tree(surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface, time: u32) {
    with_surface_tree_downward(
        surface,
        (),
        |_, _, &()| TraversalAction::DoChildren(()),
        |_surf, states, &()| {
            for callback in states
                .cached_state
                .get::<SurfaceAttributes>()
                .current()
                .frame_callbacks
                .drain(..)
            {
                callback.done(time);
            }
        },
        |_, _, &()| true,
    );
}
