use crate::config::Config;
use crate::game::event::GameEvent;
use crate::game::random::{GameRandom, random};
use crate::game::Game;
use crate::high_score::table::HighScoreTable;
use crate::high_score::NewHighScore;

use rand::{Rng, thread_rng};
use rand::prelude::ThreadRng;
use crate::game::bottle::SendGarbage;
use crate::game::metrics::GameMetrics;
use crate::game::rules::{GameConfig, MatchRules, MatchThemes};

pub struct Player {
    player: u32,
    game: Game,
    winner: bool
}

impl Player {
    pub fn new(player: u32, random: GameRandom, game_config: GameConfig) -> Result<Self, String> {
        Ok(
            Self {
                player,
                game: Game::new(player, game_config.virus_level(), game_config.speed(), random)?,
                winner: false
            }
        )
    }

    pub fn player(&self) -> u32 {
        self.player
    }

    pub fn game(&self) -> &Game {
        &self.game
    }

    pub fn game_mut(&mut self) -> &mut Game {
        &mut self.game
    }

    fn set_winner(&mut self) {
        self.winner = true;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchState {
    Normal,
    Paused,
    GameOver { high_score: Option<NewHighScore> },
}

impl MatchState {
    pub fn is_paused(&self) -> bool {
        self == &MatchState::Paused
    }

    pub fn is_game_over(&self) -> bool {
        matches!(self, MatchState::GameOver { .. })
    }

    pub fn is_normal(&self) -> bool {
        self == &MatchState::Normal
    }
}

pub struct Match {
    pub players: Vec<Player>,
    high_scores: HighScoreTable,
    state: MatchState,
    game_config: GameConfig,
    rng: ThreadRng
}

impl Match {
    pub fn new(game_config: GameConfig) -> Self {
        assert!(game_config.players() > 0);
        let randoms = random(game_config.players() as usize, game_config.random());
        Self {
            players: randoms
                .into_iter()
                .enumerate()
                .map(|(pid, rand)| Player::new(pid as u32, rand, game_config).unwrap())
                .collect::<Vec<Player>>(),
            high_scores: HighScoreTable::load().unwrap(),
            state: MatchState::Normal,
            game_config,
            rng: thread_rng()
        }
    }

    pub fn unset_flags(&mut self) {
        for player in self.players.iter_mut() {
            player.game.set_soft_drop(false);
        }
    }

    pub fn toggle_paused(&mut self) -> Option<GameEvent> {
        match self.state {
            MatchState::Normal => {
                self.state = MatchState::Paused;
                Some(GameEvent::Paused)
            }
            MatchState::Paused => {
                self.state = MatchState::Normal;
                Some(GameEvent::UnPaused)
            }
            _ => None,
        }
    }

    pub fn state(&self) -> MatchState {
        self.state
    }

    pub fn next_level_ends_match(&self, player: u32) -> bool {
        match self.game_config.rules() {
            MatchRules::LevelSprint { levels: sprint_levels } => self.player(player).game().completed_levels() + 1 >= sprint_levels,
            MatchRules::ThemeSprint => self.player(player).game().completed_levels() + 1 >= MatchThemes::count() as u32,
            _ => false
        }
    }

    pub fn set_winner(&mut self, player: u32) {
        // todo move to a SelectWinner { player: u32 } state
        self.player_mut(player).set_winner();
    }

    pub fn check_for_winning_player(&self) -> Option<u32> {
        if self.state.is_game_over() {
            return None;
        }

        if let Some(winner) = self.players.iter().find(|p| p.winner) {
            return Some(winner.player);
        }

        match self.game_config.rules() {
            MatchRules::ScoreSprint {
                score: sprint_score,
            } => {
                let best_game = self.highest_score();
                if best_game.score() >= sprint_score {
                    Some(best_game.player())
                } else {
                    None
                }
            }
            MatchRules::LevelSprint {
                levels: sprint_levels
            } => {
                let best_game = self.highest_virus_level();
                let completed_levels = best_game.virus_level() - self.game_config.virus_level();
                if completed_levels >= sprint_levels {
                    Some(best_game.player())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn maybe_set_game_over(&mut self) -> bool {
        if self.state.is_game_over() {
            return false;
        }

        let best_game = self.highest_score();

        let high_score = if self.high_scores.is_high_score(best_game.score()) {
            Some(NewHighScore::new(best_game.player(), best_game.score()))
        } else {
            None
        };

        self.state = MatchState::GameOver { high_score };
        true
    }

    pub fn mut_game<F>(&mut self, player: u32, mut f: F)
    where
        F: FnMut(&mut Game),
    {
        if self.state.is_normal() {
            let player = self.players.get_mut(player as usize).unwrap();
            f(&mut player.game)
        }
    }

    pub fn player(&self, player: u32) -> &Player {
        self.players.get(player as usize).unwrap()
    }

    pub fn player_mut(&mut self, player: u32) -> &mut Player {
        self.players.get_mut(player as usize).unwrap()
    }

    pub fn send_garbage(&mut self, from_player: u32, garbage: SendGarbage) {
        if self.players.len() < 2 {
            return;
        }

        let other_players = (0..self.players.len())
            .filter(|&p| p != from_player as usize)
            .collect::<Vec<usize>>();

        let pid = if other_players.len() == 1 {
            other_players[0]
        } else {
            other_players[self.rng.gen_range(0..other_players.len())]
        };
        self.players.get_mut(pid).unwrap().game.send_garbage(garbage);
    }

    fn highest_score(&self) -> GameMetrics {
        self.players
            .iter()
            .map(|p| p.game.metrics())
            .max_by(|x, y| x.score().cmp(&y.score()))
            .unwrap()
    }

    fn highest_virus_level(&self) -> GameMetrics {
        self.players
            .iter()
            .map(|p| p.game.metrics())
            .max_by(|x, y| x.virus_level().cmp(&y.virus_level()))
            .unwrap()
    }
}
