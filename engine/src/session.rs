//! A match: the players, their games, who is winning and how garbage moves between them.

use crate::game::{Attack, Game, StageState};
use crate::high_score::table::HighScoreTable;
use crate::high_score::NewHighScore;
use num_format::{Locale, ToFormattedString};
use rand::prelude::ThreadRng;
use rand::{rng, RngExt};

/// How a match is won. Game-neutral: stages are whatever a game calls a stage (a cleared
/// bottle, ten lines...).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchRules {
    /// endless; the highest score when everyone is out wins
    Marathon,
    /// first to complete this many stages
    StageSprint { stages: u32 },
    /// first to this score
    ScoreSprint { score: u32 },
    /// one stage per theme, in order
    ThemeSprint,
}

impl MatchRules {
    pub const ONE_STAGE_SPRINT: Self = Self::StageSprint { stages: 1 };
    pub const DEFAULT_SCORE_SPRINT: Self = Self::ScoreSprint { score: 10_000 };
    pub const VS_MODES: [Self; 3] = [
        Self::ONE_STAGE_SPRINT,
        Self::ThemeSprint,
        Self::DEFAULT_SCORE_SPRINT,
    ];
    pub const SINGLE_PLAYER_MODES: [Self; 4] = [
        Self::Marathon,
        Self::ONE_STAGE_SPRINT,
        Self::ThemeSprint,
        Self::DEFAULT_SCORE_SPRINT,
    ];

    pub fn name(&self, stage_noun: &str) -> String {
        match self {
            MatchRules::Marathon => "marathon".to_string(),
            MatchRules::StageSprint { stages } => format!("{} {} sprint", stages, stage_noun),
            MatchRules::ScoreSprint { score } => {
                format!("{} point sprint", score.to_formatted_string(&Locale::en))
            }
            MatchRules::ThemeSprint => "theme sprint".to_string(),
        }
    }

    pub fn allow_manual_theme_change(&self) -> bool {
        self != &Self::ThemeSprint
    }

    pub fn default_by_players(players: u32) -> Self {
        if players == 1 {
            MatchRules::Marathon
        } else {
            MatchRules::ONE_STAGE_SPRINT
        }
    }
}

pub struct Player<G: Game> {
    player: u32,
    game: G,
    winner: bool,
}

impl<G: Game> Player<G> {
    pub fn new(player: u32, game: G) -> Self {
        Self {
            player,
            game,
            winner: false,
        }
    }

    pub fn player(&self) -> u32 {
        self.player
    }

    pub fn game(&self) -> &G {
        &self.game
    }

    pub fn game_mut(&mut self) -> &mut G {
        &mut self.game
    }

    pub fn is_winner(&self) -> bool {
        self.winner
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

pub struct Match<G: Game> {
    pub players: Vec<Player<G>>,
    high_scores: HighScoreTable,
    state: MatchState,
    rules: MatchRules,
    /// stages in a theme sprint
    theme_count: u32,
    rng: ThreadRng,
}

impl<G: Game> Match<G> {
    pub fn new(games: Vec<G>, rules: MatchRules, theme_count: u32) -> Self {
        assert!(!games.is_empty());
        Self {
            players: games
                .into_iter()
                .enumerate()
                .map(|(pid, game)| Player::new(pid as u32, game))
                .collect(),
            high_scores: HighScoreTable::load().unwrap(),
            state: MatchState::Normal,
            rules,
            theme_count,
            rng: rng(),
        }
    }

    pub fn rules(&self) -> MatchRules {
        self.rules
    }

    pub fn player_count(&self) -> u32 {
        self.players.len() as u32
    }

    pub fn is_single_player(&self) -> bool {
        self.players.len() == 1
    }

    pub fn unset_flags(&mut self) {
        for player in self.players.iter_mut() {
            player.game.set_soft_drop(false);
        }
    }

    /// returns true if the pause state changed
    pub fn toggle_paused(&mut self) -> Option<bool> {
        match self.state {
            MatchState::Normal => {
                self.state = MatchState::Paused;
                Some(true)
            }
            MatchState::Paused => {
                self.state = MatchState::Normal;
                Some(false)
            }
            _ => None,
        }
    }

    pub fn state(&self) -> MatchState {
        self.state
    }

    fn sprint_stages(&self) -> Option<u32> {
        match self.rules {
            MatchRules::StageSprint { stages } => Some(stages),
            MatchRules::ThemeSprint => Some(self.theme_count),
            _ => None,
        }
    }

    /// whether completing the stage this player is on ends the match
    pub fn next_stage_ends_match(&self, player: u32) -> bool {
        match self.sprint_stages() {
            Some(stages) => self.player(player).game().completed_stages() + 1 >= stages,
            None => false,
        }
    }

    pub fn set_winner(&mut self, player: u32) {
        self.player_mut(player).winner = true;
    }

    pub fn check_for_winning_player(&self) -> Option<u32> {
        if self.state.is_game_over() {
            return None;
        }

        if let Some(winner) = self.players.iter().find(|p| p.winner) {
            return Some(winner.player);
        }

        match self.rules {
            MatchRules::ScoreSprint {
                score: sprint_score,
            } => {
                let best = self.highest_score();
                if best.game.score() >= sprint_score {
                    Some(best.player)
                } else {
                    None
                }
            }
            MatchRules::StageSprint { .. } | MatchRules::ThemeSprint => {
                let stages = self.sprint_stages().unwrap_or(u32::MAX);
                let best = self.most_stages();
                if best.game.completed_stages() >= stages {
                    Some(best.player)
                } else {
                    None
                }
            }
            MatchRules::Marathon => None,
        }
    }

    /// the player whose theme music should be played: a declared winner, otherwise whoever
    /// has completed the most stages (score breaks ties). `None` when exactly tied.
    pub fn leading_player(&self) -> Option<u32> {
        if let Some(winner) = self.players.iter().find(|p| p.winner) {
            return Some(winner.player);
        }
        let mut ranked = self
            .players
            .iter()
            .map(|p| (p.game.completed_stages(), p.game.score(), p.player))
            .collect::<Vec<_>>();
        ranked.sort_by_key(|(stages, score, _)| std::cmp::Reverse((*stages, *score)));
        match ranked.as_slice() {
            [] => None,
            [best] => Some(best.2),
            [best, second, ..] if (best.0, best.1) == (second.0, second.1) => None,
            [best, ..] => Some(best.2),
        }
    }

    pub fn maybe_set_game_over(&mut self) -> bool {
        if self.state.is_game_over() {
            return false;
        }

        let best = self.highest_score();
        let high_score = if self.high_scores.is_high_score(best.game.score()) {
            Some(NewHighScore::new(best.player, best.game.score()))
        } else {
            None
        };

        self.state = MatchState::GameOver { high_score };
        true
    }

    pub fn mut_game<F>(&mut self, player: u32, mut f: F)
    where
        F: FnMut(&mut G),
    {
        if self.state.is_normal() {
            let player = self.players.get_mut(player as usize).unwrap();
            f(&mut player.game)
        }
    }

    pub fn player(&self, player: u32) -> &Player<G> {
        self.players.get(player as usize).unwrap()
    }

    pub fn player_mut(&mut self, player: u32) -> &mut Player<G> {
        self.players.get_mut(player as usize).unwrap()
    }

    /// route an attack to a random other player
    pub fn send_attack(&mut self, from_player: u32, attack: Attack) {
        if self.players.len() < 2 {
            return;
        }

        let other_players = (0..self.players.len())
            .filter(|&p| p != from_player as usize)
            .collect::<Vec<usize>>();

        let pid = if other_players.len() == 1 {
            other_players[0]
        } else {
            other_players[self.rng.random_range(0..other_players.len())]
        };
        self.players
            .get_mut(pid)
            .unwrap()
            .game
            .receive_attack(attack);
    }

    pub fn stage_state(&self, player: u32) -> StageState {
        self.player(player).game().stage_state()
    }

    fn highest_score(&self) -> &Player<G> {
        self.players
            .iter()
            .max_by_key(|p| p.game.score())
            .unwrap()
    }

    fn most_stages(&self) -> &Player<G> {
        self.players
            .iter()
            .max_by_key(|p| (p.game.completed_stages(), p.game.score()))
            .unwrap()
    }
}
