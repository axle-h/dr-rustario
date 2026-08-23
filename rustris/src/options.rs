//! The match options Rustris offers on the main menu.

use crate::game::random::{random_tetrominos, RandomMode};
use crate::game::rules::{AiDifficulty, AiMode, GameConfig, MatchRules, MatchThemes};
use crate::game::Game;
use engine::app::ThemeMode;
use engine::menu::MenuItem;
use std::str::FromStr;

pub const STAGE_NOUN: &str = "level";
const MAX_START_LEVEL: u32 = 9;

const THEMES: &str = "themes";
const MODE: &str = "mode";
const LEVEL: &str = "level";
const RANDOM: &str = "random";
const AI: &str = "ai";
const AI_OFF: &str = "off";
const AI_DEMO: &str = "demo";

fn ai_names() -> Vec<String> {
    let mut names = vec![AI_OFF.to_string()];
    names.extend(AiDifficulty::ALL.iter().map(|d| format!("vs {}", d.name())));
    names.push(AI_DEMO.to_string());
    names
}

fn ai_name(mode: AiMode) -> String {
    match mode {
        AiMode::Off => AI_OFF.to_string(),
        AiMode::Opponent(difficulty) => format!("vs {}", difficulty.name()),
        AiMode::Demo => AI_DEMO.to_string(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Options {
    config: GameConfig,
}

impl Options {
    fn modes(players: u32) -> Vec<MatchRules> {
        if players == 1 {
            MatchRules::SINGLE_PLAYER_MODES.to_vec()
        } else {
            MatchRules::VS_MODES.to_vec()
        }
    }

    pub fn set_players(&mut self, players: u32) {
        self.config.players = players;
        self.config.rules = MatchRules::default_by_players(players);
    }

    /// `compact` leaves out the mode and randomiser, for a mixed match's second game
    pub fn menu_items(&self, compact: bool) -> Vec<MenuItem> {
        let modes = Self::modes(self.config.players);
        let items = vec![
            MenuItem::select_list(
                THEMES,
                MatchThemes::names()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
                self.config.themes as usize,
            ),
            MenuItem::select_list(
                MODE,
                modes.iter().map(|m| m.name(STAGE_NOUN)).collect(),
                modes
                    .iter()
                    .position(|&m| m == self.config.rules)
                    .unwrap_or(0),
            ),
            MenuItem::select_list(
                LEVEL,
                (0..=MAX_START_LEVEL).map(|i| i.to_string()).collect(),
                self.config.level as usize,
            ),
            MenuItem::select_list(
                RANDOM,
                RandomMode::names()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
                self.config.random as usize,
            ),
            MenuItem::select_list(
                AI,
                ai_names(),
                ai_names()
                    .iter()
                    .position(|n| *n == ai_name(self.config.ai))
                    .unwrap_or(0),
            ),
        ];
        if compact {
            items
                .into_iter()
                .filter(|item| item.name() != MODE && item.name() != RANDOM && item.name() != AI)
                .collect()
        } else {
            items
        }
    }

    /// returns true if the selection was one of these options
    pub fn select(&mut self, name: &str, value: &str) -> bool {
        match name {
            THEMES => self.config.themes = MatchThemes::from_str(value).unwrap(),
            MODE => {
                let modes = Self::modes(self.config.players);
                if let Some(mode) = modes.iter().find(|m| m.name(STAGE_NOUN) == value) {
                    self.config.rules = *mode;
                }
            }
            LEVEL => self.config.level = value.parse::<u32>().unwrap(),
            RANDOM => self.config.random = RandomMode::from_str(value).unwrap(),
            AI => {
                self.config.ai = if value == AI_DEMO {
                    AiMode::Demo
                } else {
                    match value
                        .strip_prefix("vs ")
                        .and_then(AiDifficulty::from_name)
                    {
                        Some(difficulty) => AiMode::Opponent(difficulty),
                        None => AiMode::Off,
                    }
                };
            }
            _ => return false,
        }
        true
    }


    /// `count` games sharing one seed, so players face the same pieces
    pub fn games(&self, count: usize) -> Vec<Game> {
        random_tetrominos(self.config.random, count)
            .into_iter()
            .map(|rand| Game::new(self.config.level, rand))
            .collect()
    }

    pub fn theme_mode(&self) -> ThemeMode {
        match self.config.themes {
            MatchThemes::All => ThemeMode::All,
            themes => ThemeMode::Fixed(themes.initial_index()),
        }
    }

    pub fn rules(&self) -> MatchRules {
        self.config.rules
    }

    pub fn ai(&self) -> AiMode {
        self.config.ai
    }

    /// the players the AI plays for and how fast, given the match's player count
    pub fn ai_players(&self, players: u32) -> Vec<(u32, std::time::Duration)> {
        match self.config.ai {
            AiMode::Off => vec![],
            AiMode::Demo => vec![(0, std::time::Duration::ZERO)],
            AiMode::Opponent(difficulty) => {
                if players > 1 {
                    vec![(players - 1, difficulty.key_delay())]
                } else {
                    vec![]
                }
            }
        }
    }
}
