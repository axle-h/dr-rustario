use crate::game::geometry::{BottlePoint, Rotation};
use crate::game::pill::{VirusColor, VitaminOrdinal};
use std::ops::Neg;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Block {
    Empty,
    /// vitamin on the active pill
    Vitamin(VirusColor, Rotation, VitaminOrdinal),

    /// vitamin on a stacked pill
    Stack(VirusColor, Rotation, VitaminOrdinal),

    /// orphaned vitamin on a stacked pill or player garbage
    Garbage(VirusColor),

    /// a virus
    Virus(VirusColor),

    /// active vitamin ghost
    Ghost(VirusColor, Rotation, VitaminOrdinal),
}

pub fn block_partner_offset(rotation: Rotation, ordinal: VitaminOrdinal) -> BottlePoint {
    let offset = match rotation {
        Rotation::North => BottlePoint::new(1, 0),
        Rotation::East => BottlePoint::new(0, 1),
        Rotation::South => BottlePoint::new(-1, 0),
        Rotation::West => BottlePoint::new(0, -1),
    };

    match ordinal {
        VitaminOrdinal::Left => offset,
        VitaminOrdinal::Right => offset.neg(),
    }
}

impl Block {
    pub fn is_destructible(&self) -> bool {
        matches!(
            self,
            Block::Virus(_) | Block::Garbage(_) | Block::Stack(_, _, _)
        )
    }

    pub fn is_empty(&self) -> bool {
        self == &Block::Empty
    }

    pub fn is_virus(&self) -> bool {
        matches!(self, Block::Virus(_))
    }

    /// gets the color of any destructible colored block
    pub fn destructible_color(&self) -> Option<VirusColor> {
        match self {
            Block::Stack(color, _, _) | Block::Garbage(color) | Block::Virus(color) => Some(*color),
            _ => None,
        }
    }

    pub fn find_stack_partner_offset(&self) -> Option<BottlePoint> {
        if let Block::Stack(_, rotation, ordinal) = self {
            Some(block_partner_offset(*rotation, *ordinal))
        } else {
            None
        }
    }
}
