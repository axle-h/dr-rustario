use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::animation::destroy::SWEEP_DURATION;
use crate::config::Config;
use crate::event::GameEvent;
use crate::game::ai::agent::AiAgent;
use crate::game::ai::action_evaluator::ActionEvaluator;
use crate::game::ai::game_result::GameResult;
use crate::game::ai::linear::LinearCoefficients;
use crate::game::Game;
use crate::game::random::{RandomTetromino, Seed};

pub struct HeadlessGame {
    agent: AiAgent,
    game: Game,
    end_game: EndGame,
    options: HeadlessGameOptions,
    duration: Duration,
    game_over: bool,
}

impl HeadlessGame {
    pub fn new(
        rng: RandomTetromino,
        agent: AiAgent,
        options: HeadlessGameOptions,
        end_game: EndGame,
    ) -> Self {
        let mut game = Game::new(1, 0, rng);
        if options.record {
            game.start_recording();
        }

        Self {
            agent,
            game,
            duration: Duration::ZERO,
            game_over: false,
            options,
            end_game
        }
    }
    
    pub fn play(&mut self) -> GameResult {
        loop {
            if let Some(result) = self.update() {
                return result;
            }
        }
    }

    pub fn save_recording<P: AsRef<std::path::Path>>(&self, path: P) -> std::io::Result<()> {
        self.game.save_recording(path)
    }

    fn update(&mut self) -> Option<GameResult> {
        self.duration += self.options.step;

        let result = self.result();
        if self.game_over || self.end_game.is_end_game(result, self.duration) {
            return Some(result);
        }

        self.agent.act(&mut self.game);
        let mut events = self.game.empty_event_buffer();

        if let Some(event) = self.game.update(self.options.step) {
            events.push(event);
        }

        for event in events {
            match event {
                GameEvent::GameOver { .. } => {
                    self.game_over = true;
                    return Some(self.result());
                },
                GameEvent::Destroy(_) => {
                    // simulate line clear animation
                    self.duration += self.options.line_clear_delay;
                }
                _ => ()
            }
        }

        None
    }

    fn result(&self) -> GameResult {
        GameResult::new(self.game.score, self.game.lines, self.game.level, self.game_over, self.duration)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HeadlessGameOptions {
    pub line_clear_delay: Duration,
    pub step: Duration,
    pub look_ahead: usize,
    pub record: bool,
}

pub const DEFAULT_LOOKAHEAD: usize = 0;

impl Default for HeadlessGameOptions {
    fn default() -> Self {
        Self {
            step: Duration::from_millis(16), // 60hz
            line_clear_delay: SWEEP_DURATION,
            look_ahead: DEFAULT_LOOKAHEAD,
            record: false
        }   
    }
}

pub struct HeadlessGameFixture {
    config: Config,
    seed: Seed,
    game_options: HeadlessGameOptions,
    end_game: EndGame,
}

impl HeadlessGameFixture {
    pub fn new(config: Config, seed: Seed, game_options: HeadlessGameOptions, end_game: EndGame) -> Self {
        Self { config, seed, game_options, end_game }
    }
    
    pub fn next_seed(&mut self) {
        self.seed += Seed::from(1);
    }
    
    pub fn current_seed(&self) -> Seed {
        self.seed
    }
    
    pub fn play(&self, action_evaluate: ActionEvaluator) -> GameResult {
        let rng = RandomTetromino::new(
            self.config.game.random_mode,
            self.config.game.min_garbage_per_hole,
            self.seed
        );
        let agent = AiAgent::new(action_evaluate, self.game_options.look_ahead);
        let mut game = HeadlessGame::new(rng, agent, self.game_options, self.end_game);
        let result = game.play();

        if self.game_options.record {
            let system_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");
            // TODO return this somehow
            let path = format!("game-{}.json", system_time.as_secs());
            game.save_recording(path).expect("Failed to save recording");
        }

        result
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EndGame {
    pub score: u32,
    pub lines: u32,
    pub duration: Duration,
}

impl Default for EndGame {
    fn default() -> Self {
        Self::NONE
    }
}

impl EndGame {
    pub const NONE: Self = Self {
        score: u32::MAX,
        lines: u32::MAX,
        duration: Duration::MAX
    };

    pub fn of_score(score: u32) -> Self {
        Self {
            score,
            ..Default::default()
        }
    }

    pub fn of_lines(lines: u32) -> Self {
        Self {
            lines,
            ..Default::default()
        }
    }

    pub fn of_seconds(seconds: u64) -> Self {
        Self {
            duration: Duration::from_secs(seconds),
            ..Default::default()
        }
    }
    
    pub fn is_end_game(&self, result: GameResult, duration: Duration) -> bool {
        result.score() >= self.score
            || result.lines() >= self.lines
            || duration >= self.duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_fixture() -> HeadlessGameFixture {
        HeadlessGameFixture::new(
            Config::default(),
            100.into(),
            HeadlessGameOptions::default(),
            EndGame::of_seconds(5)
        )
    }
    
    #[test]
    fn runs_headless_game() {
        let fixture = test_fixture();
        let evaluator = ActionEvaluator::Linear(LinearCoefficients::default());
        let result = fixture.play(evaluator);
        assert!(result.score() > 0);
    }

    #[test]
    fn same_score_for_the_same_inputs() {
        let fixture = test_fixture();
        let evaluator = ActionEvaluator::Linear(LinearCoefficients::default());
        let result1 = fixture.play(evaluator);
        let result2 = fixture.play(evaluator);
        assert_eq!(result1, result2);
    }
}