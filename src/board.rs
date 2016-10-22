use geom::{Geom, MOVE_ALL_ADJACENTS, Point, Points, Size, Table};
use rand::Rng;
use std::mem;
use std::ops::Index;

#[derive(Debug, Clone)]
pub struct Board {
    table: Table<bool>,
    count: Table<u8>,
    buffer: Table<bool>,
}

impl Board {
    pub fn new_empty(size: Size) -> Board {
        Board {
            table: Table::new_empty(size, false, false),
            count: Table::new_empty(size, 0, 0),
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
            self.update_count();
        }
    }

    pub fn set_iter<I>(&mut self, it: I)
        where I: Iterator<Item = (Point, bool)>
    {
        for (p, v) in it {
            if self.table.contains(p) {
                self.table[p] = v;
            }
        }
        self.update_count();
    }

    pub fn clear(&mut self) {
        for p in self.table.points() {
            self.table[p] = false;
        }
        self.update_count();
    }

    pub fn random_init<R>(&mut self, rng: &mut R)
        where R: Rng
    {
        for (p, v) in self.table.points().zip(rng.gen_iter()) {
            self.table[p] = v;
        }
        self.update_count();
    }

    pub fn grow(&mut self) {
        for p in self.table.points() {
            let num_alive = self.count[p];
            self.buffer[p] = num_alive == 3 || (self.table[p] && num_alive == 2);
        }

        mem::swap(&mut self.table, &mut self.buffer);
        self.update_count();
    }

    fn update_count(&mut self) {
        for p in self.table.points() {
            self.count[p] = 0;
        }

        for p in self.table.points() {
            if self.table[p] {
                for &mv in MOVE_ALL_ADJACENTS.iter() {
                    if self.table.contains(p + mv) {
                        self.count[p + mv] += 1;
                    }
                }
            }
        }
    }
}

impl Index<Point> for Board {
    type Output = bool;

    #[inline]
    fn index(&self, p: Point) -> &bool {
        &self.table[p]
    }
}
