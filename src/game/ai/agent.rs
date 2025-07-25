use std::cmp::Ordering;
use itertools::Itertools;
use crate::game::ai::apply_inputs::ApplyInputs;
use crate::game::ai::action_evaluator::ActionEvaluator;
use crate::game::ai::input_search::{InputSearch, InputSequenceResult};
use crate::game::ai::input_sequence::{InputSequence, Translation};
use crate::game::{Game, GameState};
use crate::game::ai::board_features::{BoardFeatures, StackStats};
use crate::game::ai::headless_game::DEFAULT_LOOKAHEAD;
use crate::game::ai::linear::LinearCoefficients;
use crate::game::ai::neural::{NeuralGenome, TetrisNeuralNetwork};
use crate::game::board::Board;
use crate::game::tetromino::TetrominoShape;
use crate::game::recording::{GameRecording, GamePlayback};
use std::path::Path;
use std::io;

pub struct AiAgent {
    action_evaluate: ActionEvaluator,
    wait_sate: Option<AgentWaitState>,
    look_ahead: usize,
    /// Optional recording of agent decisions
    recording: Option<GameRecording>,
    /// Optional playback for replaying recorded decisions
    playback: Option<GamePlayback>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum AgentWaitState {
    Spawn,
    SoftDrop(InputSequence),
    Alt(TetrominoShape, InputSequence),
}

impl AiAgent {
    pub fn new(action_evaluate: ActionEvaluator, look_ahead: usize) -> Self {
        Self {
            action_evaluate,
            wait_sate: None,
            look_ahead,
            recording: None,
            playback: None,
        }
    }
    
    pub fn default_linear() -> Self {
        Self::new(ActionEvaluator::Linear(LinearCoefficients::default()), DEFAULT_LOOKAHEAD)
    }
    
    pub fn default_neural() -> Self {
        Self::new(ActionEvaluator::NeuralNetwork(TetrisNeuralNetwork::default()), DEFAULT_LOOKAHEAD)
    }

    fn apply_inputs(&mut self, game: &mut Game, inputs: &InputSequence) {
        let (before_soft_drop, after_soft_drop) = inputs.split_at_soft_drop();
        game.apply_inputs(&before_soft_drop);

        if let Some(after_soft_drop) = after_soft_drop {
            self.wait_sate = Some(AgentWaitState::SoftDrop(after_soft_drop));
        } else {
            self.wait_sate = Some(AgentWaitState::Spawn);
        }
    }

    pub fn reset(&mut self) {
        self.wait_sate = None;
    }

    pub fn act(&mut self, game: &mut Game) {
        // Handle wait states (this works for both playback and AI modes)
        if let Some(wait_state) = self.wait_sate.clone() {
            match wait_state {
                AgentWaitState::Spawn => {
                    if matches!(game.state, GameState::Spawn(_, _)) {
                        self.wait_sate = None;
                    }
                }
                AgentWaitState::SoftDrop(post_soft_drop_inputs)  => {
                    match game.state {
                        GameState::Fall(_) => {
                            // continue soft dropping until a lock
                            game.set_soft_drop(true);
                            return;
                        }
                        GameState::Lock(_) => {
                            // if we are in a lock state, we can apply the soft drop inputs
                            self.wait_sate = None;
                            self.apply_inputs(game, &post_soft_drop_inputs);
                        }
                        _ => (),
                    }
                }
                AgentWaitState::Alt(alt_shape, alt_inputs) => {
                    if matches!(game.state, GameState::Fall(_)) {
                        if let Some(shape) = game.board.tetromino().map(|t| t.shape()) {
                            if shape == alt_shape {
                                // we are in the alt state, apply the inputs
                                self.wait_sate = None;
                                self.apply_inputs(game, &alt_inputs);
                            }
                        }
                    }
                }
            }
            return; // wait for wait state to be resolved
        }

        if !matches!(game.state, GameState::Fall(_)) {
            return; // only act when in a fall state
        }
        
        if let Some(shape) = game.board.tetromino().map(|t| t.shape()) {
            let (best_inputs, is_alt) = if let Some(playback) = &mut self.playback {
                // Playback mode: get the recorded decision
                if let Some(recorded_input) = playback.next_decision() {
                    match recorded_input.keys {
                        Some(input_sequence) => (input_sequence, recorded_input.is_alt),
                        None => {
                            // This was a null decision - do nothing
                            return;
                        }
                    }
                } else {
                    // Playback finished
                    return;
                }
            } else {
                // Normal AI decision-making when not in playback mode
                let best_result = self.best_move(game, shape, &game.random.peek_buffer());

                let (alt_next_shape, alt_next_peek) = game.hold
                    .map(|state| (state.shape, 0..))
                    .unwrap_or_else(|| (game.random.peek(), 1..));

                let alt_best_move = self.best_move(game, alt_next_shape, &game.random.peek_buffer()[alt_next_peek]);
                let (best_inputs, is_alt) = match (best_result, alt_best_move) {
                    (None, None) => {
                        // Record a null decision if no moves are possible
                        if let Some(recording) = &mut self.recording {
                            recording.record_null_decision();
                        }
                        return;
                    },
                    (Some((m, _)), None) => (m, false),
                    (None, Some((m, _))) => (m, true),
                    (Some((m1, c1)), Some((m2, c2))) =>
                        if c1 < c2 { (m1, false) } else { (m2, true) }
                };

                // Record the decision we're making (only in AI mode)
                if let Some(recording) = &mut self.recording {
                    recording.record_decision(best_inputs.clone(), is_alt);
                }

                (best_inputs, is_alt)
            };

            // Execute the decision (same code path for both playback and AI)
            if is_alt {
                let alt_next_shape = game.hold
                    .map(|state| state.shape)
                    .unwrap_or_else(|| game.random.peek());

                // hold the current and wait for the alt shape to fall
                self.wait_sate = Some(AgentWaitState::Alt(alt_next_shape, best_inputs));
                game.hold();
            } else {
                self.apply_inputs(game, &best_inputs);
            }
        } else {
            unreachable!("the game should nver be in a fall state without a tetromino");
        }
    }
    
    fn best_move(&self, game: &Game, shape: TetrominoShape, peek: &[TetrominoShape]) -> Option<(InputSequence, f64)> {
        // Normal AI decision-making (playback is now handled at the act level)
        self.best_single_move(game.board, game.board.stack_stats(), shape)
            .map(|(result, cost)| (result.inputs().clone(), cost))
    }


    fn best_single_move(&self, board_from: Board, stack_stats_before: StackStats, shape: TetrominoShape) -> Option<(InputSequenceResult, f64)> {
        board_from.search_all_inputs(shape)
            .into_iter()
            .map(|r| {
                let score = self.action_evaluate.evaluate_action(&board_from, stack_stats_before, r.board());
                (r, score)
            })
            .max_by(|m1, m2| self.compare_moves(m1, m2))
    }

    fn compare_moves(&self, (result1, cost1): &(InputSequenceResult, f64), (result2, cost2): &(InputSequenceResult, f64)) -> Ordering {
        // if multiple moves have teh same score then we must order them to deterministically choose
        cost1.total_cmp(cost2).then_with(|| result1.inputs().cmp(&result2.inputs()))
    }

    /// Start recording agent decisions
    pub fn start_recording(&mut self) -> Result<(), String> {
        if self.playback.is_some() {
            Err("Cannot start recording while in playback mode".to_string())
        } else {
            self.recording = Some(GameRecording::new());
            Ok(())
        }
    }

    /// Save the current recording to a file
    pub fn save_recording<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        if let Some(recording) = &self.recording {
            recording.save_to_file(path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Start playback from a file
    pub fn start_playback<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        if self.recording.is_some() {
            Err("Cannot start playback while in recording mode".to_string())
        } else {
            self.playback = Some(GamePlayback::load_from_file(path).map_err(|e| e.to_string())?);
            Ok(())
        }
    }
}

impl Default for AiAgent {
    fn default() -> Self {
        Self::default_neural()
    }
}