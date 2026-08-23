use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use crate::game::ai::game_result::GameResult;
use crate::game::ai::headless_game::EndGame;
use crate::game::ai::mutation::RateLimits;

/// what the genetic algorithm is optimising for
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Objective {
    /// do not lose: a game that is not over beats any game that is, then higher score wins
    Survival,
    /// maximise tetris line clears within a fixed piece budget, then higher score wins
    Score,
}

impl Display for Objective {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Objective::Survival => write!(f, "survival"),
            Objective::Score => write!(f, "score"),
        }
    }
}

impl Objective {
    /// fitness used for weighting parents during selection (must be >= 0)
    pub fn fitness(&self, result: &GameResult) -> f64 {
        match self {
            Objective::Survival => result.score() as f64,
            Objective::Score => result.tetris_lines() as f64,
        }
    }

    /// ordering of two results, `Greater` means `a` is the better result
    pub fn cmp(&self, a: &GameResult, b: &GameResult) -> Ordering {
        match self {
            Objective::Survival => b.game_over().cmp(&a.game_over())
                .then_with(|| a.score().cmp(&b.score())),
            Objective::Score => a.tetris_lines().cmp(&b.tetris_lines())
                .then_with(|| a.score().cmp(&b.score())),
        }
    }
}

/// a phase of training: the objective plus everything about how games are evaluated and genomes mutated
#[derive(Clone, Debug)]
pub struct Phase {
    pub objective: Objective,
    pub end_game: EndGame,
    pub seeds_per_game: usize,
    pub mutation_rate: RateLimits,
    pub crossover_rate: RateLimits,
    /// magnitude of a coefficient nudge when a gene mutates
    pub mutation_step: f64,
    pub max_generations: usize,
}

impl Phase {
    /// train from scratch until a member survives `line_cap` lines
    pub fn survival(line_cap: u32) -> Self {
        Self {
            objective: Objective::Survival,
            end_game: EndGame::of_lines(line_cap),
            seeds_per_game: 1,
            mutation_rate: RateLimits::new(0.1 ..= 0.20),
            crossover_rate: RateLimits::new(0.1 ..= 0.20),
            mutation_step: 0.1,
            max_generations: usize::MAX,
        }
    }

    /// gently fine-tune an already surviving model for tetris play within `piece_cap` pieces
    pub fn score(piece_cap: u32) -> Self {
        Self {
            objective: Objective::Score,
            end_game: EndGame::of_pieces(piece_cap),
            seeds_per_game: 3,
            mutation_rate: RateLimits::new(0.01 ..= 0.05),
            crossover_rate: RateLimits::new(0.01 ..= 0.05),
            mutation_step: 0.02,
            max_generations: usize::MAX,
        }
    }

    pub fn with_max_generations(mut self, max_generations: usize) -> Self {
        self.max_generations = max_generations;
        self
    }

    /// the survival phase is complete once a member has reached the line cap without losing
    pub fn is_complete(&self, best: &GameResult) -> bool {
        match self.objective {
            Objective::Survival => !best.game_over() && best.lines() >= self.end_game.lines,
            Objective::Score => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use super::*;

    fn result(score: u32, lines: u32, game_over: bool, tetris_lines: u32) -> GameResult {
        GameResult::new(score, lines, 0, game_over, Duration::ZERO).with_pieces(0, tetris_lines)
    }

    #[test]
    fn survival_prefers_not_losing_over_score() {
        let alive = result(10, 10, false, 0);
        let dead = result(1000, 100, true, 40);
        assert_eq!(Objective::Survival.cmp(&alive, &dead), Ordering::Greater);
        assert_eq!(Objective::Survival.cmp(&dead, &alive), Ordering::Less);
        assert_eq!(Objective::Survival.cmp(&alive, &result(20, 10, false, 0)), Ordering::Less);
    }

    #[test]
    fn score_prefers_tetris_lines_regardless_of_game_over() {
        let timid = result(5000, 100, false, 0);
        let aggressive = result(100, 8, true, 8);
        assert_eq!(Objective::Score.cmp(&aggressive, &timid), Ordering::Greater);
        // same tetris lines, break tie on score
        assert_eq!(Objective::Score.cmp(&result(100, 8, true, 8), &result(200, 8, false, 8)), Ordering::Less);
    }

    #[test]
    fn survival_phase_completes_at_the_line_cap() {
        let phase = Phase::survival(100);
        assert!(!phase.is_complete(&result(0, 99, false, 0)));
        assert!(!phase.is_complete(&result(0, 100, true, 0)));
        assert!(phase.is_complete(&result(0, 100, false, 0)));
        assert!(!Phase::score(10).is_complete(&result(0, 1000, false, 1000)));
    }
}
