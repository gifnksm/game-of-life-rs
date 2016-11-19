extern crate board_game_geom as geom;
extern crate graphics;
extern crate image as im;
extern crate opengl_graphics;
extern crate piston;
extern crate sdl2_window;
extern crate rand;
extern crate time;

use app::{App, AppSettings};
use opengl_graphics::{GlGraphics, OpenGL};
use piston::input::{Button, Event, Key, MouseButton, MouseCursorEvent, MouseScrollEvent,
                    PressEvent, ReleaseEvent, RenderEvent, ResizeEvent, UpdateEvent};
use piston::window::{OpenGLWindow, WindowSettings};
use sdl2_window::Sdl2Window;

mod app;
mod board;

fn main() {
    let app_settings = AppSettings::default();
    let opengl = OpenGL::V2_1;
    let window: Sdl2Window = WindowSettings::new("Conway's Game of Life",
                                                 (app_settings.win_size.0 as u32,
                                                  app_settings.win_size.1 as u32))
        .opengl(opengl)
        .srgb(false)
        .exit_on_esc(true)
        .build()
        .expect("failed to build Window");
    let gl_graphics = GlGraphics::new(opengl);

    let mut app = App::new(&app_settings, gl_graphics);
    app.random_init(&mut rand::thread_rng());

    event_loop::run(window, handle_event, app);
}

fn handle_event(window: &mut Sdl2Window, e: Event, app: &mut App) {
    if let Some(_args) = e.update_args() {
        app.update();
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
            Key::Equals => app.zoom(1),
            Key::Minus => app.zoom(-1),
            Key::Right => app.slide(geom::Move(1, 0)),
            Key::Left => app.slide(geom::Move(-1, 0)),
            Key::Up => app.slide(geom::Move(0, -1)),
            Key::Down => app.slide(geom::Move(0, 1)),
            _ => {}
        }
    }

    if let Some(Button::Keyboard(key)) = e.release_args() {
        match key {
            Key::Right => app.slide(geom::Move(-1, 0)),
            Key::Left => app.slide(geom::Move(1, 0)),
            Key::Up => app.slide(geom::Move(0, 1)),
            Key::Down => app.slide(geom::Move(0, -1)),
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
        match vol[1] {
            x if x > 0.0 => app.zoom(1),
            x if x < 0.0 => app.zoom(-1),
            _ => {}
        }
    }

    if let Some(args) = e.render_args() {
        window.make_current();
        app.draw(args);
    }
}

#[cfg(not(target_os = "emscripten"))]
mod event_loop {
    use piston::event_loop::Events;
    use piston::input::Event;
    use sdl2_window::Sdl2Window;

    pub fn run<T>(mut window: Sdl2Window,
                  handler: fn(window: &mut Sdl2Window, e: Event, arg: &mut T),
                  mut arg: T) {
        let mut events = window.events();
        while let Some(e) = events.next(&mut window) {
            handler(&mut window, e, &mut arg);
        }
    }
}

#[cfg(target_os = "emscripten")]
mod event_loop {
    extern crate libc;
    use piston::input::{AfterRenderArgs, Event, RenderArgs, UpdateArgs};
    use piston::window::Window;
    use sdl2_window::Sdl2Window;
    use std::mem;

    extern "C" {
        pub fn emscripten_set_main_loop_arg(func: extern "C" fn(*mut libc::c_void),
                                            arg: *mut libc::c_void,
                                            fps: libc::c_int,
                                            simulate_infinite_loop: libc::c_int);
        pub fn emscripten_cancel_main_loop();
        pub fn emscripten_get_now() -> libc::c_float;
    }

    struct EventLoop<T> {
        last_updated: f64,
        window: Sdl2Window,
        handler: fn(window: &mut Sdl2Window, e: Event, arg: &mut T),
        arg: T,
    }

    pub fn run<T>(window: Sdl2Window,
                  handler: fn(window: &mut Sdl2Window, e: Event, arg: &mut T),
                  arg: T) {
        unsafe {
            let mut events = Box::new(EventLoop {
                last_updated: emscripten_get_now() as f64,
                window: window,
                handler: handler,
                arg: arg,
            });
            let events_ptr = &mut *events as *mut EventLoop<_> as *mut libc::c_void;
            emscripten_set_main_loop_arg(main_loop_c::<T>, events_ptr, 0, 1);
            mem::forget(events);
        }
    }

    extern "C" fn main_loop_c<T>(arg: *mut libc::c_void) {
        unsafe {
            let mut events: &mut EventLoop<T> = mem::transmute(arg);
            let window = &mut events.window;
            let handler = events.handler;
            let arg = &mut events.arg;
            window.swap_buffers();

            let e = Event::AfterRender(AfterRenderArgs);
            handler(window, e, arg);

            while let Some(e) = window.poll_event() {
                handler(window, Event::Input(e), arg);
            }

            if window.should_close() {
                emscripten_cancel_main_loop();
                return;
            }

            let now = emscripten_get_now() as f64;
            let dt = now - events.last_updated;
            events.last_updated = now;

            let e = Event::Update(UpdateArgs { dt: dt });
            handler(window, e, arg);

            let size = window.size();
            let draw_size = window.draw_size();
            let e = Event::Render(RenderArgs {
                ext_dt: dt,
                width: size.width,
                height: size.height,
                draw_width: draw_size.width,
                draw_height: draw_size.height,
            });
            handler(window, e, arg);
        }
    }
}
