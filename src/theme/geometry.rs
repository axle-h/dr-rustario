use sdl2::rect::{Point, Rect};
use crate::game::bottle::{BOTTLE_HEIGHT, BOTTLE_WIDTH};
use crate::game::geometry::BottlePoint;
use crate::game::pill::Vitamins;

#[derive(Debug, Copy, Clone)]
pub struct BottleGeometry {
    raw_block_size: u32,
    block_size: u32,
    block_overlap: i32,
    height: u32,
    width: u32,
    offset: Point,
}

impl BottleGeometry {
    pub fn new<P: Into<Point>>(raw_block_size: u32, block_overlap: i32, offset: P) -> Self {
        let block_size = (raw_block_size as i32 + block_overlap) as u32;
        let height = block_size * BOTTLE_HEIGHT;
        let width = block_size * BOTTLE_WIDTH;
        Self {
            raw_block_size,
            block_size,
            block_overlap,
            height,
            width,
            offset: offset.into(),
        }
    }

    fn i_to_x(&self, i: i32) -> i32 {
        i * self.block_size as i32 + self.offset.x()
    }

    fn j_to_y(&self, j: i32) -> i32 {
        j * self.block_size as i32 + self.offset.y()
    }

    pub fn block_overlap(&self) -> i32 {
        self.block_overlap
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

    pub fn point<P : Into<BottlePoint>>(&self, point: P) -> Point {
        let point = point.into();
        Point::new(
            self.i_to_x(point.x()),
            self.j_to_y(point.y())
        )
    }

    pub fn raw_block<P : Into<BottlePoint>>(&self, point: P) -> Rect {
        let point = point.into();
        Rect::new(
            self.i_to_x(point.x()),
            self.j_to_y(point.y()),
            self.raw_block_size,
            self.raw_block_size,
        )
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
        let geometry = BottleGeometry::new(7, 1, (96, 72));
        assert_eq!(geometry.game_snip(), Rect::new(96, 72, 64, 128));
        assert_eq!(geometry.raw_block((0, 15)), Rect::new(96, 192, 7, 7));
        assert_eq!(geometry.raw_block((4, 12)), Rect::new(128, 168, 7, 7));
    }
}