use board::Board;
use geom::{Move, Point, Size};
use piston_window::{self, Context, Graphics};
use rand::Rng;
use std::cmp;

pub struct App {
    win_size: Size,
    rect_size: i32,
    offset: Move,

    mouse_pos: Point,
    onmouse_cell: Point,

    drawing: bool,
    erasing: bool,
    moving: Option<(Point, Move)>,

    board: Board,
}

impl Default for App {
    fn default() -> Self {
        let win_size = Size(640, 480);
        let rect_size = 4;

        App {
            win_size: win_size,
            rect_size: rect_size,
            offset: Move(0, 0),

            mouse_pos: Point(0, 0),
            onmouse_cell: Point(0, 0),

            drawing: false,
            erasing: false,
            moving: None,

            board: Board::new_empty(Size(win_size.0 / rect_size, win_size.1 / rect_size)),
        }
    }
}

impl App {
    pub fn random_init<R>(&mut self, rng: &mut R)
        where R: Rng
    {
        self.board.random_init(rng)
    }

    pub fn clear(&mut self) {
        self.board.clear()
    }

    pub fn grow(&mut self) {
        self.board.grow()
    }

    pub fn win_size(&self) -> Size {
        self.win_size
    }
    pub fn set_win_size(&mut self, size: Size) {
        self.win_size = size;
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

    pub fn draw<G>(&mut self, ctx: Context, g2d: &mut G)
        where G: Graphics
    {
        self.adjust_offset();
        piston_window::clear([0.3, 0.3, 0.3, 1.0], g2d);

        for p in self.board.points() {
            let color = if self.board[p] {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                [0.0, 0.0, 0.0, 1.0]
            };
            let rect = [(self.offset.0 + p.0 * self.rect_size) as f64,
                        (self.offset.1 + p.1 * self.rect_size) as f64,
                        self.rect_size as f64,
                        self.rect_size as f64];

            piston_window::rectangle(color, rect, ctx.transform, g2d);
        }

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
