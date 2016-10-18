use geom::{Geom, MOVE_ALL_ADJACENTS, Point, Points, Size, Table};
use rand::Rng;
use std::mem;
use std::ops::Index;

#[derive(Debug, Clone)]
pub struct Board {
    table: Table<bool>,
    buffer: Table<bool>,
}

impl Board {
    pub fn new_empty(size: Size) -> Board {
        Board {
            table: Table::new_empty(size, false, false),
            buffer: Table::new_empty(size, false, false),
        }
    }

    pub fn points(&self) -> Points {
        self.table.points()
    }

    pub fn size(&self) -> Size {
        self.table.size()
    }

    pub fn set(&mut self, p: Point, v: bool) {
        if self.table.contains(p) {
            self.table[p] = v;
        }
    }

    pub fn clear(&mut self) {
        for p in self.table.points() {
            self.table[p] = false;
        }
    }

    pub fn random_init<R>(&mut self, rng: &mut R)
        where R: Rng
    {
        for (p, v) in self.table.points().zip(rng.gen_iter()) {
            self.table[p] = v;
        }
    }

    pub fn grow(&mut self) {
        for p in self.table.points() {
            let num_alive =
                MOVE_ALL_ADJACENTS.iter().cloned().filter(|&m| self.table[p + m]).count();
            self.buffer[p] = if self.table[p] {
                num_alive == 2 || num_alive == 3
            } else {
                num_alive == 3
            }
        }

        mem::swap(&mut self.table, &mut self.buffer);
    }
}

impl Index<Point> for Board {
    type Output = bool;

    #[inline]
    fn index(&self, p: Point) -> &bool {
        &self.table[p]
    }
}
