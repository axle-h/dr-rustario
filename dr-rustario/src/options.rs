//! The match options Dr. Rustario offers on the main menu.

use crate::game::random::{random, RandomMode};
use crate::game::rules::{GameConfig, MatchRules, MatchThemes, MAX_VIRUS_LEVEL};
use crate::game::{Game, GameSpeed};
use engine::app::{MatchSettings, ThemeMode};
use engine::menu::MenuItem;
use std::str::FromStr;

pub const STAGE_NOUN: &str = "level";

const THEMES: &str = "themes";
const MODE: &str = "mode";
const LEVEL: &str = "level";
const SPEED: &str = "speed";
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
        self.config.set_players(players);
        self.config
            .set_rules(MatchRules::default_by_players(players));
    }

    pub fn menu_items(&self) -> Vec<MenuItem> {
        let modes = Self::modes(self.config.players());
        vec![
            MenuItem::select_list(
                THEMES,
                MatchThemes::names()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
                self.config.themes() as usize,
            ),
            MenuItem::select_list(
                MODE,
                modes.iter().map(|m| m.name(STAGE_NOUN)).collect(),
                modes
                    .iter()
                    .position(|&m| m == self.config.rules())
                    .unwrap_or(0),
            ),
            MenuItem::select_list(
                LEVEL,
                (0..=MAX_VIRUS_LEVEL).map(|i| i.to_string()).collect(),
                self.config.virus_level() as usize,
            ),
            MenuItem::select_list(
                SPEED,
                GameSpeed::names()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
                self.config.speed() as usize,
            ),
            MenuItem::select_list(
                RANDOM,
                RandomMode::names()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
                self.config.random() as usize,
            ),
        ]
    }

    /// returns true if the selection was one of these options
    pub fn select(&mut self, name: &str, value: &str) -> bool {
        match name {
            THEMES => self
                .config
                .set_themes(MatchThemes::from_str(value).unwrap()),
            MODE => {
                let modes = Self::modes(self.config.players());
                if let Some(mode) = modes.iter().find(|m| m.name(STAGE_NOUN) == value) {
                    self.config.set_rules(*mode);
                }
            }
            LEVEL => self.config.set_virus_level(value.parse::<u32>().unwrap()),
            SPEED => self.config.set_speed(GameSpeed::from_str(value).unwrap()),
            RANDOM => self.config.set_random(RandomMode::from_str(value).unwrap()),
            _ => return false,
        }
        true
    }

    pub fn settings(&self) -> MatchSettings {
        MatchSettings {
            rules: self.config.rules(),
            themes: match self.config.themes() {
                MatchThemes::All => ThemeMode::All,
                MatchThemes::Nes => ThemeMode::Fixed(0),
                MatchThemes::Snes => ThemeMode::Fixed(1),
                MatchThemes::N64 => ThemeMode::Fixed(2),
                MatchThemes::Particle => ThemeMode::Fixed(3),
            },
        }
    }

    pub fn games(&self) -> Result<Vec<Game>, String> {
        random(self.config.players() as usize, self.config.random())
            .into_iter()
            .map(|rand| Game::new(self.config.virus_level(), self.config.speed(), rand))
            .collect()
    }
}
