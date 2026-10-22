mod assets;

use std::error::Error;

use smithay::backend::allocator::Fourcc;
use smithay::backend::input::{AbsolutePositionEvent, InputEvent};
use smithay::backend::renderer::element::memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement};
use smithay::backend::renderer::element::Kind;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::backend::renderer::utils::draw_render_elements;
use smithay::backend::renderer::{Color32F, Frame, Renderer};
use smithay::backend::winit::{self, WinitEvent};
use smithay::utils::{Physical, Point, Rectangle, Transform};
use ::winit::platform::pump_events::PumpStatus;

const BACKGROUND: [f32; 4] = [0.07, 0.07, 0.08, 1.0];
const TEXT_SCALE: usize = 6;
const CURSOR_SCALE: usize = 2;

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

fn run(fullscreen: bool) -> Result<(), Box<dyn Error>> {
    let (mut backend, mut winit) = winit::init::<GlesRenderer>()?;

    // The host cursor would sit on top of ours, so hide it.
    backend.window().set_cursor_visible(false);
    if fullscreen {
        backend
            .window()
            .set_fullscreen(Some(::winit::window::Fullscreen::Borderless(None)));
    }

    let hello = sprite(assets::text("hello world", TEXT_SCALE, assets::OCEAN_BLUE));
    let cursor = sprite(assets::cursor(CURSOR_SCALE));

    let mut pointer: Point<f64, Physical> = Point::from((0.0, 0.0));
    let mut pointer_initialised = false;

    loop {
        let size = backend.window_size();

        let status = winit.dispatch_new_events(|event| {
            if let WinitEvent::Input(InputEvent::PointerMotionAbsolute { event }) = event {
                pointer = Point::from((event.x_transformed(size.w), event.y_transformed(size.h)));
                pointer_initialised = true;
            }
        });

        match status {
            PumpStatus::Continue => (),
            PumpStatus::Exit(_) => return Ok(()),
        }

        let damage = Rectangle::from_size(size);

        if !pointer_initialised {
            pointer = Point::from((size.w as f64 / 2.0, size.h as f64 / 2.0));
        }

        {
            let (renderer, mut framebuffer) = backend.bind()?;

            let text_x = (size.w - hello.width) / 2;
            let text_y = (size.h - hello.height) / 2;

            let elements = vec![
                MemoryRenderBufferRenderElement::from_buffer(
                    renderer,
                    Point::from((text_x as f64, text_y as f64)),
                    &hello.buffer,
                    None,
                    None,
                    None,
                    Kind::Unspecified,
                )?,
                MemoryRenderBufferRenderElement::from_buffer(
                    renderer,
                    pointer,
                    &cursor.buffer,
                    None,
                    None,
                    None,
                    Kind::Unspecified,
                )?,
            ];

            let mut frame = renderer.render(&mut framebuffer, size, Transform::Flipped180)?;
            frame.clear(Color32F::from(BACKGROUND), &[damage])?;
            draw_render_elements(&mut frame, 1.0, &elements, &[damage])?;
            let _ = frame.finish()?;
        }

        backend.submit(Some(&[damage]))?;
    }
}
