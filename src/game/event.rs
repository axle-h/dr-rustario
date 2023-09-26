use std::collections::HashSet;
use crate::game::block::Block;
use crate::game::bottle::SendGarbage;
use crate::game::geometry::{BottlePoint, Rotation};
use crate::game::pill::{Garbage, PillShape, VirusColor, Vitamins};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameEvent {
    Move,
    Rotate,
    Hold,
    SoftDrop,
    HardDrop {
        player: u32,
        vitamins: Vitamins,
        dropped_rows: u32,
    },
    Fall,
    SendGarbage { player: u32, garbage: SendGarbage },
    ReceivedGarbage { player: u32, garbage: Vec<Garbage> },
    DropGarbage,
    Spawn { player: u32, shape: PillShape, is_hold: bool },
    SpeedLevelUp,
    GameOver { player: u32 },
    Victory { player: u32 },
    LevelComplete { player: u32 },
    Lock { player: u32, vitamins: Vitamins, hard_or_soft_dropped: bool },
    Destroy {
        player: u32,
        blocks: Vec<ColoredBlock>,
        is_combo: bool
    },
    Paused,
    UnPaused,
    Quit,
    NextTheme,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ColoredBlock {
    pub position: BottlePoint,
    pub color: VirusColor,
    pub is_virus: bool
}

impl ColoredBlock {
    pub fn virus(x: i32, y: i32, color: VirusColor) -> Self {
        Self { position: BottlePoint::new(x, y), color, is_virus: true }
    }

    pub fn stack(x: i32, y: i32, color: VirusColor) -> Self {
        Self { position: BottlePoint::new(x, y), color, is_virus: false }
    }

    pub fn from_block(position: BottlePoint, block: Block) -> Self {
        Self {
            position,
            color: block.destructible_color().unwrap(),
            is_virus: block.is_virus()
        }
    }

}

