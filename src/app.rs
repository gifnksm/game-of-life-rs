use board::Board;
use geom::{Move, Point, Size};
use im::{ImageBuffer, Rgba};
use piston_window::{self, Event, G2dTexture, PistonWindow, Texture, TextureSettings, Transformed};
use piston_window::texture::Filter;
use rand::Rng;
use std::cmp;

pub struct AppSettings {
    pub win_size: Size,
    pub board_size: Size,
    pub rect_size: i32,
}

impl Default for AppSettings {
    fn default() -> Self {
        let win_size = Size(640, 480);
        let rect_size = 4;
        let board_size = Size(win_size.0 / rect_size, win_size.1 / rect_size);

        AppSettings {
            win_size: win_size,
            board_size: board_size,
            rect_size: rect_size,
        }
    }
}

pub struct App {
    win_size: Size,
    rect_size: i32,
    offset: Move,

    mouse_pos: Point,
    onmouse_cell: Point,

    drawing: bool,
    erasing: bool,
    moving: Option<(Point, Move)>,

    texture: G2dTexture<'static>,
    canvas: ImageBuffer<Rgba<u8>, Vec<u8>>,
    invalidated: bool,

    board: Board,
}

impl App {
    pub fn new<W>(settings: &AppSettings, window: &mut PistonWindow<W>) -> Self
        where W: piston_window::Window
    {
        let canvas = ImageBuffer::new(settings.board_size.0 as u32, settings.board_size.1 as u32);
        let texture = Texture::from_image(&mut window.factory,
                                          &canvas,
                                          &TextureSettings::new().filter(Filter::Linear))
            .expect("failed to build Texture");

        App {
            win_size: settings.win_size,
            rect_size: settings.rect_size,
            offset: Move(0, 0),

            mouse_pos: Point(0, 0),
            onmouse_cell: Point(0, 0),

            drawing: false,
            erasing: false,
            moving: None,

            texture: texture,
            canvas: canvas,
            invalidated: true,

            board: Board::new_empty(settings.board_size),
        }
    }

    pub fn random_init<R>(&mut self, rng: &mut R)
        where R: Rng
    {
        self.board.random_init(rng);
        self.invalidated = true;
    }

    pub fn clear(&mut self) {
        self.board.clear();
        self.invalidated = true;
    }

    pub fn grow(&mut self) {
        self.board.grow();
        self.invalidated = true;
    }

    pub fn set_win_size(&mut self, size: Size) {
        self.win_size = size;
        self.invalidated = true;
    }

    pub fn fit_to_win_size<W>(&mut self, window: &mut PistonWindow<W>)
        where W: piston_window::Window
    {
        let new_size = Size(self.win_size.0 / self.rect_size,
                            self.win_size.1 / self.rect_size);
        let mut board = Board::new_empty(new_size);
        for x in 0..cmp::min(board.size().0, self.board.size().0) {
            for y in 0..cmp::min(board.size().1, self.board.size().1) {
                let p = Point(x, y);
                board.set(p, self.board.get(p));
            }
        }

        self.board = board;
        self.canvas = ImageBuffer::new(new_size.0 as u32, new_size.1 as u32);
        self.texture = Texture::from_image(&mut window.factory,
                                           &self.canvas,
                                           &TextureSettings::new().filter(Filter::Linear))
            .expect("failed to build Texture");
        self.invalidated = true;
    }

    pub fn mouse_move(&mut self, mouse_pos: Point) {
        self.mouse_pos = mouse_pos;

        let onmouse_cell = self.pos2cell(mouse_pos);
        let old_cell = self.onmouse_cell;
        self.onmouse_cell = onmouse_cell;

        if let Some((start_pos, start_offset)) = self.moving {
            self.offset = start_offset + (mouse_pos - start_pos);
        }

        let op = match (self.drawing, self.erasing) {
            (true, true) => None,
            (true, false) => Some(true),
            (false, true) => Some(false),
            (false, false) => None,
        };

        if let Some(val) = op {
            let diff = onmouse_cell - old_cell;
            let a0 = diff.0.abs();
            let a1 = diff.1.abs();
            let a = cmp::max(a0, a1);
            for i in 0..a {
                let mv = Move(diff.0 * i / a, diff.1 * i / a);
                self.board.set(old_cell + mv, val);
            }
            self.board.set(onmouse_cell, val);
            self.invalidated = true;
        }
    }

    pub fn zoom(&mut self, scale: i32) {
        let pos0 = ((self.mouse_pos.0 - self.offset.0) as f64) / (self.rect_size as f64);
        let pos1 = ((self.mouse_pos.1 - self.offset.1) as f64) / (self.rect_size as f64);

        if scale > 0 {
            for _ in 0..scale.abs() {
                if self.rect_size < 32 {
                    self.rect_size *= 2;
                }
            }
        } else {
            for _ in 0..scale.abs() {
                if self.rect_size > 1 {
                    self.rect_size /= 2;
                }
            }
        }

        self.offset = Move(((self.mouse_pos.0 as f64) - pos0 * (self.rect_size as f64)) as i32,
                           ((self.mouse_pos.1 as f64) - pos1 * (self.rect_size as f64)) as i32);
        self.invalidated = true;
    }

    pub fn drawing(&mut self, val: bool) {
        self.drawing = val;
        let mouse_pos = self.mouse_pos;
        self.mouse_move(mouse_pos);
    }
    pub fn erasing(&mut self, val: bool) {
        self.erasing = val;
        let mouse_pos = self.mouse_pos;
        self.mouse_move(mouse_pos);
    }
    pub fn moving(&mut self, val: bool) {
        if val {
            self.moving = Some((self.mouse_pos, self.offset));
        } else {
            self.moving = None;
        }
    }

    pub fn draw(&mut self, window: &mut PistonWindow, e: &Event) {
        self.adjust_offset();

        if self.invalidated {
            self.invalidated = false;
            for x in 0..self.board.size().0 {
                for y in 0..self.board.size().1 {
                    let p = Point(x, y);
                    let color = if self.board.get(p) {
                        [255, 255, 255, 255]
                    } else {
                        [0, 0, 0, 255]
                    };
                    self.canvas.put_pixel(p.0 as u32, p.1 as u32, Rgba(color));
                }
            }
            self.texture
                .update(&mut window.encoder, &self.canvas)
                .expect("failed to update Texture");
        }

        window.draw_2d(e, |ctx, g2d| {
            piston_window::clear([0.3, 0.3, 0.3, 1.0], g2d);
            piston_window::image(&self.texture,
                                 ctx.trans(self.offset.0 as f64, self.offset.1 as f64)
                                     .scale(self.rect_size as f64, self.rect_size as f64)
                                     .transform,
                                 g2d);
        });
    }

    fn board_size(&self) -> Size {
        Size(self.rect_size * self.board.size().0,
             self.rect_size * self.board.size().1)
    }

    fn adjust_offset(&mut self) {
        let board_size = self.board_size();

        self.offset.0 = if board_size.0 < self.win_size.0 {
            (self.win_size.0 - board_size.0) / 2
        } else {
            clamp(self.offset.0, self.win_size.0 - board_size.0, 0)
        };

        self.offset.1 = if board_size.1 < self.win_size.1 {
            (self.win_size.1 - board_size.1) / 2
        } else {
            clamp(self.offset.1, self.win_size.1 - board_size.1, 0)
        };
    }

    fn pos2cell(&self, pos: Point) -> Point {
        Point((pos.0 - self.offset.0) / self.rect_size,
              (pos.1 - self.offset.1) / self.rect_size)
    }
}

fn clamp(val: i32, min: i32, max: i32) -> i32 {
    cmp::min(cmp::max(val, min), max)
}
