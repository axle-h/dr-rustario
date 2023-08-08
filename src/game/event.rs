use std::collections::HashSet;
use crate::game::geometry::BottlePoint;
use crate::game::pill::{Garbage, Vitamins};

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
    ReceivedGarbage { player: u32, garbage: Vec<Garbage> },
    DropGarbage,
    Spawn { player: u32, vitamins: Vitamins },
    GameOver { player: u32 },

    Lock { player: u32, vitamins: Vitamins, hard_or_soft_dropped: bool },
    Destroy(HashSet<BottlePoint>),
}