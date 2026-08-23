//! The match options Rustris offers on the main menu.

use crate::game::random::{random_tetrominos, RandomMode};
use crate::game::rules::{GameConfig, MatchRules, MatchThemes};
use crate::game::Game;
use engine::app::{MatchSettings, ThemeMode};
use engine::menu::MenuItem;
use std::str::FromStr;

pub const STAGE_NOUN: &str = "level";
const MAX_START_LEVEL: u32 = 9;

const THEMES: &str = "themes";
const MODE: &str = "mode";
const LEVEL: &str = "level";
const RANDOM: &str = "random";

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

    pub fn menu_items(&self) -> Vec<MenuItem> {
        let modes = Self::modes(self.config.players);
        vec![
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
        ]
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
            _ => return false,
        }
        true
    }

    pub fn settings(&self) -> MatchSettings {
        MatchSettings {
            rules: self.config.rules,
            themes: match self.config.themes {
                MatchThemes::All => ThemeMode::All,
                themes => ThemeMode::Fixed(themes.initial_index()),
            },
        }
    }

    pub fn games(&self) -> Vec<Game> {
        random_tetrominos(self.config.random, self.config.players as usize)
            .into_iter()
            .map(|rand| Game::new(self.config.level, rand))
            .collect()
    }
}
