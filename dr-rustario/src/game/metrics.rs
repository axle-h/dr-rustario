use crate::game::pill::PillShape;
use crate::game::random::PEEK_SIZE;
use crate::game::GameSpeed;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameMetrics {
    player: u32,
    virus_level: u32,
    speed: GameSpeed,
    viruses: u32,
    score: u32,
    queue: [PillShape; PEEK_SIZE],
    hold: Option<PillShape>,
}

impl GameMetrics {
    pub fn new(
        player: u32,
        virus_level: u32,
        speed: GameSpeed,
        viruses: u32,
        score: u32,
        queue: [PillShape; PEEK_SIZE],
        hold: Option<PillShape>,
    ) -> Self {
        Self {
            player,
            virus_level,
            speed,
            viruses,
            score,
            queue,
            hold,
        }
    }

    pub fn player(&self) -> u32 {
        self.player
    }
    pub fn virus_level(&self) -> u32 {
        self.virus_level
    }
    pub fn virus_count(&self) -> u32 {
        self.viruses
    }
    pub fn score(&self) -> u32 {
        self.score
    }
    pub fn queue(&self) -> [PillShape; 5] {
        self.queue
    }
    pub fn hold(&self) -> Option<PillShape> {
        self.hold
    }
}
