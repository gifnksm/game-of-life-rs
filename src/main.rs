#![feature(drop_types_in_const)]

extern crate board_game_geom as geom;
extern crate graphics;
extern crate image as im;
extern crate libc;
extern crate opengl_graphics;
extern crate piston;
extern crate sdl2_window;
extern crate rand;
extern crate shader_version;
extern crate time;

use app::{App, AppSettings};
use opengl_graphics::GlGraphics;
use piston::event_loop::{Events, WindowEvents};
use piston::input::{Button, Key, MouseButton, MouseCursorEvent, MouseScrollEvent, PressEvent,
                    ReleaseEvent, RenderEvent, ResizeEvent, UpdateEvent};
use piston::window::{OpenGLWindow, WindowSettings};
use sdl2_window::Sdl2Window;
use shader_version::OpenGL;

mod app;
mod board;

type AppWindow = Sdl2Window;

static mut APP: Option<App> = None;
static mut WINDOW: Option<AppWindow> = None;
static mut EVENTS: Option<WindowEvents> = None;
static mut GL_GRAPHICS: Option<GlGraphics> = None;

fn main_loop() -> bool {
    unsafe {
        if APP.is_none() {
            let app_settings = AppSettings::default();
            let mut app = App::new(&app_settings);
            app.random_init(&mut rand::thread_rng());
            let window: AppWindow = WindowSettings::new("Conway's Game of Life",
                                                        (app_settings.win_size.0 as u32,
                                                         app_settings.win_size.1 as u32))
                .opengl(OpenGL::V2_1)
                .srgb(false)
                .exit_on_esc(true)
                .build()
                .expect("failed to build Window");
            let events = window.events();
            let gl_graphics = GlGraphics::new(OpenGL::V2_1);

            APP = Some(app);
            WINDOW = Some(window);
            EVENTS = Some(events);
            GL_GRAPHICS = Some(gl_graphics);
        }
    }

    let mut app = unsafe { APP.as_mut().unwrap() };
    let mut window = unsafe { WINDOW.as_mut().unwrap() };
    let mut events = unsafe { EVENTS.as_mut().unwrap() };
    let mut gl = unsafe { GL_GRAPHICS.as_mut().unwrap() };

    if let Some(e) = events.next(window) {
        if let Some(_args) = e.update_args() {
            app.run();
        }

        if let Some(size) = e.resize_args() {
            app.set_win_size(geom::Size(size[0] as i32, size[1] as i32));
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::R => app.random_init(&mut rand::thread_rng()),
                Key::C => app.clear(),
                Key::S => app.toggle_running(),
                Key::F => app.fit_to_win_size(),
                Key::Space => app.step(),
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

        true
    } else {
        false
    }
}

#[cfg(not(target_os = "emscripten"))]
fn run() {
    while main_loop() {}
}

#[cfg(target_os = "emscripten")]
fn run() {
    extern "C" {
        pub fn emscripten_set_main_loop(m: extern "C" fn(),
                                        fps: libc::c_int,
                                        infinite: libc::c_int);
    }
    extern "C" fn main_loop_c() {
        main_loop();
    }
    unsafe {
        emscripten_set_main_loop(main_loop_c, 60, 1);
    }
}

fn main() {
    run();
}
