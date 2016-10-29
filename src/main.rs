extern crate board_game_geom as geom;
extern crate gfx;
extern crate gfx_core;
extern crate gfx_device_gl;
extern crate gfx_graphics;
extern crate graphics;
extern crate image as im;
extern crate piston;
extern crate sdl2_window;
extern crate shader_version;
extern crate rand;
extern crate time;

use app::{App, AppSettings};
use gfx_core::Device;
use gfx_core::factory::Typed;
use gfx_graphics::{Gfx2d, GfxGraphics};
use piston::event_loop::Events;
use piston::input::{AfterRenderEvent, Button, Key, MouseButton, MouseCursorEvent,
                    MouseScrollEvent, PressEvent, ReleaseEvent, RenderEvent, ResizeEvent,
                    UpdateEvent};
use piston::window::{OpenGLWindow, Window, WindowSettings};
use sdl2_window::Sdl2Window;
use shader_version::OpenGL;

mod app;
mod board;

type AppWindow = Sdl2Window;
type AppFactory = gfx_device_gl::Factory;
type AppResources = gfx_device_gl::Resources;
type AppCommandBuffer = gfx_device_gl::CommandBuffer;
type AppGraphics<'a> = GfxGraphics<'a, AppResources, AppCommandBuffer>;
type GfxEncoder = gfx::Encoder<AppResources, gfx_device_gl::CommandBuffer>;

fn create_main_targets(dim: gfx::tex::Dimensions)
                       -> (gfx::handle::RenderTargetView<gfx_device_gl::Resources,
                                                         gfx::format::Srgba8>,
                           gfx::handle::DepthStencilView<gfx_device_gl::Resources,
                                                         gfx::format::DepthStencil>) {
    use gfx::format::{DepthStencil, Format, Formatted, Srgba8};

    let color_format: Format = <Srgba8 as Formatted>::get_format();
    let depth_format: Format =
        <DepthStencil as Formatted>::get_format();
    let (output_color, output_stencil) =
        gfx_device_gl::create_main_targets_raw(dim, color_format.0, depth_format.0);
    let output_color = Typed::new(output_color);
    let output_stencil = Typed::new(output_stencil);
    (output_color, output_stencil)
}

fn main() {
    let mut running = true;
    let mut rng = rand::thread_rng();

    let app_settings = AppSettings::default();
    let mut app = App::new(&app_settings);
    app.random_init(&mut rng);

    let win_settings = WindowSettings::new("Conway's Game of Life",
                                           (app_settings.win_size.0 as u32,
                                            app_settings.win_size.1 as u32))
        .exit_on_esc(true);

    let mut window: AppWindow = win_settings.build()
        .expect("failed to build PistonWindow");

    let (mut device, mut factory) = gfx_device_gl::create(|s| {
        window.get_proc_address(s) as *const _
    });
    let (mut output_color, mut output_stencil) = {
        let aa = win_settings.get_samples() as gfx::tex::NumSamples;
        let draw_size = window.draw_size();
        let dim = (draw_size.width as u16, draw_size.height as u16, 1, aa.into());
        create_main_targets(dim)
    };

    let mut g2d = Gfx2d::new(OpenGL::V3_2, &mut factory);
    let mut encoder: GfxEncoder = factory.create_command_buffer().into();
    let mut events = window.events();

    while let Some(e) = events.next(&mut window) {
        if let Some(_args) = e.after_render_args() {
            device.cleanup();
        }

        let dim = output_color.raw().get_dimensions();
        let (w, h) = (dim.0, dim.1);
        let draw_size = window.draw_size();
        if w != draw_size.width as u16 || h != draw_size.height as u16 {
            let dim = (draw_size.width as u16, draw_size.height as u16, dim.2, dim.3);
            let (new_output_color, new_output_stencil) = create_main_targets(dim);
            output_color = new_output_color;
            output_stencil = new_output_stencil;
        }

        if let Some(_args) = e.update_args() {
            if running {
                app.grow();
            }
        }

        if let Some(size) = e.resize_args() {
            app.set_win_size(geom::Size(size[0] as i32, size[1] as i32));
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::R => app.random_init(&mut rng),
                Key::C => app.clear(),
                Key::S => running = !running,
                Key::F => app.fit_to_win_size(),
                Key::Space => {
                    if !running {
                        app.grow();
                    }
                }
                _ => {}
            }
        }

        if let Some(Button::Mouse(button)) = e.press_args() {
            match button {
                MouseButton::Left => app.drawing(true),
                MouseButton::Right => app.erasing(true),
                MouseButton::Middle => app.moving(true),
                _ => {}
            }
        }

        if let Some(Button::Mouse(button)) = e.release_args() {
            match button {
                MouseButton::Left => app.drawing(false),
                MouseButton::Right => app.erasing(false),
                MouseButton::Middle => app.moving(false),
                _ => {}
            }
        }

        if let Some(pos) = e.mouse_cursor_args() {
            app.mouse_move(geom::Point(pos[0] as i32, pos[1] as i32));
        }

        if let Some(vol) = e.mouse_scroll_args() {
            app.zoom(vol[1] as i32);
        }

        if let Some(args) = e.render_args() {
            window.make_current();
            app.update_texture(&mut factory, &mut encoder);
            g2d.draw(&mut encoder,
                     &output_color,
                     &output_stencil,
                     args.viewport(),
                     |ctx, g| app.draw(ctx, g));
            encoder.flush(&mut device);
        }
    }
}
