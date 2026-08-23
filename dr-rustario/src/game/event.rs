use crate::game::block::Block;
use crate::game::geometry::BottlePoint;
use crate::game::pill::VirusColor;

pub use engine::game::GameEvent;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ColoredBlock {
    pub position: BottlePoint,
    pub color: VirusColor,
    pub is_virus: bool,
}

impl ColoredBlock {
    #[cfg(test)]
    pub fn virus(x: i32, y: i32, color: VirusColor) -> Self {
        Self {
            position: BottlePoint::new(x, y),
            color,
            is_virus: true,
        }
    }

    pub fn from_block(position: BottlePoint, block: Block) -> Self {
        Self {
            position,
            color: block.destructible_color().unwrap(),
            is_virus: block.is_virus(),
        }
    }
}
