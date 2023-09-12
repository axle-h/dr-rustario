use num_format::{Locale, ToFormattedString};
use strum::IntoEnumIterator;
use crate::game::GameSpeed;

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr, strum::EnumIter, strum::EnumString)]
pub enum MatchThemes {
    /// Run themes in order, switching at the next level
    #[strum(serialize = "all")]
    All = 0,

    #[strum(serialize = "nes")]
    Nes = 1,

    #[strum(serialize = "snes")]
    Snes = 2,

    #[strum(serialize = "n64")]
    N64 = 3
}

impl MatchThemes {
    pub fn names() -> Vec<&'static str> {
        Self::iter().map(|e| e.into()).collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchRules {
    /// Endless game, player with the highest score at the end wins
    Marathon,
    /// Race to a certain number of virus levels
    LevelSprint { levels: u32 },
    /// Race to some score
    ScoreSprint { score: u32 },
}

impl MatchRules {
    pub const ONE_LEVEL_SPRINT: Self = Self::LevelSprint { levels: 1 };
    pub const DEFAULT_SCORE_SPRINT: Self = Self::ScoreSprint { score: 10_000 };

    pub const VS_MODES: [Self; 2] = [Self::ONE_LEVEL_SPRINT, Self::DEFAULT_SCORE_SPRINT];
    pub const SINGLE_PLAYER_MODES: [Self; 3] = [Self::Marathon, Self::ONE_LEVEL_SPRINT, Self::DEFAULT_SCORE_SPRINT];

    pub fn name(&self) -> String {
        match self {
            MatchRules::Marathon => "marathon".to_string(),
            MatchRules::LevelSprint { levels } => format!("{} level sprint", levels),
            MatchRules::ScoreSprint { score } => format!("{} point sprint", score.to_formatted_string(&Locale::en)),
        }
    }

    pub fn default_by_players(players: u32) -> Self {
        if players == 1 {
            MatchRules::Marathon
        } else {
            MatchRules::ONE_LEVEL_SPRINT
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameConfig {
    players: u32,
    virus_level: u32,
    speed: GameSpeed,
    themes: MatchThemes,
    rules: MatchRules
}

impl GameConfig {
    pub fn new(players: u32, virus_level: u32, speed: GameSpeed, themes: MatchThemes, rules: MatchRules) -> Self {
        Self { players, virus_level, speed, themes, rules }
    }

    pub fn players(&self) -> u32 {
        self.players
    }
    pub fn is_single_player(&self) -> bool {
        self.players == 1
    }

    pub fn virus_level(&self) -> u32 {
        self.virus_level
    }
    pub fn speed(&self) -> GameSpeed {
        self.speed
    }
    pub fn themes(&self) -> MatchThemes {
        self.themes
    }
    pub fn rules(&self) -> MatchRules {
        self.rules
    }

    pub fn set_players(&mut self, players: u32) {
        self.players = players;
    }
    pub fn set_virus_level(&mut self, virus_level: u32) {
        self.virus_level = virus_level;
    }
    pub fn set_speed(&mut self, speed: GameSpeed) {
        self.speed = speed;
    }
    pub fn set_themes(&mut self, themes: MatchThemes) {
        self.themes = themes;
    }
    pub fn set_rules(&mut self, rules: MatchRules) {
        self.rules = rules;
    }
}

impl Default for GameConfig {
    fn default() -> Self {
        Self::new(1, 0, GameSpeed::Medium, MatchThemes::All, MatchRules::Marathon)
    }
}