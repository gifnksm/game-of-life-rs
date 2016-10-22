extern crate board_game_geom as geom;
extern crate image as im;
extern crate piston_window;
extern crate rand;
extern crate time;

use app::{App, AppSettings};
use piston_window::*;

mod app;
mod board;

fn main() {
    let mut running = true;
    let mut rng = rand::thread_rng();

    let app_settings = AppSettings::default();

    let mut window: PistonWindow = WindowSettings::new("Conway's Game of Life",
                                                       (app_settings.win_size.0 as u32,
                                                        app_settings.win_size.1 as u32))
        .exit_on_esc(true)
        .build()
        .expect("failed to build PistonWindow");

    let mut app = App::new(&app_settings);
    app.random_init(&mut rng);

    while let Some(e) = window.next() {
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

        if let Some(_args) = e.render_args() {
            app.draw(&mut window, &e);
        }
    }
}
