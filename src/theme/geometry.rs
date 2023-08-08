use sdl2::rect::{Point, Rect};
use crate::game::bottle::{BOTTLE_HEIGHT, BOTTLE_WIDTH};
use crate::game::pill::Vitamins;

#[derive(Debug, Copy, Clone)]
pub struct BottleGeometry {
    block_size: u32,
    block_overlap: i32,
    height: u32,
    width: u32,
    offset: Point,
}

impl BottleGeometry {
    pub fn new<P: Into<Point>>(block_size: u32, block_overlap: i32, offset: P) -> Self {
        let overlapped_block_size = (block_size as i32 + block_overlap) as u32;
        let height = overlapped_block_size * BOTTLE_HEIGHT;
        let width = overlapped_block_size * BOTTLE_WIDTH;
        Self {
            block_size,
            block_overlap,
            height,
            width,
            offset: offset.into(),
        }
    }

    fn i_to_x(&self, i: u32) -> i32 {
        i as i32 * (self.block_size as i32 + self.block_overlap) + self.offset.x() + self.block_overlap
    }

    fn j_to_y(&self, j: u32) -> i32 {
        j as i32 * (self.block_size as i32 + self.block_overlap) + self.offset.y() + self.block_overlap
    }

    pub fn block_size(&self) -> u32 {
        self.block_size
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn block_rect(&self, i: u32, j: u32) -> Rect {
        Rect::new(
            self.i_to_x(i),
            self.j_to_y(j),
            self.block_size,
            self.block_size,
        )
    }

    pub fn vitamin_rects(&self, vitamins: Vitamins) -> [Rect; 2] {
        vitamins.map(|vitamin| self.block_rect(vitamin.position().x() as u32, vitamin.position().y() as u32))
    }

    pub fn game_snip(&self) -> Rect {
        Rect::new(self.offset.x(), self.offset.y(), self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nes_dr_mario() {
        // these are the real nes dr mario coordinates
        let geometry = BottleGeometry::new(9, -1, (96, 72));
        assert_eq!(geometry.game_snip(), Rect::new(96, 72, 64, 128));
        assert_eq!(geometry.block_rect(0, 15), Rect::new(95, 191, 9, 9));
        assert_eq!(geometry.block_rect(4, 12), Rect::new(127, 167, 9, 9));
    }
}