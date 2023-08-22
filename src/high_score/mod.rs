pub mod table;
pub mod render;
pub mod event;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NewHighScore {
    pub player: u32,
    pub score: u32,
}

impl NewHighScore {
    pub fn new(player: u32, score: u32) -> Self {
        Self { player, score }
    }
}