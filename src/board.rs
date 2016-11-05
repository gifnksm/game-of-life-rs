use geom::{Point, Size};
use rand::Rng;
use std::mem;

type Cell = u64;
const BITS: usize = 64;
const MSB: Cell = 1 << (BITS - 1);

#[derive(Debug, Clone)]
pub struct Board {
    size: Size,
    hsize: i32,
    table: Vec<Cell>,
    ls: Vec<Cell>,
    rs: Vec<Cell>,
    buffer: Vec<Cell>,
}

impl Board {
    pub fn new_empty(size: Size) -> Self {
        let hsize = (size.0 + ((BITS - 1) as i32)) / (BITS as i32) + 1;
        let len = (hsize as usize) * (((size.1 + 2) as usize)) + 1;
        Board {
            hsize: hsize,
            size: size,
            table: vec![0; len],
            ls: vec![0; len],
            rs: vec![0; len],
            buffer: vec![0; len],
        }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn contains(&self, p: Point) -> bool {
        0 <= p.0 && p.0 < self.size.0 && 0 <= p.1 && p.1 < self.size.1
    }

    pub fn get(&self, p: Point) -> bool {
        let (offset, mask) = self.get_pos(p);
        (self.table[offset] & mask) != 0
    }

    pub fn set(&mut self, p: Point, v: bool) {
        let (offset, mask) = self.get_pos(p);
        if v {
            self.table[offset] |= mask;
        } else {
            self.table[offset] &= !mask;
        }
    }

    pub fn clear(&mut self) {
        for v in &mut self.table {
            *v = 0;
        }
    }

    pub fn random_init<R>(&mut self, rng: &mut R)
        where R: Rng
    {
        for x in 0..self.size.0 {
            for y in 0..self.size.1 {
                self.set(Point(x, y), rng.gen());
            }
        }
    }

    pub fn grow(&mut self) {
        for cx in 0..self.hsize {
            for cy in 0..self.size.1 {
                let o = self.offset(cx, cy);
                let tc = self.table[o];
                let tl = self.table[self.offset(cx - 1, cy)];
                let tr = self.table[self.offset(cx + 1, cy)];
                self.ls[o] = (tc >> 1) | ((tl & 1) << (BITS - 1));
                self.rs[o] = (tc << 1) | ((tr & MSB) >> (BITS - 1));
            }
        }

        for cx in 0..self.hsize {
            for cy in 0..self.size.1 {
                let oc = self.offset(cx, cy);
                let ou = self.offset(cx, cy - 1);
                let od = self.offset(cx, cy + 1);

                let t = [self.ls[ou],
                         self.ls[oc],
                         self.ls[od],
                         self.table[ou],
                         self.table[od],
                         self.rs[ou],
                         self.rs[oc],
                         self.rs[od]];

                let mut c0 = !(t[0] | t[1]);
                let mut c1 = t[0] ^ t[1];
                let mut c2 = t[0] & t[1];

                let mut c3 = c2 & t[2];
                c2 = (c2 & !t[2]) | (c1 & t[2]);
                c1 = (c1 & !t[2]) | (c0 & t[2]);
                c0 &= !t[2];

                // let mut c4 = c3 & t[3];
                c3 = (c3 & !t[3]) | (c2 & t[3]);
                c2 = (c2 & !t[3]) | (c1 & t[3]);
                c1 = (c1 & !t[3]) | (c0 & t[3]);
                c0 &= !t[3];

                // let mut c5 = c4 & t[4];
                // c4 = (c4 & !t[4]) | (c3 & t[4]);
                c3 = (c3 & !t[4]) | (c2 & t[4]);
                c2 = (c2 & !t[4]) | (c1 & t[4]);
                c1 = (c1 & !t[4]) | (c0 & t[4]);
                c0 &= !t[4];

                // let mut c6 = c5 & t[5];
                // c5 = (c5 & !t[5]) | (c4 & t[5]);
                // c4 = (c4 & !t[5]) | (c3 & t[5]);
                c3 = (c3 & !t[5]) | (c2 & t[5]);
                c2 = (c2 & !t[5]) | (c1 & t[5]);
                c1 = (c1 & !t[5]) | (c0 & t[5]);
                c0 &= !t[5];

                // let mut c7 = c6 & t[6];
                // c6 = (c6 & !t[6]) | (c5 & t[6]);
                // c5 = (c5 & !t[6]) | (c4 & t[6]);
                // c4 = (c4 & !t[6]) | (c3 & t[6]);
                c3 = (c3 & !t[6]) | (c2 & t[6]);
                c2 = (c2 & !t[6]) | (c1 & t[6]);
                c1 = (c1 & !t[6]) | (c0 & t[6]);
                // c0 &= !t[6];

                // let mut c8 = c7 & t[7];
                // c7 = (c7 & !t[7]) | (c6 & t[7]);
                // c6 = (c6 & !t[7]) | (c5 & t[7]);
                // c5 = (c5 & !t[7]) | (c4 & t[7]);
                // c4 = (c4 & !t[7]) | (c3 & t[7]);
                c3 = (c3 & !t[7]) | (c2 & t[7]);
                c2 = (c2 & !t[7]) | (c1 & t[7]);
                // c1 = (c1 & !t[7]) | (c0 & t[7]);
                // c0 &= !t[7];

                self.buffer[oc] = c3 | (self.table[oc] & c2);
            }
        }

        mem::swap(&mut self.table, &mut self.buffer);
    }

    fn offset(&self, cx: i32, cy: i32) -> usize {
        ((cx + 1) as usize) + ((cy + 1) as usize) * (self.hsize as usize)
    }

    fn get_pos(&self, p: Point) -> (usize, Cell) {
        let cx = (p.0 + (BITS as i32)) / (BITS as i32) - 1;
        let cy = p.1;
        let offset = self.offset(cx, cy);
        let mask = MSB >> (p.0 % (BITS as i32));
        (offset, mask)
    }
}
