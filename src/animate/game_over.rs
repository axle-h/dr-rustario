use std::time::Duration;

const GAME_OVER_SCREEN_DELAY: Duration = Duration::from_secs(3);
const GAME_OVER_SCREEN_VISIBLE_FOR: Duration = Duration::from_secs(3);
const GAME_OVER_FRAME_DURATION: Duration = Duration::from_millis(300);

#[derive(Clone, Debug)]
pub struct State {
    duration: Duration,
    game_over_screen_frame: usize,
    is_complete: bool,
    is_dismissed: bool
}

impl State {
    fn new() -> Self {
        Self { duration: Duration::ZERO, game_over_screen_frame: 0, is_complete: false, is_dismissed: false }
    }

    pub fn is_complete(&self) -> bool {
        self.is_complete || self.is_dismissed
    }

    pub fn is_dismissed(&self) -> bool {
        self.is_dismissed
    }

    pub fn dr_frame(&self) -> usize {
        // todo as needed
        0
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
    state: Option<State>
}

impl GameOverAnimation {
    pub fn new(game_over_screen_frames: usize) -> Self {
        Self { game_over_screen_frames, state: None }
    }

    pub fn update(&mut self, delta: Duration) {
        if let Some(state) = self.state.as_mut() {
            state.duration += delta;
            state.is_complete = state.duration >= (GAME_OVER_SCREEN_DELAY + GAME_OVER_SCREEN_VISIBLE_FOR);
            state.game_over_screen_frame =
                (state.duration.as_millis() / GAME_OVER_FRAME_DURATION.as_millis()) as usize
                    % self.game_over_screen_frames;
        }
    }

    pub fn game_over(&mut self) {
        self.state = Some(State::new());
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