use crate::animate::dr::{DrAnimation, DrAnimationType};
use std::time::Duration;

// delay until game over screen is displayed
const GAME_OVER_SCREEN_DELAY: Duration = Duration::from_secs(3);

// delay after game over screen visible that the game over animation is complete
const GAME_OVER_SCREEN_VISIBLE_FOR: Duration = Duration::from_secs(3);

const GAME_OVER_FRAME_DURATION: Duration = Duration::from_millis(300);

#[derive(Clone, Debug)]
pub struct State {
    duration: Duration,
    dr: DrAnimation,
    game_over_screen_frame: usize,
    is_complete: bool,
    is_dismissed: bool,
}

impl State {
    fn new(dr: DrAnimation) -> Self {
        Self {
            dr,
            duration: Duration::ZERO,
            game_over_screen_frame: 0,
            is_complete: false,
            is_dismissed: false,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.is_complete || self.is_dismissed
    }

    pub fn is_dismissed(&self) -> bool {
        self.is_dismissed
    }

    pub fn dr_frame(&self) -> usize {
        self.dr.frame()
    }

    pub fn game_over_screen_frame(&self) -> Option<usize> {
        if self.duration >= GAME_OVER_SCREEN_DELAY {
            Some(self.game_over_screen_frame)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct GameOverAnimation {
    game_over_screen_frames: usize,
    dr_type: DrAnimationType,
    dr_frames: usize,
    state: Option<State>,
}

impl GameOverAnimation {
    pub fn new(game_over_screen_frames: usize, dr_type: DrAnimationType, dr_frames: usize) -> Self {
        Self {
            game_over_screen_frames,
            dr_type,
            dr_frames,
            state: None,
        }
    }

    pub fn update(&mut self, delta: Duration) {
        if let Some(state) = self.state.as_mut() {
            state.duration += delta;
            state.dr.update(delta);
            state.is_complete =
                state.duration >= (GAME_OVER_SCREEN_DELAY + GAME_OVER_SCREEN_VISIBLE_FOR);
            state.game_over_screen_frame =
                (state.duration.as_millis() / GAME_OVER_FRAME_DURATION.as_millis()) as usize
                    % self.game_over_screen_frames;
        }
    }

    pub fn game_over(&mut self) {
        let dr = DrAnimation::new(self.dr_type, self.dr_frames);
        self.state = Some(State::new(dr));
    }

    pub fn state(&self) -> Option<&State> {
        self.state.as_ref()
    }

    pub fn dismiss(&mut self) {
        if let Some(state) = self.state.as_mut() {
            state.is_dismissed = true;
        }
    }
}
