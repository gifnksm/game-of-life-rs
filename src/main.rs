extern crate board_game_geom as geom;
extern crate gfx;
extern crate gfx_core;
extern crate graphics;
extern crate image as im;
extern crate opengl_graphics;
extern crate piston;
extern crate sdl2_window;
extern crate rand;
extern crate shader_version;
extern crate time;

use app::{App, AppSettings};
use opengl_graphics::GlGraphics;
use piston::event_loop::Events;
use piston::input::{Button, Key, MouseButton, MouseCursorEvent, MouseScrollEvent, PressEvent,
                    ReleaseEvent, RenderEvent, ResizeEvent, UpdateEvent};
use piston::window::{OpenGLWindow, WindowSettings};
use sdl2_window::Sdl2Window;
use shader_version::OpenGL;

mod app;
mod board;

type AppWindow = Sdl2Window;

fn main() {
    let mut running = true;
    let mut rng = rand::thread_rng();

    let app_settings = AppSettings::default();
    let mut app = App::new(&app_settings);
    app.random_init(&mut rng);

    let opengl = OpenGL::V2_1;
    let win_settings = WindowSettings::new("Conway's Game of Life",
                                           (app_settings.win_size.0 as u32,
                                            app_settings.win_size.1 as u32))
        .opengl(opengl)
        .srgb(false)
        .exit_on_esc(true);

    let mut window: AppWindow = win_settings.build()
        .expect("failed to build PistonWindow");
    let mut gl = GlGraphics::new(opengl);
    let mut events = window.events();
    while let Some(e) = events.next(&mut window) {
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
            gl.draw(args.viewport(),
                    |ctx, g2d| graphics::image(app.texture(), ctx.transform, g2d));
        }
    }
}
