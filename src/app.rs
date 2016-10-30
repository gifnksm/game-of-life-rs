use board::Board;
use geom::{Move, Point, Size};
use im::{ImageBuffer, Rgba};
use opengl_graphics::{Texture, TextureSettings};
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

    running: bool,
    mouse_pos: Point,
    onmouse_cell: Point,

    drawing: bool,
    erasing: bool,
    moving: Option<(Point, Move)>,

    texture: Option<Texture>,
    canvas: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    invalidated: bool,

    board: Board,
}

impl App {
    pub fn new(settings: &AppSettings) -> Self {
        App {
            win_size: settings.win_size,
            rect_size: settings.rect_size,
            offset: Move(0, 0),

            running: true,
            mouse_pos: Point(0, 0),
            onmouse_cell: Point(0, 0),

            drawing: false,
            erasing: false,
            moving: None,

            texture: None,
            canvas: None,
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

    pub fn step(&mut self) {
        if !self.running {
            self.board.grow();
            self.invalidated = true;
        }
    }

    pub fn run(&mut self) {
        if self.running {
            self.board.grow();
            self.invalidated = true;
        }
    }

    pub fn set_win_size(&mut self, size: Size) {
        self.win_size = size;
        self.invalidated = true;
        self.canvas = None;
        self.texture = None;
    }

    pub fn fit_to_win_size(&mut self) {
        let new_size = Size(self.win_size.0 / self.rect_size,
                            self.win_size.1 / self.rect_size);
        let mut board = Board::new_empty(new_size);

        let x_off = if board.size().0 > self.board.size().0 {
            (board.size().0 - self.board.size().0) / 2
        } else {
            0
        };
        let y_off = if board.size().1 > self.board.size().1 {
            (board.size().1 - self.board.size().1) / 2
        } else {
            0
        };

        for x in 0..cmp::min(board.size().0, self.board.size().0) {
            for y in 0..cmp::min(board.size().1, self.board.size().1) {
                board.set(Point(x + x_off, y + y_off), self.board.get(Point(x, y)));
            }
        }

        self.board = board;
        self.invalidated = true;
    }

    pub fn toggle_running(&mut self) {
        self.running = !self.running;
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
                let p = old_cell + Move(diff.0 * i / a, diff.1 * i / a);
                if self.board.contains(p) {
                    self.board.set(p, val);
                }
            }
            if self.board.contains(onmouse_cell) {
                self.board.set(onmouse_cell, val);
            }
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

    pub fn texture(&mut self) -> &Texture {
        self.adjust_offset();

        if self.canvas.is_none() || self.texture.is_none() {
            let canvas = ImageBuffer::new(self.win_size.0 as u32, self.win_size.1 as u32);
            let texture = Texture::from_image(&canvas, &TextureSettings::new());
            self.canvas = Some(canvas);
            self.texture = Some(texture);
            self.invalidated = true;
        }

        {
            let canvas = self.canvas.as_mut().unwrap();
            let texture = self.texture.as_mut().unwrap();

            if self.invalidated {
                self.invalidated = false;

                for wx in 0..self.win_size.0 {
                    for wy in 0..self.win_size.1 {
                        let p = Point((wx - self.offset.0) / self.rect_size,
                                      (wy - self.offset.1) / self.rect_size);
                        let color = if !self.board.contains(p) {
                            [128, 128, 128, 255]
                        } else if self.board.get(p) {
                            [255, 255, 255, 255]
                        } else {
                            [0, 0, 0, 255]
                        };
                        canvas.put_pixel(wx as u32, wy as u32, Rgba(color));
                    }
                }

                texture.update(canvas);
            }
        }

        self.texture.as_ref().unwrap()
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
