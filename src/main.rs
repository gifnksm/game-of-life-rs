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

type AppWindow = Sdl2Window;

struct Arg {
    app: App,
    window: AppWindow,
    gl_graphics: GlGraphics,
}

impl Arg {
    fn new() -> Arg {
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
        let gl_graphics = GlGraphics::new(OpenGL::V2_1);
        Arg {
            app: app,
            window: window,
            gl_graphics: gl_graphics,
        }
    }
}

fn handle_event(e: Event, arg: &mut Arg) {
    let &mut Arg { ref mut app, ref mut window, ref mut gl_graphics } = arg;

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
        gl_graphics.draw(args.viewport(),
                         |ctx, g2d| graphics::image(app.texture(), ctx.transform, g2d));
    }
}

#[cfg(not(target_os = "emscripten"))]
fn main() {
    use piston::event_loop::Events;

    let mut arg = Arg::new();
    let mut events = arg.window.events();
    while let Some(e) = events.next(&mut arg.window) {
        handle_event(e, &mut arg);
    }
}

#[cfg(target_os = "emscripten")]
fn main() {
    extern crate libc;
    use piston::window::Window;
    use piston::input::{AfterRenderArgs, RenderArgs, UpdateArgs};

    extern "C" {
        pub fn emscripten_set_main_loop_arg(func: extern "C" fn(*mut libc::c_void),
                                            arg: *mut libc::c_void,
                                            fps: libc::c_int,
                                            simulate_infinite_loop: libc::c_int);
        pub fn emscripten_cancel_main_loop();
        pub fn emscripten_get_now() -> libc::c_float;
    }

    struct EMArg {
        last_updated: f64,
        arg: Arg,
    }

    let mut arg = EMArg {
        last_updated: unsafe { emscripten_get_now() as f64 },
        arg: Arg::new(),
    };

    unsafe {
        let emarg = &mut arg as *mut _ as *mut libc::c_void;
        emscripten_set_main_loop_arg(main_loop_c, emarg, 0, 1);
    }

    extern "C" fn main_loop_c(arg: *mut libc::c_void) {
        unsafe {
            let mut emarg: &mut EMArg = std::mem::transmute(arg);
            let arg = &mut emarg.arg;
            arg.window.swap_buffers();

            let e = Event::AfterRender(AfterRenderArgs);
            handle_event(e, arg);

            while let Some(e) = arg.window.poll_event() {
                handle_event(Event::Input(e), arg);
            }

            if arg.window.should_close() {
                emscripten_cancel_main_loop();
                return;
            }

            let now = emscripten_get_now() as f64;
            let dt = now - emarg.last_updated;
            emarg.last_updated = now;

            let e = Event::Update(UpdateArgs { dt: dt });
            handle_event(e, arg);

            let size = arg.window.size();
            let draw_size = arg.window.draw_size();
            let e = Event::Render(RenderArgs {
                ext_dt: dt,
                width: size.width,
                height: size.height,
                draw_width: draw_size.width,
                draw_height: draw_size.height,
            });
            handle_event(e, arg);
        }
    }
}
